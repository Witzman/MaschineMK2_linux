use crate::font::FONT5X8;

// Measured on the hardware 2026-08-08, not guessed. Each screen is 512x64.
// One HID report carries 512 bytes, which at header byte 7 = 0x20 (32 rows)
// is 16 bytes per row - a 128x32 tile. A full screen is therefore 8 reports:
// 4 column tiles (header byte 1 = 0, 8, 16, 24, in 16-pixel units) by 2 row
// bands (header byte 3 = 0, 32). Verified by painting tiles and watching the
// lit area go quarter -> half -> full with no seam or gap.
//
// The old WIDTH of 128 is why text came out "readable but too big": it filled
// a quarter of the panel, so everything looked magnified.
pub const WIDTH: usize = 512;
pub const HEIGHT: usize = 64;
pub const STRIDE: usize = WIDTH / 8; // 64 bytes per row

pub const TILE_W: usize = 128;       // pixels per report
pub const TILE_STRIDE: usize = TILE_W / 8;
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
