use crate::font::FONT5X8;

// Measured on the hardware, not guessed. Each screen is 512x64, addressed
// 1:1 - see TILE_W below for how a report maps onto it.
//
// The old WIDTH of 128 is why text came out "readable but too big": it filled
// a quarter of the panel, so everything looked magnified.
pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 64;
pub const STRIDE: usize = WIDTH / 8; // 64 bytes per row

// With the tile geometry corrected the canvas is the panel: 512x64, one
// logical row per physical row. The mapping is kept as an identity hook
// because the earlier wrong geometry made it look as though rows were being
// dropped, and a future panel variant may genuinely need one.
pub const LOGICAL_H: usize = HEIGHT;

pub fn logical_row(lrow: usize) -> usize { lrow }

// A report is a 128x32 tile: the panel reads 16 bytes per row, 32 rows,
// 512 bytes. Confirmed the hard way - feeding it 8-byte rows made every pair
// of our rows land in one panel row, so text drawn past x=64 reappeared at
// x=0 (a "wrap") and only half of each strip survived. 4-byte rows fragmented
// it further still.
//
// A full screen is therefore 4 column tiles (byte 1 = 0, 8, 16, 24 in 16-px
// units) by 2 row bands (byte 3 = 0, 32). BOTH bands must be sent: byte 7 is
// 0x20, so one report only ever covers 32 rows, and sending a single band per
// tile leaves rows 32-63 holding whatever was on the panel before.
pub const TILE_W: usize = 128;       // pixels per report
pub const TILE_STRIDE: usize = TILE_W / 8;
// Header byte 7 is 0x20: a report carries 32 rows, so each 64-px strip needs
// two of them - byte 3 = 0 and 32. Sending one 64-row report per strip left
// rows 32-63 holding whatever was on the panel before, which is exactly how
// stale text survived a full redraw.
pub const BAND_H: usize = 32;        // rows per report
pub const TILES: usize = WIDTH / TILE_W;
pub const BANDS: usize = HEIGHT / BAND_H;

pub fn clear(bits: &mut [u8; HEIGHT * STRIDE]) {
    for b in bits.iter_mut() { *b = 0; }
}

pub fn set_pixel(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize) {
    if x >= WIDTH || y >= HEIGHT { return; }
    bits[y * STRIDE + x / 8] |= 0x80 >> (x % 8);
}

pub fn draw_char(bits: &mut [u8; HEIGHT * STRIDE], px: usize, py: usize, c: u8) {
    let idx = match c {
        32..=127 => (c - 32) as usize,
        _ => 0,
    };
    let glyph = &FONT5X8[idx];
    for col in 0..5 {
        let col_byte = glyph[col];
        for row in 0..8 {
            if (col_byte >> row) & 1 == 1 {
                set_pixel(bits, px + col, py + row);
            }
        }
    }
}

pub fn draw_text(bits: &mut [u8; HEIGHT * STRIDE], px: usize, py: usize, text: &str) {
    let mut x = px;
    for c in text.bytes() {
        if x + 5 > WIDTH { break; }
        draw_char(bits, x, py, c);
        x += 6; // 5px char + 1px gap
    }
}

// --- primitives for the rig's screen layout ---------------------------------
//
// The reference is Maschine's own screen: boxed labels along the top under the
// buttons, a rule, then one column per encoder with a small caps name above a
// double-height value. That needs three things the original font code had no
// answer for - scaled text, filled/outlined/dashed boxes, and inversion for
// the selected item - so they live here rather than being open-coded per
// layout.

pub fn clear_pixel(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize) {
    if x >= WIDTH || y >= HEIGHT { return; }
    bits[y * STRIDE + x / 8] &= !(0x80 >> (x % 8));
}

// Glyphs are square on the panel now that rows map 1:1, so no horizontal
// compensation is needed. Kept as a named constant because getting this wrong
// once already cost a full round of unreadable screens.
pub const X_SCALE: usize = 1;

/// Character cell width at a given scale, gap included.
pub fn char_w(scale: usize) -> usize { 6 * X_SCALE * scale.max(1) }

/// Pixel width a string occupies at a given scale, trailing gap excluded.
pub fn text_w(text: &str, scale: usize) -> usize {
    let n = text.len();
    if n == 0 { 0 } else { n * char_w(scale) - X_SCALE * scale.max(1) }
}

pub fn draw_char_scaled(bits: &mut [u8; HEIGHT * STRIDE], px: usize, py: usize, c: u8, scale: usize) {
    let sy = scale.max(1);
    let sx = sy * X_SCALE;
    let idx = match c { 32..=127 => (c - 32) as usize, _ => 0 };
    let glyph = &FONT5X8[idx];
    for col in 0..5 {
        let col_byte = glyph[col];
        for row in 0..8 {
            if (col_byte >> row) & 1 == 1 {
                for dy in 0..sy {
                    for dx in 0..sx {
                        set_pixel(bits, px + col * sx + dx, py + row * sy + dy);
                    }
                }
            }
        }
    }
}

pub fn draw_text_scaled(bits: &mut [u8; HEIGHT * STRIDE], px: usize, py: usize, text: &str, scale: usize) {
    let s = scale.max(1);
    let mut x = px;
    for c in text.bytes() {
        if x + 5 * s * X_SCALE > WIDTH { break; }
        draw_char_scaled(bits, x, py, c, s);
        x += char_w(s);
    }
}

pub fn hline(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize) {
    for i in 0..w { set_pixel(bits, x + i, y); }
}

pub fn vline(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, h: usize) {
    for i in 0..h { set_pixel(bits, x, y + i); }
}

/// Every other pixel - the separator Maschine draws under its tab row.
pub fn dotted_hline(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize) {
    let mut i = 0;
    while i < w { set_pixel(bits, x + i, y); i += 2; }
}

pub fn fill_rect(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize, h: usize) {
    for dy in 0..h { for dx in 0..w { set_pixel(bits, x + dx, y + dy); } }
}

pub fn rect(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize, h: usize) {
    if w == 0 || h == 0 { return; }
    hline(bits, x, y, w);
    hline(bits, x, y + h - 1, w);
    vline(bits, x, y, h);
    vline(bits, x + w - 1, y, h);
}

/// Outline drawn every other pixel - Maschine's "available but not selected"
/// box, distinct from a solid one at a glance.
pub fn dashed_rect(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize, h: usize) {
    if w == 0 || h == 0 { return; }
    let mut i = 0;
    while i < w { set_pixel(bits, x + i, y); set_pixel(bits, x + i, y + h - 1); i += 2; }
    let mut j = 0;
    while j < h { set_pixel(bits, x, y + j); set_pixel(bits, x + w - 1, y + j); j += 2; }
}

/// Swap lit and unlit inside a box. Drawing text first and inverting after is
/// how a label becomes dark-on-light without a second draw path.
pub fn invert_rect(bits: &mut [u8; HEIGHT * STRIDE], x: usize, y: usize, w: usize, h: usize) {
    for dy in 0..h {
        for dx in 0..w {
            let (px, py) = (x + dx, y + dy);
            if px >= WIDTH || py >= HEIGHT { continue; }
            bits[py * STRIDE + px / 8] ^= 0x80 >> (px % 8);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_bitmap_is_all_zeros() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        clear(&mut bits);
        assert!(bits.iter().all(|&b| b == 0));
    }

    #[test]
    fn set_pixel_top_left() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        set_pixel(&mut bits, 0, 0);
        assert_eq!(bits[0], 0x80);
    }

    #[test]
    fn set_pixel_second_in_row() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        set_pixel(&mut bits, 1, 0);
        assert_eq!(bits[0], 0x40);
    }

    #[test]
    fn set_pixel_byte_boundary() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        set_pixel(&mut bits, 8, 0);
        assert_eq!(bits[1], 0x80);
    }

    #[test]
    fn set_pixel_out_of_bounds_does_not_panic() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        set_pixel(&mut bits, 200, 200);
    }

    #[test]
    fn draw_text_marks_pixels() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        draw_text(&mut bits, 0, 0, "0");
        assert!(bits[..STRIDE].iter().any(|&b| b != 0));
    }

    #[test]
    fn draw_text_does_not_overflow() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        draw_text(&mut bits, 0, 0, "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789");
    }
}

#[cfg(test)]
mod render_dump {
    use super::*;

    /// Renders text and prints it as ASCII art so the glyph layout can be read
    /// without a panel in the loop.
    #[test]
    fn dump_text_layout() {
        let mut bits = [0u8; HEIGHT * STRIDE];
        draw_text_scaled(&mut bits, 0, 0, "AB C", 1);
        for y in 0..8 {
            let mut line = String::new();
            for x in 0..40 {
                line.push(if bits[y * STRIDE + x / 8] & (0x80 >> (x % 8)) != 0 { '#' } else { '.' });
            }
            println!("{}", line);
        }
        println!("char_w(1)={} text_w(\"AB C\",1)={}", char_w(1), text_w("AB C", 1));
    }
}
