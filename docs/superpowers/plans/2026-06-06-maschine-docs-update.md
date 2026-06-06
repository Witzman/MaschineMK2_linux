# Maschine MK2 Documentation Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update three Zynthian tutorial files and the MIDI Reference to reflect new MaschineMK2_linux functionality: standard CC output (encoders + buttons), web editor, config persistence, MIDI IN port, 8-page sequencer, per-step note/velocity, euclidean fill, and MIDI clock sync.

**Architecture:** Documentation-only changes across two git repos (`MaschineMK2_linux` for README fix; `zynth-docs` for tutorials + MIDI Reference). Every `htmldoku/*.md` edit requires running `generate-html.py` and committing the full `docs/zynthian-Doku/` output — never defer. All new tutorial parts tagged `[draft]` until Pi-verified.

**Tech Stack:** Markdown, Python (generate-html.py), git

---

## File Map

| File | Repo | Change |
|------|------|--------|
| `README.md` | `MaschineMK2_linux` | Fix encoder CC defaults 17–24 → 16–23 |
| `htmldoku/project-midi-reference.md` | `zynth-docs` | Update Maschine MK2 encoder/button table, add MIDI Control port, resolve Conflict 5, update Section 4 |
| `htmldoku/project-maschine-mk2.md` | `zynth-docs` | Update Driver Reference (RPN → CC), update Part 2 note, add Part 4 |
| `htmldoku/project-maschine-step-sequencer.md` | `zynth-docs` | Rewrite Part 2, rewrite Part 3 Step 2, add Part 4 (euclidean), add Part 5 (MIDI clock) |

---

## Task 1: Fix encoder CC defaults in MaschineMK2_linux README

**Files:**
- Modify: `README.md` in `MaschineMK2_linux` repo

The README says `"encoder_ccs": [17,18,19,20,21,22,23,24]` and "default 17–24" in the MIDI Mapping table. The source (`src/config.rs`) defaults to `[16, 17, 18, 19, 20, 21, 22, 23]` (CC 16–23). README is wrong.

- [ ] **Step 1: Find and replace encoder CC defaults in README.md**

In `/home/witzman/zynth/MaschineMK2_linux/README.md`:

Replace the Config Persistence example JSON:
```json
"encoder_ccs": [17,18,19,20,21,22,23,24]
```
with:
```json
"encoder_ccs": [16,17,18,19,20,21,22,23]
```

Replace the MIDI Mapping table row:
```
| Encoders 1–8 | CC | 17–24 | Absolute 0–127; CC numbers configurable per-encoder |
```
with:
```
| Encoders 1–8 | CC | 16–23 | Absolute 0–127; CC numbers configurable per-encoder via `maschine.json` or web editor |
```

- [ ] **Step 2: Commit to MaschineMK2_linux repo**

```bash
cd /home/witzman/zynth/MaschineMK2_linux
git add README.md
git commit -m "fix: correct encoder CC defaults to 16-23 (config.rs is authoritative)"
git push
```

Expected: `main` branch pushed to `git@github.com:Witzman/MaschineMK2_linux.git`.

---

## Task 2: Update MIDI Reference — Maschine MK2 table (RPN → CC)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-midi-reference.md`

The new daemon sends standard CC for both encoders and buttons. The RPN14/RPN7 references are obsolete.

- [ ] **Step 1: Replace the Maschine MK2 capability table in Section 1**

Find this block in `project-midi-reference.md` (lines ~16–22):

```
| 8 encoders | **RPN14** | 1 | RPN numbers 16–23, values 0–8191 |
| ~30 transport / function buttons | **RPN7** | 1 | RPN numbers 1–48 (see table below) |
| Group buttons A–H | *(none — internal state only)* | — | sets note base: A=24 B=36 C=48 D=60 E=72 F=84 G=96 H=108 |
```

Replace with:

```
| 8 encoders | **CC** | 1 | CC 16–23 (configurable via `maschine.json` or web editor), absolute 0–127 |
| ~30 transport / function buttons | **CC** | 1 | CC 1–14 (see table below), value 127 = press, 0 = release |
| Group buttons A–H — pad mode | *(none — internal state only)* | — | sets note base: A=24 B=36 C=48 D=60 E=72 F=84 G=96 H=108 |
| Group buttons A–H — sequencer mode | *(none — internal state only)* | — | switches active sequencer page (1–8) |
| `MIDI Control` ALSA input port | accepts NoteOn/Off 0–15, Clock, Start, Stop | — | NoteOn 0–15 → pad LED color/brightness; Clock/Start/Stop → forwarded to `Pads MIDI` output |
```

- [ ] **Step 2: Remove the obsolete Zynthian limitation note**

Find and delete this block (immediately after the table):

```
> **Zynthian limitation:** Encoders send RPN14, transport buttons send RPN7. Neither is standard CC 0–119. Zynthian CC Learn cannot capture them. MIDI filter rules remapping RPN → CC are required before any binding is possible.
```

- [ ] **Step 3: Update the transport/function button table header**

Find:
```
**Transport / function button RPN7 map (Ch 1):**
```

Replace with:
```
**Transport / function button CC map (Ch 1, value 127 = press, 0 = release):**
```

- [ ] **Step 4: Add MIDI IN port section after the OSC interface line**

After the OSC interface line (`**OSC interface:** daemon listens on...`), add:

```markdown
**MIDI IN (`MIDI Control` ALSA input port):** Connect any MIDI source to drive pad LEDs and sync the sequencer. NoteOn note 0 = bottom-left pad (offset 0), note 15 = top-right pad (offset 15). Velocity maps to LED brightness. Clock/Start/Stop messages are forwarded to `Pads MIDI` output and can lock the built-in step sequencer. Connect with `aconnect <source>:<port> <maschine-client>:1`.
```

- [ ] **Step 5: Save, run generator, commit HTML**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-midi-reference.md docs/zynthian-Doku/
git commit -m "docs: update Maschine MK2 MIDI table — encoders/buttons now standard CC, add MIDI Control port"
```

---

## Task 3: Update MIDI Reference — assignment matrix + Conflict 5 + Section 4

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-midi-reference.md`

- [ ] **Step 1: Update assignment matrix rows for encoders and buttons**

Find these two rows in the assignment matrix (Section 2):

```
| Maschine MK2 | 8 Encoders | 1 | RPN14 16–23 | **unmapped** | Maschine MK2 P2 | `[blocked]` |
| Maschine MK2 | Transport buttons | 1 | RPN7 1–48 | **unmapped** | Maschine MK2 P2 | `[blocked]` |
```

Replace with:

```
| Maschine MK2 | 8 Encoders | 1 | CC 16–23 (configurable) | **unassigned** | Maschine MK2 P2 | `[draft]` |
| Maschine MK2 | Transport buttons | 1 | CC 1–14 (127/0) | **unassigned** | Maschine MK2 P2 | `[draft]` |
| Maschine MK2 | MIDI Control IN → pad LEDs | — | NoteOn 0–15 | pad LED color/brightness | Maschine MK2 P4 | `[draft]` |
```

- [ ] **Step 2: Replace Conflict 5 — now resolved**

Find the Conflict 5 block:

```
### Conflict 5 — Maschine RPN14/RPN7 invisible to CC Learn

Encoders (RPN14 16–23) and transport buttons (RPN7 1–48) cannot be captured by CC Learn. Blocks all Maschine MK2 Part 2 work.

**Resolution:** Design MIDI filter rules (`ZYNTHIAN_MIDI_FILTER_RULES`) to remap selected RPNs to CC numbers before the signal reaches chains. Example: `RPN 16 CH1 => CC 20 CH1`. Requires Maschine MK2 P2 redesign.
```

Replace with:

```
### Conflict 5 — ~~Maschine RPN14/RPN7 invisible to CC Learn~~ RESOLVED (2026-06-06)

Encoders now send standard CC 16–23 (configurable). Transport buttons now send CC 1–14 (value 127 press, 0 release). Both are standard CC 0–119 — Zynthian CC Learn can capture them. Maschine MK2 Part 2 redesign is now unblocked.

**Previous issue:** Encoders sent RPN14, buttons sent RPN7 — neither capturable by CC Learn. MIDI filter rules were planned as a workaround. No longer needed.
```

- [ ] **Step 3: Remove RPN row from Section 4 MIDI Feature Map**

Find and delete this row in the Section 4 table:

```
| RPN14 / RPN7 (Maschine encoders/buttons) | — | — | **not natively supported** — needs MIDI filter RPN→CC rule |
```

- [ ] **Step 4: Update Going Further — remove obsolete RPN filter task**

Find:
```
- Design MIDI filter rules for Maschine RPN→CC remapping, enabling Maschine MK2 P2
```

Replace with:
```
- Map Maschine encoder CCs to Zynthian synth parameters via CC Learn (Maschine MK2 Part 2)
```

- [ ] **Step 5: Save, run generator, commit HTML**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-midi-reference.md docs/zynthian-Doku/
git commit -m "docs: resolve Conflict 5 — Maschine encoders/buttons now standard CC; update assignment matrix"
```

---

## Task 4: Update project-maschine-mk2.md Driver Reference (RPN → CC)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-mk2.md`

- [ ] **Step 1: Update the encoders section in Driver Reference**

Find:
```
### 8 Encoders

Send **RPN14** (14-bit, 4-message CC sequence) on Ch1, mapped to RPN numbers 16–23. Values range 0–8191. **Standard MIDI CC Learn cannot capture these.**
```

Replace with:
```
### 8 Encoders

Send standard **CC** on Ch1. Default CC numbers: 16–23 (Encoder 1 = CC 16, Encoder 8 = CC 23). Values 0–127. CC Learn can capture these.

CC numbers are configurable per-encoder via `maschine.json` or the web editor (see Part 4).
```

- [ ] **Step 2: Update transport buttons section in Driver Reference**

Find:
```
### Transport and Function Buttons

Send **RPN7** (3-message CC sequence) on Ch1. **Standard MIDI CC Learn cannot capture these.**
```

Replace with:
```
### Transport and Function Buttons

Send standard **CC** on Ch1. Value 127 = button pressed, 0 = button released. CC Learn can capture these.
```

- [ ] **Step 3: Update the button CC map table header**

Find the table immediately after "Send standard **CC** on Ch1...":
```
| Button | RPN number |
|--------|-----------|
```

Replace with:
```
| Button | CC number |
|--------|-----------|
```

- [ ] **Step 4: Update the Step Sequencer section in Driver Reference**

Find:
```
### Step Sequencer (Pad Mode 2)

Activated by Shift + Pad Mode a second time. Pads toggle steps on/off instead of playing notes. Play/Stop control playback. Speed set by Shift + encoder B6.
```

Replace with:
```
### Step Sequencer (Pad Mode 2)

Activated by Shift + Pad Mode a second time. Pads toggle steps on/off instead of playing notes. Play starts playback; Erase stops it. Speed set by Shift + encoder B6 [low].

**Group buttons A–H** switch between 8 independent 16-step pages in sequencer mode (not note base — that is pad mode behaviour).

**Per-step editing:** tap a step to select it (orange LED), then Encoder 1 = velocity (0–127), Encoder 2 = note offset (0–127).

**Euclidean fill:** Shift + Group A–H fills the current page with 1–8 evenly distributed hits.

**MIDI clock sync:** external MIDI clock received on the `MIDI Control` input port locks the step rate (6 ticks = one 16th-note step). Fallback to internal BPM after 500 ms of clock silence.
```

- [ ] **Step 5: Update Part 2 note — CC Learn no longer blocked**

Find:
```
> **Note:** Zynthian's CC Learn does not work with the Maschine MK2 daemon. The encoders send RPN14 (14-bit RPN, not standard CC) and the transport buttons send RPN7 (also not standard CC). Zynthian CC Learn only captures standard CC 0–119. This part needs to be redesigned around MIDI filter rules that remap RPN → CC before reaching Zynthian's routing layer.

This part is pending redesign. See the Driver Reference section below for the full MIDI output spec.
```

Replace with:
```
> **Update (2026-06-06):** The daemon now sends standard CC for both encoders (CC 16–23) and transport buttons (CC 1–14). Zynthian CC Learn can capture these. This part is ready to be designed and tested. See the Driver Reference below for CC numbers.

This part is ready for implementation. Encoders send CC 16–23 (configurable); transport buttons send CC 1–14 (127 = press, 0 = release).
```

- [ ] **Step 6: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-mk2.md docs/zynthian-Doku/
git commit -m "docs: update Maschine MK2 Driver Reference — encoders/buttons now standard CC"
```

---

## Task 5: Add Part 4 to project-maschine-mk2.md (web editor, config, MIDI IN, display)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-mk2.md`

- [ ] **Step 1: Insert Part 4 before the Driver Reference section**

Find the line:
```
## Driver Reference
```

Insert the following block immediately before it:

````markdown
## Part 4 — Web Editor, Config Persistence, and MIDI IN `[draft]`

The daemon includes a browser-based LED and config editor, a JSON config file that persists settings across restarts, and an ALSA MIDI input port (`MIDI Control`) that accepts NoteOn messages to drive pad LEDs from any MIDI source.

### Step 1 — Sync and rebuild the daemon

From the machine where the source lives (not the Pi):

```bash
rsync -av --exclude='target/' ~/zynth/MaschineMK2_linux/ root@192.168.2.123:~/zynth/MaschineMK2_linux/
```

Then on the Pi:

```bash
ssh root@192.168.2.123
cd ~/zynth/MaschineMK2_linux
source "$HOME/.cargo/env"
cargo build --release 2>&1 | tail -3
```

Expected last line: `Finished 'release' profile [optimized] target(s) in ...`

**Verify:**
```bash
ls -lh ~/zynth/MaschineMK2_linux/target/release/maschine
```
File exists. Size is typically 600K–1M.

### Step 2 — Restart the daemon service

```bash
systemctl restart maschine-mk2.service
systemctl status maschine-mk2.service --no-pager
```

Expected: `Active: active (running)`

**Verify:** Service is running after restart.

### Step 3 — Open the web editor

Open a browser and navigate to:

```
http://192.168.2.123:9001
```

Wait up to 5 seconds for the WebSocket connection to establish.

> Use the Pi's IP address (`192.168.2.123`), not `zynthian.local` — mDNS does not resolve from a browser on Windows hosts.

**Verify:** The web editor loads and shows a pad grid or connection status.

[low] Exact web UI layout and control labels need Pi verification.

### Step 4 — Change a pad LED color

In the web editor, click any pad in the grid. Select a color.

**Verify:** The corresponding pad on the Maschine MK2 hardware lights in the selected color within 1–2 seconds.

### Step 5 — Verify config persistence

In the web editor config panel, change the CC number for Encoder 1 from 16 to a different value (e.g. 20).

Then restart the daemon:

```bash
systemctl restart maschine-mk2.service
```

Check the saved config:

```bash
cat ~/zynth/MaschineMK2_linux/maschine.json
```

Expected: `encoder_ccs` shows `20` in the first position.

**Verify:** Encoder CC change survives a daemon restart.

### Step 6 — Drive pad LEDs from MIDI IN

Connect any MIDI source to the `MIDI Control` ALSA port:

```bash
aconnect -l | grep -A3 maschine
```

Note the client number (e.g. `28`) and port `1` (MIDI Control). Connect a source:

```bash
aconnect <source-client>:<port> 28:1
```

Replace `28` with the actual maschine.rs client number. For a quick test, connect the Xboard:

```bash
aconnect <xboard-client>:0 28:1
```

Press keys in the range MIDI notes 0–15 on the connected source (on a standard keyboard, these are very low notes — C-1 to D#0).

**Verify:** Pad LEDs on the Maschine light in response to incoming NoteOn messages. Velocity controls brightness.

[low] Exact `aconnect` client numbers and best test procedure need Pi verification.

### Step 7 — Check the display

Look at the Maschine MK2's 128×64 OLED display while the daemon is running.

**Verify:** The display shows note names or encoder CC values.

[low] Display content and layout need Pi verification.

---

**Verify (Part 4 complete):** Web editor loads and connects, pad LED changes on color set, `maschine.json` shows updated CC after restart, NoteOn to MIDI Control port lights corresponding pads.

---

````

- [ ] **Step 2: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-mk2.md docs/zynthian-Doku/
git commit -m "docs: add Maschine MK2 Part 4 — web editor, config persistence, MIDI IN port"
```

---

## Task 6: Rewrite Part 2 of project-maschine-step-sequencer.md

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-step-sequencer.md`

The old Part 2 "Melodic Pattern" says Group buttons set note base in sequencer mode — that is now wrong. In sequencer mode Group A–H switch pages. Note base must be set in pad mode before entering the sequencer. Per-step note editing now uses Encoder 2 (not Shift+pad).

- [ ] **Step 1: Replace Part 2 entirely**

Find the entire Part 2 block from:
```
## Part 2 — Melodic Pattern `[draft]`
```
up to (but not including):
```
## Part 3 — Tempo and Snapshot `[draft]`
```

Replace with:

````markdown
## Part 2 — 8 Pages and Per-Step Note/Velocity `[draft]`

The sequencer has 8 independent 16-step pages. In sequencer mode, **Group A–H switch pages** — each button corresponds to one page. The lit Group button shows the active page.

Per-step note and velocity editing: tap any active step (its LED turns **orange** — selected), then turn Encoder 1 to adjust velocity (0–127) or Encoder 2 to adjust note offset (0–127).

Set the note base **before** entering sequencer mode: in pad mode, Group A–H set the note base (A = C1 through H = C8). Once you enter sequencer mode, those same buttons switch pages instead.

### Step 1 — Set note base in pad mode

Before entering sequencer mode, press **Group D** on the Maschine MK2 (while still in normal pad mode).

Group D sets the note base to **60 (C4)** — all steps with no note offset play C4.

**Verify:** Press a pad in pad mode — it plays C4 (requires a chain on MIDI ch 1 with a synth loaded).

### Step 2 — Enter sequencer mode

1. Hold **Shift**, press **Pad Mode** — enters pad mode 1
2. Hold **Shift**, press **Pad Mode** a second time — enters sequencer mode

**Verify:** Pressing a pad toggles a step (no note plays; LED brightens or dims).

### Step 3 — Navigate pages with Group buttons

In sequencer mode, pressing **Group A** switches to page 1, **Group B** to page 2, and so on. The lit Group button shows the active page.

1. Press **Group A** — page 1 is active. Program a short pattern: tap 3–4 pads to activate steps.
2. Press **Group B** — page 2 is active. Tap different pads to program a different pattern.
3. Press **Group A** again — the original page 1 pattern is still there.

**Verify:** Switching pages changes which step pads are lit. Each page holds its own independent pattern.

### Step 4 — Select a step for editing

Tap any lit (active) step pad. Its LED turns **orange** — this step is selected for per-step editing.

**Verify:** One pad glows orange while other active pads stay at normal brightness.

### Step 5 — Adjust velocity

With a step selected (orange LED), turn **Encoder 1** clockwise to raise velocity, counterclockwise to lower it.

Range: 0 (silent) to 127 (full velocity).

Press **Play**, then press **Erase** to stop. Listen for volume variation across steps.

**Verify:** The selected step plays at a noticeably different volume from unedited steps.

### Step 6 — Adjust note offset

With a step selected (orange LED), turn **Encoder 2** clockwise to raise the note offset, counterclockwise to lower it.

The note offset is added to the current note base. With Group D set before entering sequencer mode (base = 60 = C4):
- Offset 0 → C4
- Offset 7 → G4
- Offset 12 → C5
- Offset −12 → C3

Press **Play** and listen for pitch variation across steps.

**Verify:** The selected step plays a different pitch from unedited steps.

### Step 7 — Build a phrase

Program a 4–8 step pattern. Select each step and assign different note offsets and velocities to create a melody.

**Verify:** Playback produces distinct pitches and volumes per step as programmed.

---

**Verify (Part 2 complete):** Pages are independent, Group buttons switch pages in sequencer mode, per-step velocity and note offset are audible on playback.

````

- [ ] **Step 2: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-step-sequencer.md docs/zynthian-Doku/
git commit -m "docs: rewrite step sequencer Part 2 — Group buttons switch pages, per-step note/vel via encoders"
```

---

## Task 7: Fix Part 3 of project-maschine-step-sequencer.md (remove wrong Group transpose step)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-step-sequencer.md`

Part 3 Step 2 says "Press Group A–H to transpose" while the sequencer is running. This is wrong — in sequencer mode, Group A–H switch pages.

- [ ] **Step 1: Replace Step 2 in Part 3**

Find in Part 3:

```
### Step 2 — Transpose with Group buttons

Press any Group button (A–H) while the sequencer is running to shift the note base of all steps:

| Button | Note base | Register |
|--------|-----------|----------|
| A | C1 | sub-bass |
| B | C2 | bass |
| C | C3 (default) | mid-low |
| D | C4 (middle C) | mid |
| E | C5 | mid-high |
| F | C6 | high |
| G | C7 | very high |
| H | C8 | extreme high |

[low] Whether Group transpose applies immediately to the playing step or only from the next step needs Pi verification.

**Verify:** Group button press shifts the pitch of the entire pattern up or down.
```

Replace with:

```markdown
### Step 2 — Change note base between patterns

Group buttons A–H set the note base **in pad mode only**. In sequencer mode they switch pages. To shift the entire pattern's pitch range, exit sequencer mode first.

To change note base and return:

1. Press **Erase** to stop playback
2. Hold **Shift**, press **Pad Mode** — returns to pad mode 1
3. Hold **Shift**, press **Pad Mode** again — returns to normal pad mode
4. Press a **Group** button to set note base (A = C1 … H = C8)
5. Re-enter sequencer mode: Shift + Pad Mode twice
6. Resume playback

[low] Exact steps to return from sequencer mode to pad mode and back need Pi verification.

**Verify:** After setting a different Group button and re-entering sequencer mode, steps play at the new note base.
```

- [ ] **Step 2: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-step-sequencer.md docs/zynthian-Doku/
git commit -m "docs: fix step sequencer Part 3 — Group buttons switch pages in seq mode, not transpose"
```

---

## Task 8: Add Part 4 to project-maschine-step-sequencer.md (euclidean fill)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-step-sequencer.md`

- [ ] **Step 1: Insert Part 4 before the "Going Further" section**

Find:
```
## Going Further
```

Insert the following block immediately before it:

````markdown
## Part 4 — Euclidean Fill `[draft]`

Hold **Shift** and press any Group button (A–H) to fill the current sequencer page with an evenly-distributed rhythm. Group A = 1 hit, Group H = 8 hits. The hits are distributed using the Bresenham (Euclidean) algorithm — the first hit always falls on step 0.

| Shift + Group | Hits | Approximate pattern across 16 steps |
|---|---|---|
| A | 1 | step 0 only |
| B | 2 | steps 0, 8 |
| C | 3 | steps 0, 5, 10 |
| D | 4 | steps 0, 4, 8, 12 |
| E | 5 | steps 0, 3, 6, 9, 12 |
| F | 6 | steps 0, 2, 5, 7, 10, 13 |
| G | 7 | steps 0, 2, 4, 6, 8, 10, 12 |
| H | 8 | steps 0, 2, 4, 6, 8, 10, 12, 14 |

[low] Exact step positions for each density need Pi verification — positions shown are approximate.

### Step 1 — Switch to an empty page

In sequencer mode, press **Group B** to switch to page 2. Confirm no steps are lit (empty page).

If page 2 has steps, tap each lit pad to deactivate it before proceeding.

**Verify:** No step pads are lit on page 2.

### Step 2 — Apply a 4-hit euclidean fill

Hold **Shift** and press **Group D** (4 hits).

**Verify:** 4 step pads light up, evenly spaced across the 16 positions (steps 0, 4, 8, 12).

### Step 3 — Start playback

Press **Play**.

**Verify:** 4 evenly-spaced notes play in a repeating loop — a basic four-on-the-floor rhythm at the current step rate.

### Step 4 — Try different densities

Hold **Shift** + press **Group H** — fills page with 8 hits.

Hold **Shift** + press **Group A** — fills page with 1 hit (step 0 only).

**Verify:** Pad LEDs update immediately to show each new pattern. Playback changes density without stopping.

---

**Verify (Part 4 complete):** Shift+Group fills page with correct number of evenly-spaced hits; LED display matches; plays correctly; switching densities mid-playback works.

````

- [ ] **Step 2: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-step-sequencer.md docs/zynthian-Doku/
git commit -m "docs: add step sequencer Part 4 — euclidean fill via Shift+Group"
```

---

## Task 9: Add Part 5 to project-maschine-step-sequencer.md (MIDI clock sync)

**Files:**
- Modify: `~/zynth-docs/htmldoku/project-maschine-step-sequencer.md`

- [ ] **Step 1: Insert Part 5 before the "Going Further" section**

Find:
```
## Going Further
```

Insert the following block immediately before it:

````markdown
## Part 5 — MIDI Clock Sync `[draft]`

Connect any MIDI clock source (Zynthian, a DAW, or a hardware sequencer) to the daemon's `MIDI Control` ALSA input port. The sequencer locks to the external clock: 24 pulses per quarter note (ppqn), 6 ticks = one 16th-note step. If no clock tick arrives for 500 ms, the sequencer falls back to its internal BPM timer automatically.

**Prerequisites:** Parts 1 and 2 complete. A MIDI clock source available — Zynthian transport or a DAW connected to the same machine.

### Step 1 — Find port numbers

```bash
ssh root@192.168.2.123
aconnect -l | grep -A3 maschine
```

Look for:
```
client 28: 'maschine.rs' [...]
    0 'Pads MIDI   '
    1 'MIDI Control'
```

Note the client number (e.g. `28`). The `MIDI Control` port is `28:1`.

[low] Client number varies. Verify on Pi.

### Step 2 — Find the Zynthian MIDI clock output

[low] Zynthian MIDI clock output port name needs Pi verification. Run:

```bash
aconnect -l
```

Look for a Zynthian or JACK MIDI output port that sends MIDI clock. Common candidates: `ZynMidiRouter` or a port bridged via `a2jmidid`.

### Step 3 — Connect the clock source

```bash
aconnect <clock-source-client>:<port> 28:1
```

Replace `<clock-source-client>:<port>` with the clock output found in Step 2, and `28` with the actual maschine.rs client number.

Verify the connection:
```bash
aconnect -l | grep -A6 maschine
```

The clock source should appear listed as a sender to the maschine.rs client.

**Verify:** Connection is visible in `aconnect -l`.

### Step 4 — Start the clock and verify sync

Start transport in Zynthian (or the clock source). Press **Play** on the Maschine MK2.

**Verify:** Steps advance in sync with the Zynthian tempo. Changing tempo in Zynthian changes the step rate on the Maschine within 1–2 steps.

### Step 5 — Stop and verify position hold

Stop transport in Zynthian.

**Verify:** The sequencer halts. The last active step pad stays lit — position is not reset to step 0 on stop.

### Step 6 — Test fallback to internal clock

With the clock connected and sequencer running, disconnect:

```bash
aconnect -d <clock-source-client>:<port> 28:1
```

Wait 2–3 seconds.

**Verify:** The sequencer does not stall or freeze. After approximately 500 ms, it continues stepping at the last estimated BPM on the internal timer.

### Step 7 — Verify BPM in web editor (optional)

While the external clock is connected, open `http://192.168.2.123:9001` in a browser.

Watch the live event stream. Events of the form `{"type":"clock_bpm","bpm":120.x}` should appear while the clock is running.

**Verify:** `clock_bpm` events appear. BPM value matches the clock source's tempo.

---

**Verify (Part 5 complete):** Sequencer locks to external clock, stops on transport stop, falls back on clock silence, BPM visible in web editor event stream.

````

- [ ] **Step 2: Update "Going Further" section at bottom of file**

Find the "Going Further" section and add two new bullets:

After the existing bullets, add:
```
- Use MIDI clock sync (Part 5) with Zynthian's transport to keep the Maschine sequencer in tempo without manual BPM matching
- Combine euclidean patterns on multiple pages for polyrhythmic structures — page 1: 4 hits, page 2: 3 hits, alternate playback
```

- [ ] **Step 3: Run generator and commit**

```bash
cd ~/zynth-docs
python3 htmldoku/generate-html.py
git add htmldoku/project-maschine-step-sequencer.md docs/zynthian-Doku/
git commit -m "docs: add step sequencer Part 5 — MIDI clock sync via MIDI Control port"
```

---

## Task 10: Update inwork.md and todo.md

**Files:**
- Modify: `~/zynth-docs/MD/inwork.md`
- Modify: `~/zynth-docs/MD/todo.md`

- [ ] **Step 1: Update inwork.md — Maschine MK2 Step Sequencer status**

The step sequencer tutorial now has Parts 1–5, all `[draft]`. Update the entry description to reflect the full scope:

Find:
```
- [~] **Maschine MK2 Step Sequencer** — 16-step sequencer via MaschineMK2_linux daemon; sequencer fires on Ch2; melodic note assignment; no NI software required; prereq: Maschine MK2 Controller tutorial
```

Replace with:
```
- [~] **Maschine MK2 Step Sequencer** — 8-page 16-step sequencer; per-step note/velocity via encoders; euclidean fill; MIDI clock sync; no NI software required; prereq: Maschine MK2 Controller tutorial
```

- [ ] **Step 2: Update inwork.md — Maschine MK2 Controller entry**

Find the Maschine MK2 Controller entry (should show Parts 1+3 verified):
```
- [~] **Maschine MK2 Controller** — Parts 1+3 verified; Part 2 (CC Learn) still draft
```

Replace with:
```
- [~] **Maschine MK2 Controller** — Parts 1+3 verified; Part 2 (CC Learn, now unblocked — encoders send standard CC); Part 4 (web editor, MIDI IN, display) draft
```

- [ ] **Step 3: Update todo.md — mark MIDI Reference updates done**

In the `Debug and fix TOGGLE_SEQ` section, the sub-item about updating the MIDI Reference was already completed in previous sessions. Verify by checking `project-midi-reference.md` — SMC-PAD ch 6 and Conflict 10 are already present. Add a note or mark the sub-items done.

Find:
```
  **Update MIDI Reference page:**
  - SMC-PAD channel: change 7 → 6 everywhere
  - Master channel: change 7 → 6 everywhere
  - SINCO Private port double-routing: document as Conflict 10
```

Replace with:
```
  **Update MIDI Reference page:**
  - [x] SMC-PAD channel: change 7 → 6 everywhere — already done in current reference
  - [x] Master channel: change 7 → 6 everywhere — already done in current reference
  - [x] SINCO Private port double-routing: document as Conflict 10 — already present
  - [x] Maschine encoder/button MIDI type: updated RPN → CC (2026-06-06)
  - [x] Conflict 5 resolved (2026-06-06)
```

- [ ] **Step 4: Add new todo item for Pi testing of new parts**

Add under Active:
```
- [ ] **Test Maschine MK2 Part 4 on Pi (web editor, MIDI IN)**
  - [ ] Confirm web editor loads at http://192.168.2.123:9001
  - [ ] Confirm pad LED changes on color set
  - [ ] Confirm maschine.json persists after restart
  - [ ] Confirm MIDI Control IN drives pad LEDs

- [ ] **Test step sequencer Part 2 on Pi (pages, per-step note/vel)**
  - [ ] Confirm Group A–H switch pages in sequencer mode
  - [ ] Confirm step selection (orange LED)
  - [ ] Confirm Encoder 1 = velocity, Encoder 2 = note offset
  - [ ] Blocked: Part 1 must pass first

- [ ] **Test step sequencer Part 4 on Pi (euclidean fill)**
  - [ ] Confirm Shift+Group D = 4 evenly-spaced hits
  - [ ] Verify exact step positions match table in tutorial
  - [ ] Blocked: Part 2 must pass first

- [ ] **Test step sequencer Part 5 on Pi (MIDI clock sync)**
  - [ ] Identify Zynthian MIDI clock output port
  - [ ] Confirm sequencer locks to external clock
  - [ ] Confirm fallback on clock silence
  - [ ] Blocked: Part 2 must pass first
```

- [ ] **Step 5: Commit tracking file updates to zynth-docs**

```bash
cd ~/zynth-docs
git add MD/inwork.md MD/todo.md
git commit -m "docs: update inwork.md + todo.md — Maschine MK2 tutorial scope and new test items"
git push
```

---

## Self-Review Checklist

- [x] Task 1 covers README encoder CC fix (source authoritative: 16–23)
- [x] Tasks 2–3 cover all MIDI Reference updates: encoder/button table, MIDI Control port, assignment matrix, Conflict 5 resolved, Section 4 RPN row removed
- [x] Task 4 covers Driver Reference RPN → CC + Part 2 unblocking
- [x] Task 5 covers Part 4 (web editor, config, MIDI IN, display)
- [x] Task 6 covers Part 2 full rewrite (Group → pages, Encoder 1/2 for vel/note)
- [x] Task 7 covers Part 3 fix (remove wrong Group transpose step)
- [x] Task 8 covers Part 4 euclidean fill
- [x] Task 9 covers Part 5 MIDI clock sync
- [x] Task 10 covers inwork.md + todo.md tracking updates
- [x] Every htmldoku edit task includes `generate-html.py` + commit step
- [x] All new tutorial parts tagged `[draft]`
- [x] `[low]` tags on every step that needs Pi verification before marking verified
- [x] No TBD or placeholder steps — all `[low]` items name exactly what to verify
- [x] Part 5 prereq correctly states "Parts 1 and 2" (fixed from spec)
- [x] Encoder CC defaults consistent throughout: 16–23 everywhere
