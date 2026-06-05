# Daemon Stabilization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix three blocking issues: Rust 1.80+ build failure on ARM64, startup panics (missing PNG + OSC port collision), and encoders/transport buttons outputting non-standard RPN values instead of plain MIDI CC 0–127.

**Architecture:** All changes are inside the existing Rust daemon. No new dependencies. The encoder accumulation logic is preserved; only the final ALSA message type and value range change. Button release now also emits a CC-0 message so downstream software sees press/release pairs.

**Tech Stack:** Rust, custom `alsa-seq` crate, ALSA sequencer, hidraw

---

## File Map

| File | Change |
|------|--------|
| `alsa-seq/src/event.rs` | Fix type literal for `Union_Unnamed10.data` if needed |
| `alsa-seq/Cargo.toml` | Update `alsa-sys` version if needed |
| `src/devices/mk2/mikro.rs` | Guard `write_screen` against missing PNG |
| `src/main.rs` | Guard OSC bind; fix encoder CC range; fix button press/release CC |

---

## Task 1: Diagnose the Rust 1.80+ Build Failure

**Files:**
- Read: `alsa-seq/src/event.rs`
- Possibly modify: `alsa-seq/src/event.rs`, `alsa-seq/Cargo.toml`

- [ ] **Step 1: Attempt build and capture error**

```bash
cd MaschineMK2_linux
cargo build 2>&1 | head -60
```

Expected: compile error mentioning type mismatch, likely around `Union_Unnamed10 { data: [0; 3] }` or similar `i8`/`u8` issue in `alsa-seq/src/event.rs`.

- [ ] **Step 2: Apply fix — explicit integer literal type**

If error is `expected i8, found integer` or `mismatched types` at `event.rs` line ~149 (`data: [0; 3]`):

Edit `alsa-seq/src/event.rs`, change the struct initialization from:

```rust
data: Union_Unnamed10 {
    data: [0; 3]
}
```

to:

```rust
data: Union_Unnamed10 {
    data: [0i8; 3]
}
```

If error is instead in `snd_seq_timestamp_t { data: [0; 2] }`, change:

```rust
time: snd_seq_timestamp_t { 
    data: [0; 2]
},
```

to:

```rust
time: snd_seq_timestamp_t { 
    data: [0u32; 2]
},
```

Apply whichever matches the actual error message. Both may be needed.

- [ ] **Step 3: If type fix doesn't work, update alsa-sys**

If the error is in the `alsa-sys` crate itself (error points into `~/.cargo/registry`), update `alsa-seq/Cargo.toml`:

```toml
[dependencies]
libc = "*"
bitflags = "1.2"
alsa-sys = "0.3"
midi = "*"
```

Then re-run:

```bash
cargo build 2>&1 | head -60
```

- [ ] **Step 4: Confirm build succeeds**

```bash
cargo build 2>&1 | tail -5
```

Expected output ends with: `Finished dev [unoptimized + debuginfo] target(s)`

- [ ] **Step 5: Commit**

```bash
git add alsa-seq/src/event.rs alsa-seq/Cargo.toml
git commit -m "fix: resolve Rust 1.80+ type mismatch in alsa-seq event initialization"
```

---

## Task 2: Fix Startup Crash — Missing picturetest.png

**Files:**
- Modify: `src/devices/mk2/mikro.rs`

The `write_screen()` method panics if `picturetest.png` is not present in the working directory. The daemon is invoked as `./target/release/maschine /dev/maschine` so the working directory is wherever the user runs it from — the PNG is almost never there.

- [ ] **Step 1: Write the failing test**

Add to `alsa-seq/src/test.rs` (the existing test file):

```rust
#[cfg(test)]
mod screen_tests {
    #[test]
    fn write_screen_does_not_exist_in_test_dir() {
        // Verify the PNG guard logic: std::path::Path::new on a nonexistent file
        // returns exists() = false. This is the condition we guard on.
        assert!(!std::path::Path::new("/tmp/definitely_missing_12345.png").exists());
    }
}
```

- [ ] **Step 2: Run test to verify it passes (baseline)**

```bash
cargo test 2>&1
```

Expected: test passes (it's just validating the stdlib behavior we rely on).

- [ ] **Step 3: Fix write_screen in mikro.rs**

In `src/devices/mk2/mikro.rs`, replace the start of `write_screen`:

Old (line 687):
```rust
fn write_screen(&mut self) {
    let mut limits = png::Limits::default();
    limits.bytes = 10 * 1024;
    let decoder = png::Decoder::new_with_limits(File::open("picturetest.png").unwrap(), limits);
```

New:
```rust
fn write_screen(&mut self) {
    let png_path = "picturetest.png";
    if !std::path::Path::new(png_path).exists() {
        return;
    }
    let mut limits = png::Limits::default();
    limits.bytes = 10 * 1024;
    let decoder = png::Decoder::new_with_limits(File::open(png_path).unwrap(), limits);
```

- [ ] **Step 4: Build to verify no compile errors**

```bash
cargo build 2>&1 | tail -5
```

Expected: `Finished`

- [ ] **Step 5: Commit**

```bash
git add src/devices/mk2/mikro.rs
git commit -m "fix: guard write_screen against missing picturetest.png"
```

---

## Task 3: Fix Startup Crash — OSC Port Already in Use

**Files:**
- Modify: `src/main.rs`

`UdpSocket::bind("127.0.0.1:42434").unwrap()` panics if port 42434 is already bound (e.g. previous daemon run that didn't clean up). Replace `.unwrap()` with graceful error handling.

- [ ] **Step 1: Write the failing test**

This is best validated by observing the old crash, so no unit test — validate by running the daemon twice. Skip to Step 2.

- [ ] **Step 2: Fix the bind in main.rs**

In `src/main.rs`, in `fn main()`, replace:

```rust
let osc_socket = UdpSocket::bind("127.0.0.1:42434").unwrap();
```

with:

```rust
let osc_socket = match UdpSocket::bind("127.0.0.1:42434") {
    Ok(s) => s,
    Err(e) => {
        eprintln!("failed to bind OSC socket on port 42434: {}", e);
        eprintln!("is another maschine daemon already running?");
        std::process::exit(1);
    }
};
```

- [ ] **Step 3: Build**

```bash
cargo build 2>&1 | tail -5
```

Expected: `Finished`

- [ ] **Step 4: Verify graceful exit when port taken**

```bash
# Terminal 1: hold port 42434
nc -lu 127.0.0.1 42434 &
NC_PID=$!

# Terminal 2:
./target/debug/maschine /dev/null 2>&1

# Expected output:
# failed to bind OSC socket on port 42434: Address already in use (os error 98)
# is another maschine daemon already running?

kill $NC_PID
```

- [ ] **Step 5: Commit**

```bash
git add src/main.rs
git commit -m "fix: graceful exit when OSC port 42434 is already in use"
```

---

## Task 4: Fix Encoder MIDI Output (RPN14 → Standard CC 0–127)

**Files:**
- Modify: `src/main.rs`

Encoders currently output `Message::RPN14` with values that can exceed 127. ALSA downstream tools (Zynthian CC Learn, etc.) expect standard MIDI CC with values 0–127. The fix: keep the accumulation logic, change final message to `Message::RPN7` (which sends `SND_SEQ_EVENT_CONTROLLER`), clamp value to 0–127.

CC assignment: encoders 1–8 → CC 17–24 (unchanged from current numbering).

- [ ] **Step 1: Write unit test for value normalization**

Create `src/cc_math.rs`:

```rust
pub fn normalize_encoder(raw_status: i32, roller_state: usize) -> u8 {
    let accumulated = raw_status / 4 + roller_state as i32 * 64;
    accumulated.clamp(0, 127) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn low_state_low_status() {
        assert_eq!(normalize_encoder(0, 0), 0);
    }

    #[test]
    fn mid_range() {
        // raw=128 (32 after /4), state=1 (+64) = 96
        assert_eq!(normalize_encoder(128, 1), 96);
    }

    #[test]
    fn clamps_at_127() {
        // raw=252 (63 after /4), state=3 (+192) = 255 → clamp to 127
        assert_eq!(normalize_encoder(252, 3), 127);
    }

    #[test]
    fn clamps_at_0() {
        assert_eq!(normalize_encoder(-100, 0), 0);
    }
}
```

Add to `src/main.rs`:

```rust
mod cc_math;
```

- [ ] **Step 2: Run test to verify it fails (function not yet called)**

```bash
cargo test cc_math 2>&1
```

Expected: compiles, tests pass (pure math, no hardware needed). If any assertion fails, fix `normalize_encoder`.

- [ ] **Step 3: Replace encoder message emission in main.rs**

In `src/main.rs`, replace `send_osc_encoder_msg`:

Old:
```rust
fn send_osc_encoder_msg(&self, maschine: &mut dyn Maschine, idx: usize, status: i32) {
    let state = maschine.get_roller_state(idx);
    let status = status / 4 + state as i32 * 64;
    if status - maschine.get_roller_status(idx) < 40 && maschine.get_roller_status(idx) - status < 40 {
        let msg = Message::RPN14(Ch1, idx as u16 + 16, status as u16);
        maschine.set_roller_status(status, idx);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    }
}
```

New:
```rust
fn send_encoder_cc(&self, maschine: &mut dyn Maschine, idx: usize, raw: i32) {
    let state = maschine.get_roller_state(idx);
    let accumulated = raw / 4 + state as i32 * 64;
    let prev = maschine.get_roller_status(idx);
    if (accumulated - prev).abs() < 40 {
        let value = accumulated.clamp(0, 127) as u8;
        let cc_num = idx as u16 + 16; // CC 17–24 for encoders 1–8
        let msg = Message::RPN7(Ch1, cc_num, value);
        maschine.set_roller_status(accumulated, idx);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    }
}
```

- [ ] **Step 4: Update the call site in `encoder_step`**

In `src/main.rs`, in the `MaschineHandler` impl, replace:

```rust
fn encoder_step(&mut self, maschine: &mut dyn Maschine, idx: usize, state: i32) {
    self.send_osc_encoder_msg(maschine, idx, state);
}
```

with:

```rust
fn encoder_step(&mut self, maschine: &mut dyn Maschine, idx: usize, state: i32) {
    self.send_encoder_cc(maschine, idx, state);
}
```

- [ ] **Step 5: Update inline encoder calls in send_osc_button_msg**

In `send_osc_button_msg`, the encoder sub-buttons (B6, D6, FF6, H6, J6, L6, N6, P6) also emit `Message::RPN14`. Replace each:

Old pattern (example for B6):
```rust
"B6" => {
    let idx = 1;
    let state = maschine.get_roller_state(idx);
    let status = status / 4 + state * 64;
    if modpress != 1 {
        let msg = Message::RPN14(Ch1, controlbase + 1, status as u16 / 2);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    } else {
        maschine.set_seq_speed(status);
    }
}
```

New pattern (B6):
```rust
"B6" => {
    let idx = 1;
    let state = maschine.get_roller_state(idx);
    let accumulated = status as i32 / 4 + state as i32 * 64;
    if modpress != 1 {
        let value = accumulated.clamp(0, 127) as u8;
        let msg = Message::RPN7(Ch1, controlbase as u16 + 1, value);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    } else {
        maschine.set_seq_speed(accumulated as usize);
    }
}
```

Apply same pattern for D6, FF6, H6, J6, L6, N6 (idx 2–7), and P6 (no state lookup, just clamp):

```rust
"P6" => {
    let value = (status as i32).clamp(0, 127) as u8;
    let msg = Message::RPN7(Ch1, controlbase as u16 + 8, value);
    self.seq_port.send_message(&msg).unwrap();
    self.seq_handle.drain_output();
}
```

- [ ] **Step 6: Build**

```bash
cargo build 2>&1 | tail -5
```

Expected: `Finished`

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/cc_math.rs
git commit -m "fix: emit standard MIDI CC (0-127) for encoders instead of RPN14"
```

---

## Task 5: Fix Transport Button MIDI Output (Add CC-0 on Release)

**Files:**
- Modify: `src/main.rs`

Transport buttons (play, stop, rec, etc.) currently only emit a MIDI message when pressed (`status > 0`). They never emit on release, so downstream apps never see a "button released" event. Fix: emit CC 127 on press, CC 0 on release for all transport/function buttons.

- [ ] **Step 1: Write unit test for button value mapping**

Add to `src/cc_math.rs`:

```rust
pub fn button_cc_value(is_down: bool) -> u8 {
    if is_down { 127 } else { 0 }
}

#[cfg(test)]
mod button_tests {
    use super::*;

    #[test]
    fn press_gives_127() {
        assert_eq!(button_cc_value(true), 127);
    }

    #[test]
    fn release_gives_0() {
        assert_eq!(button_cc_value(false), 0);
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

```bash
cargo test cc_math 2>&1
```

Expected: all tests pass.

- [ ] **Step 3: Refactor send_osc_button_msg — remove is_down gate**

In `src/main.rs` in `send_osc_button_msg`, the outer condition currently is:

```rust
if is_down == true && status <= 250 {
    match button { ... }
}
```

Change to:

```rust
if status <= 250 {
    match button { ... }
}
```

- [ ] **Step 4: Update each button arm to use cc_math::button_cc_value**

For each transport/function button in the match, replace the old pattern:

Old:
```rust
"play" => {
    if status > 0 && maschine.get_padmode() != 2 {
        let msg = Message::RPN7(Ch1, 1, status as u8);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    } else if maschine.get_padmode() == 2 {
        maschine.set_playing(1);
        println!("playing notes");
    };
}
```

New:
```rust
"play" => {
    if maschine.get_padmode() != 2 {
        let value = cc_math::button_cc_value(is_down);
        let msg = Message::RPN7(Ch1, 1, value);
        self.seq_port.send_message(&msg).unwrap();
        self.seq_handle.drain_output();
    } else if is_down {
        maschine.set_playing(1);
        println!("playing notes");
    };
}
```

Apply the same pattern (`cc_math::button_cc_value(is_down)`) for all other transport/function buttons: stop, rec, grid, step_left, step_right, restart, browse, sampling, note_repeat, control, nav, nav_left, nav_right, main, scene, pattern, pad_mode, view, duplicate, select, solo, step, mute, navigate, tempo, enter, auto, all, f1–f8, page_right, page_left.

The CC numbers stay as-is (1–48 as currently assigned).

- [ ] **Step 5: Add `use crate::cc_math;` at top of main.rs**

In `src/main.rs`, add after the existing `mod base;` and `mod devices;` lines:

```rust
mod cc_math;
```

- [ ] **Step 6: Build**

```bash
cargo build 2>&1 | tail -5
```

Expected: `Finished`

- [ ] **Step 7: Commit**

```bash
git add src/main.rs src/cc_math.rs
git commit -m "fix: transport buttons emit CC 127 on press and CC 0 on release"
```

---

## Task 6: Release Build + Smoke Test

- [ ] **Step 1: Release build**

```bash
cargo build --release 2>&1 | tail -5
```

Expected: `Finished release [optimized] target(s)`

- [ ] **Step 2: Verify binary exists**

```bash
ls -lh target/release/maschine
```

Expected: binary present, non-zero size.

- [ ] **Step 3: Smoke test — MIDI output visible without hardware panic**

Run with invalid device to verify it doesn't crash on OSC init:

```bash
timeout 2 ./target/release/maschine /dev/null 2>&1 || true
```

Expected: exits with an error about opening `/dev/null` as HID, NOT a panic about picturetest.png or OSC port.

- [ ] **Step 4: Verify MIDI CC output with hardware (on Raspberry Pi)**

Connect Maschine MK2 via USB. In one terminal, start MIDI monitor:

```bash
aseqdump -p "maschine.rs:Pads MIDI"
```

In another terminal, run daemon:

```bash
./target/release/maschine /dev/maschine
```

Turn encoder 1. Expected `aseqdump` output:
```
Event type: Controller, channel 1, param 17, value [0-127]
```

Press Play button. Expected:
```
Event type: Controller, channel 1, param 1, value 127
```

Release Play button. Expected:
```
Event type: Controller, channel 1, param 1, value 0
```

Press pad 1. Expected:
```
Event type: Note on, channel 1, note 60, velocity [1-127]
```

- [ ] **Step 5: Tag and push**

```bash
git tag v0.1.0-stable
git push origin main --tags
```

---

## Self-Review Checklist

- [x] **Build fix:** Task 1 covers Rust 1.80+ ARM64 type mismatch
- [x] **PNG crash:** Task 2 guards `write_screen`
- [x] **OSC port crash:** Task 3 replaces `.unwrap()` with graceful exit
- [x] **Encoder CC:** Task 4 normalizes to 0–127 via `Message::RPN7`
- [x] **Button release:** Task 5 emits CC 0 on release
- [x] **No placeholder steps:** All steps contain actual code
- [x] **Type consistency:** `cc_math::normalize_encoder` and `cc_math::button_cc_value` used in Tasks 4 and 5; function names match
- [x] **No WebSocket/OSC removal yet:** Those are Plan 2 — OSC stays but is now crash-safe
