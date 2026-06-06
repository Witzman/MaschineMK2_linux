# Design: MaschineMK2_linux Documentation & Tutorial Update

**Date:** 2026-06-06  
**Scope:** Update Zynthian tutorials and MIDI Reference to reflect 13 new commits merged into MaschineMK2_linux.  
**Approach:** Extend existing tutorial files in place (Option 1). No new files. Verified parts untouched.

---

## What Changed in the Codebase (13 commits)

| Commit area | New functionality |
|-------------|------------------|
| Web editor | Browser UI at `web/index.html`, WebSocket port 9001 — pad LED colors, per-pad note offsets, per-encoder CC numbers, preset layouts, live event stream |
| Config persistence | `maschine.json` — `pad_notes[16]` + `encoder_ccs[8]` written on change, loaded on startup |
| Display | 128×64 OLED renders note names + encoder CC values via built-in 5×8 bitmap font |
| MIDI IN port | ALSA input `MIDI Control` — NoteOn/Off 0–15 → pad LEDs; Clock/Start/Stop forwarded to `Pads MIDI` output |
| Sequencer: 8 pages | Group A–H switch 8 independent 16-step pages (was 1 page) |
| Per-step note/vel | Select step (orange LED) → Encoder 1 = velocity 0–127, Encoder 2 = note offset 0–127 |
| Euclidean fill | Shift+Group A–H fills current page with euclidean rhythm (1–8 hits, Bresenham distribution) |
| MIDI Clock Sync | External 24ppqn clock via `MIDI Control`; 6 ticks = 1 step; auto-fallback to internal BPM after 500ms silence; BPM emitted as `clock_bpm` WebSocket event |
| Daemon stabilization | Rust 1.80+ build fix, startup panic guards (missing PNG, OSC port collision), encoder CC range fix |

---

## Files to Update

| File | Change type |
|------|------------|
| `~/zynth-docs/htmldoku/project-maschine-mk2.md` | Add Part 4 `[draft]` |
| `~/zynth-docs/htmldoku/project-maschine-step-sequencer.md` | Add Part 2 `[draft]` + Part 3 `[draft]` |
| `~/zynth-docs/htmldoku/midi.md` (MIDI Reference) | Update encoder CC table, add MIDI IN port, fix SMC-PAD channel 7→6, add SINCO Conflict 10 |

After every `.md` edit: run `python3 htmldoku/generate-html.py`, commit full `docs/zynthian-Doku/`, push.

---

## Section 1: `project-maschine-mk2.md` — Part 4

**Title:** Part 4 — Web Editor, Config, MIDI IN, Display `[draft]`

**Prerequisite:** Part 1 complete (daemon running as systemd service).

**Steps:**

1. Rsync updated source to Pi (includes `web/` folder and new source files)
2. Rebuild daemon on Pi: `cargo build --release`
3. Open `web/index.html` in browser — WebSocket target `ws://<pi-ip>:9001` (use IP, not `.local`, for browser from Windows host)
4. Change a pad color in web UI — verify pad LED updates on hardware
5. Change an encoder CC in config panel — verify `maschine.json` written; restart daemon, verify value persists
6. Connect an external MIDI source to `MIDI Control` ALSA port (`aconnect <source> <maschine-client>:1`) — send NoteOn for notes 0–15 — verify pad LEDs light
7. Check display — verify note names and encoder CC values render on OLED

**Verify block:** Web editor loads and connects, pad LED changes on color set, config survives daemon restart, MIDI IN drives pad LEDs, display shows values.

**Known open item:** Step 3 browser URL requires Pi IP address, not mDNS — confirm `192.168.2.123` is reachable from browser on Windows host. Tag steps `[low]` until Pi-verified.

---

## Section 2: `project-maschine-step-sequencer.md` — Parts 2 and 3

### Part 2 — 8 Pages, Per-Step Note/Velocity, Euclidean Fill `[draft]`

**Prerequisite:** Part 1 complete (basic sequencer working, pattern plays on channel 2).

**Steps:**

1. Enter sequencer mode (Shift+Pad Mode twice — same as Part 1)
2. Press Group A — active page switches to page 1 (lit Group button = active page)
3. Program a pattern on page 1; press Group B and program a different pattern — verify pages are independent (switching pages changes which steps are lit)
4. Press a step pad to select it — LED turns orange
5. Turn Encoder 1 — adjust that step's velocity (0–127); turn Encoder 2 — adjust note offset (0–127)
6. Start playback — verify pitch and volume vary per-step as set
7. Hold Shift + press Group A — page fills with 1-hit euclidean pattern (single step at position 0)
8. Hold Shift + press Group H — page fills with 8-hit euclidean pattern (every other step)
9. Verify pad LEDs update immediately to show pattern

**Verify block:** Page switching shows independent patterns, per-step note/vel audible on playback, euclidean fill updates pad LEDs.

### Part 3 — MIDI Clock Sync `[draft]`

**Prerequisite:** Parts 1 and 2 complete. Zynthian must have a MIDI clock output port available.

**Steps:**

1. Find ALSA port numbers:
   ```bash
   aconnect -l
   ```
   Locate `maschine.rs` client and its `MIDI Control` port (input). Locate Zynthian's clock output port. `[low]` — Zynthian clock output port name needs Pi verification.

2. Connect Zynthian clock to daemon:
   ```bash
   aconnect <zynthian-clock-port> <maschine-client>:1
   ```

3. Start Zynthian transport — verify sequencer starts at step 0 and steps advance in time with Zynthian tempo

4. Stop Zynthian transport — verify sequencer halts, step position preserved (pads stay lit at current step)

5. Disconnect clock (`aconnect -d <zynthian-clock-port> <maschine-client>:1`) while sequencer playing — verify sequencer continues on internal BPM within ~500ms

6. Open `web/index.html` — verify `clock_bpm` events appear in event stream showing correct BPM while external clock connected

**Verify block:** Steps lock to external clock, stop halts playback, fallback resumes on disconnect, BPM visible in web editor.

**Known open items:**
- Zynthian MIDI clock output port name: tag `[low]` until verified on Pi
- Internal fallback BPM source: daemon uses last estimated interval; document if BPM is configurable

---

## Section 3: MIDI Reference Updates

**File:** `~/zynth-docs/htmldoku/midi.md`

| Item | Change |
|------|--------|
| Encoder CC defaults | `config.rs` default = 16–23 (authoritative). README says 17–24 — README is wrong. Fix README too. Tag `[low]` until confirmed on Pi. |
| Encoder CC note | Add: "CC numbers configurable per-encoder via `maschine.json` or web editor" |
| MIDI IN port | Add new row: `MIDI Control` ALSA input port — NoteOn 0–15 → pad LEDs (velocity = brightness); Clock/Start/Stop → forwarded to `Pads MIDI` output |
| SMC-PAD channel | Change 7→6 everywhere (already tracked in `todo.md`) |
| SINCO double-routing | Add as Conflict 10: SINCO Private port mirrors all events from SINCO Master → TOGGLE_SEQ fires twice per press |

---

## Open Items (resolve during implementation)

| Item | How |
|------|-----|
| Encoder CC default mismatch (16–23 vs 17–24) | Source authoritative: 16–23. Also fix README.md in MaschineMK2_linux repo. |
| Pi IP reachable from browser | Confirm `http://192.168.2.123` reachable from Windows browser for web editor step |
| Zynthian MIDI clock output port name | `aconnect -l` on Pi, note exact port string |
| Per-step note range behavior | Does Encoder 2 wrap at 127? Needs Pi test |

---

## Testing Order

All new parts are `[draft]`. Test in this order on Pi:

1. Part 4 of MK2 controller tutorial (web editor, MIDI IN) — standalone, no sequencer dependency
2. Step sequencer Part 1 — basic 16-step (already drafted, first unverified)
3. Step sequencer Part 2 — 8 pages + note/vel + euclidean (depends on Part 1 verified)
4. Step sequencer Part 3 — MIDI clock (depends on Part 2 verified)

---

## Publish Checklist

After all parts verified:

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/ docs/zynthian-Doku/
git commit -m "docs: update Maschine MK2 tutorials — web editor, 8-page sequencer, MIDI clock sync"
git push
```

Move `inwork.md` entries to `done.md` only after all parts of each tutorial reach `[verified]`.
