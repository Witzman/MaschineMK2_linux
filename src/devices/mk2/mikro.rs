//  maschine.rs: user-space drivers for native instruments USB HIDs
//  Copyright (C) 2015 William Light <wrl@illest.net>
//
//  This program is free software: you can redistribute it and/or modify
//  it under the terms of the GNU Lesser General Public License as
//  published by the Free Software Foundation, either version 3 of the
//  License, or (at your option) any later version.
//
//  This program is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU Lesser General Public License for more details.
//
//  You should have received a copy of the GNU Lesser General Public
//  License along with this program.  If not, see
//  <http://www.gnu.org/licenses/>.

use std::fs::File;
use std::mem::transmute;
use std::os::unix::io;

extern crate nix;
use midi::{Channel::Ch2, Message, U7};
use nix::unistd;

extern crate hex;
extern crate png;

use crate::base::{Maschine, MaschineButton, MaschineHandler, MaschinePad, MaschinePadStateTransition};
use crate::display;
use crate::clock::{ClockSource, ClockState};


const BUTTON_REPORT_TO_MIKROBUTTONS_MAP: [[Option<MaschineButton>; 8]; 24] = [
    [
        Some(MaschineButton::F8),
        Some(MaschineButton::F7),
        Some(MaschineButton::F6),
        Some(MaschineButton::F5),
        Some(MaschineButton::F4),
        Some(MaschineButton::F3),
        Some(MaschineButton::F2),
        Some(MaschineButton::F1),
    ],
    [
        Some(MaschineButton::Auto),
        Some(MaschineButton::All),
        Some(MaschineButton::Pageleft),
        Some(MaschineButton::Pageright),
        Some(MaschineButton::Sampling),
        Some(MaschineButton::Browse),
        Some(MaschineButton::Step),
        Some(MaschineButton::Control),
    ],
    [
        Some(MaschineButton::Nav),
        Some(MaschineButton::Noterepeat),
        Some(MaschineButton::Enter),
        Some(MaschineButton::Navright),
        Some(MaschineButton::Navleft),
        Some(MaschineButton::Tempo),
        Some(MaschineButton::Swing),
        Some(MaschineButton::Volume),
    ],
    [
        Some(MaschineButton::GroupH),
        Some(MaschineButton::GroupG),
        Some(MaschineButton::GroupF),
        Some(MaschineButton::GroupE),
        Some(MaschineButton::GroupD),
        Some(MaschineButton::GroupC),
        Some(MaschineButton::GroupB),
        Some(MaschineButton::GroupA),
    ],
    [
        Some(MaschineButton::Shift),
        Some(MaschineButton::Erase),
        Some(MaschineButton::Rec),
        Some(MaschineButton::Play),
        Some(MaschineButton::Grid),
        Some(MaschineButton::Stepright),
        Some(MaschineButton::Stepleft),
        Some(MaschineButton::Restart),
    ],
    [
        Some(MaschineButton::Mute),
        Some(MaschineButton::Solo),
        Some(MaschineButton::Select),
        Some(MaschineButton::Duplicate),
        Some(MaschineButton::Navigate),
        Some(MaschineButton::Padmode),
        Some(MaschineButton::Pattern),
        Some(MaschineButton::Scene),
    ],
    [
        Some(MaschineButton::R1),
        Some(MaschineButton::R2),
        Some(MaschineButton::R3),
        Some(MaschineButton::R4),
        Some(MaschineButton::R5),
        Some(MaschineButton::R6),
        Some(MaschineButton::R7),
        Some(MaschineButton::R8),
    ],
    [
        Some(MaschineButton::A1),
        Some(MaschineButton::A2),
        Some(MaschineButton::A3),
        Some(MaschineButton::A4),
        Some(MaschineButton::A5),
        Some(MaschineButton::A6),
        Some(MaschineButton::A7),
        Some(MaschineButton::A8),
    ],
    [
        Some(MaschineButton::B1),
        Some(MaschineButton::B2),
        Some(MaschineButton::B3),
        Some(MaschineButton::B4),
        Some(MaschineButton::B5),
        Some(MaschineButton::B6),
        Some(MaschineButton::B7),
        Some(MaschineButton::B8),
    ],
    [
        Some(MaschineButton::C1),
        Some(MaschineButton::C2),
        Some(MaschineButton::C3),
        Some(MaschineButton::C4),
        Some(MaschineButton::C5),
        Some(MaschineButton::C6),
        Some(MaschineButton::C7),
        Some(MaschineButton::C8),
    ],
    [
        Some(MaschineButton::D1),
        Some(MaschineButton::D2),
        Some(MaschineButton::D3),
        Some(MaschineButton::D4),
        Some(MaschineButton::D5),
        Some(MaschineButton::D6),
        Some(MaschineButton::D7),
        Some(MaschineButton::D8),
    ],
    [
        Some(MaschineButton::E1),
        Some(MaschineButton::E2),
        Some(MaschineButton::E3),
        Some(MaschineButton::E4),
        Some(MaschineButton::E5),
        Some(MaschineButton::E6),
        Some(MaschineButton::E7),
        Some(MaschineButton::E8),
    ],
    [
        Some(MaschineButton::FF1),
        Some(MaschineButton::FF2),
        Some(MaschineButton::FF3),
        Some(MaschineButton::FF4),
        Some(MaschineButton::FF5),
        Some(MaschineButton::FF6),
        Some(MaschineButton::FF7),
        Some(MaschineButton::FF8),
    ],
    [
        Some(MaschineButton::G1),
        Some(MaschineButton::G2),
        Some(MaschineButton::G3),
        Some(MaschineButton::G4),
        Some(MaschineButton::G5),
        Some(MaschineButton::G6),
        Some(MaschineButton::G7),
        Some(MaschineButton::G8),
    ],
    [
        Some(MaschineButton::H1),
        Some(MaschineButton::H2),
        Some(MaschineButton::H3),
        Some(MaschineButton::H4),
        Some(MaschineButton::H5),
        Some(MaschineButton::H6),
        Some(MaschineButton::H7),
        Some(MaschineButton::H8),
    ],
    [
        Some(MaschineButton::I1),
        Some(MaschineButton::I2),
        Some(MaschineButton::I3),
        Some(MaschineButton::I4),
        Some(MaschineButton::I5),
        Some(MaschineButton::I6),
        Some(MaschineButton::I7),
        Some(MaschineButton::I8),
    ],
    [
        Some(MaschineButton::J1),
        Some(MaschineButton::J2),
        Some(MaschineButton::J3),
        Some(MaschineButton::J4),
        Some(MaschineButton::J5),
        Some(MaschineButton::J6),
        Some(MaschineButton::J7),
        Some(MaschineButton::J8),
    ],
    [
        Some(MaschineButton::K1),
        Some(MaschineButton::K2),
        Some(MaschineButton::K3),
        Some(MaschineButton::K4),
        Some(MaschineButton::K5),
        Some(MaschineButton::K6),
        Some(MaschineButton::K7),
        Some(MaschineButton::K8),
    ],
    [
        Some(MaschineButton::L1),
        Some(MaschineButton::L2),
        Some(MaschineButton::L3),
        Some(MaschineButton::L4),
        Some(MaschineButton::L5),
        Some(MaschineButton::L6),
        Some(MaschineButton::L7),
        Some(MaschineButton::L8),
    ],
    [
        Some(MaschineButton::M1),
        Some(MaschineButton::M2),
        Some(MaschineButton::M3),
        Some(MaschineButton::M4),
        Some(MaschineButton::M5),
        Some(MaschineButton::M6),
        Some(MaschineButton::M7),
        Some(MaschineButton::M8),
    ],
    [
        Some(MaschineButton::N1),
        Some(MaschineButton::N2),
        Some(MaschineButton::N3),
        Some(MaschineButton::N4),
        Some(MaschineButton::N5),
        Some(MaschineButton::N6),
        Some(MaschineButton::N7),
        Some(MaschineButton::N8),
    ],
    [
        Some(MaschineButton::O1),
        Some(MaschineButton::O2),
        Some(MaschineButton::O3),
        Some(MaschineButton::O4),
        Some(MaschineButton::O5),
        Some(MaschineButton::O6),
        Some(MaschineButton::O7),
        Some(MaschineButton::O8),
    ],
    [
        Some(MaschineButton::P1),
        Some(MaschineButton::P2),
        Some(MaschineButton::P3),
        Some(MaschineButton::P4),
        Some(MaschineButton::P5),
        Some(MaschineButton::P6),
        Some(MaschineButton::P7),
        Some(MaschineButton::P8),
    ],
    [
        Some(MaschineButton::Q1),
        Some(MaschineButton::Q2),
        Some(MaschineButton::Q3),
        Some(MaschineButton::Q4),
        Some(MaschineButton::Q5),
        Some(MaschineButton::Q6),
        Some(MaschineButton::Q7),
        Some(MaschineButton::Q8),
    ],
];

#[allow(dead_code)]
struct ButtonReport {
    pub buttons: u32,
    pub encoder: u8,
}

pub struct Mikro {
    dev: io::RawFd,
    light_buf: [u8; 49],
    light_buf2: [u8; 32],
    light_buf3: [u8; 57],

    pads: [MaschinePad; 16],
    buttons: [u8; 27],

    midi_note_base: u8,
    roller_state: [usize; 9],
    roller_status: [i32; 9],
    mod_state: usize,
    padmode: usize,

    note: [[u8; 16]; 8],
    note_state: [[usize; 16]; 8],
    noteset: bool,
    noteidx: usize,

    vel: [[U7; 16]; 8],
    speed: u64,
    playing: bool,
    current_page: usize,
    selected_step: Option<usize>,
    clock_state: ClockState,
}

impl Mikro {
    fn sixteen_maschine_pads() -> [MaschinePad; 16] {
        [
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
            MaschinePad::default(),
        ]
    }

    pub fn new(dev: io::RawFd) -> Self {
        let mut _self = Mikro {
            dev: dev,
            light_buf: [0u8; 49],
            light_buf2: [0u8; 32],
            light_buf3: [0u8; 57],

            pads: Mikro::sixteen_maschine_pads(),
            buttons: [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
                0x10, 0x10, 0x10, 0x10, 0x10, 0x10,
            ],

            midi_note_base: 48,
            roller_state: [0usize; 9],
            roller_status: [0i32; 9],
            mod_state: 0,
            padmode: 0,

            note: [[48u8; 16]; 8],
            note_state: [[0usize; 16]; 8],
            noteset: false,
            noteidx: 0,

            vel: [[80u8; 16]; 8],
            speed: 100,

            playing: false,
            current_page: 0,
            selected_step: None,
            clock_state: ClockState::new(),
        };

        _self.light_buf[0] = 0x80;
        _self.light_buf2[0] = 0x82;
        _self.light_buf3[0] = 0x81;
        return _self;
    }

    fn read_buttons(&mut self, handler: &mut dyn MaschineHandler, buf: &[u8]) {
        for (idx, &byte) in buf[0..24].iter().enumerate() {
            let mut diff = (byte ^ self.buttons[idx]) as u32;
            //println!("IDX: {}, Value{}", idx, byte);
            let mut off = 0usize;
            while diff != 0 {
                off += (diff.trailing_zeros() + 1) as usize;
                let btn = BUTTON_REPORT_TO_MIKROBUTTONS_MAP[idx][8 - off]
                    .expect("unknown button received from device");
                if idx <= 7 {
                    if (byte & (1 << (off - 1))) != 0 {
                        //println!(" {} ", byte);
                        let is_down = true;
                        handler.button_down(self, btn, byte, is_down);
                    } else {
                        let is_down = false;
                        //print!(" {} ", byte);
                        handler.button_up(self, btn, byte, is_down);
                    };
                } else {
                        if idx % 2 == 0  {
                            handler.encoder_step(self, (idx - 7) / 2 ,byte as i32 );
                        } else {
                            self.set_roller_state(byte as usize, (idx - 8) / 2 as usize);
                        };
                };
                                diff >>= off;
            }

            self.buttons[idx] = byte;
        }

        if self.buttons[23] > 0xF {
            self.buttons[23] = buf[23];
            return;
        } else if self.buttons[23] == buf[23] {
            return;
        }
        self.buttons[23] = buf[23];
    }

    fn read_pads(&mut self, handler: &mut dyn MaschineHandler, buf: &[u8]) {
        let pads: &[u16] = unsafe { transmute(buf) };

        for i in 0..16 {
            let pressure = ((pads[i] & 0xFFF) as f32) / 4095.0;

            match self.pads[i].pressure_val(pressure) {
                MaschinePadStateTransition::Pressed => handler.pad_pressed(self, i, pressure),

                MaschinePadStateTransition::Aftertouch => handler.pad_aftertouch(self, i, pressure),

                MaschinePadStateTransition::Released => handler.pad_released(self, i),

                _ => {}
            }
        }
    }

    fn send_display_bits(&mut self, report_id: u8, bits: &[u8]) {
        let mut screen_buf = [0u8; 1 + 8 + 512];
        screen_buf[0] = report_id;
        screen_buf[5] = 0x08;
        screen_buf[7] = 0x20;
        screen_buf[1] = 0;
        screen_buf[3] = 0;

        let mut col: u8 = 0;
        let mut page: u8 = 0;
        let mut steps: u8 = 0;

        for &byte in bits {
            screen_buf[1] = col;
            screen_buf[3] = page;
            screen_buf[9] = byte;
            let _ = unistd::write(self.dev, &screen_buf);
            col += 1;
            steps += 1;
            if steps > 30 {
                steps = 0;
                col = 0;
                page += 1;
            }
        }
    }
}

fn set_rgb_light(rgb: &mut [u8], color: u32, brightness: f32) {
    let brightness = brightness * 0.5;

    rgb[0] = (brightness * (((color >> 16) & 0xFF) as f32)) as u8;
    rgb[1] = (brightness * (((color >> 8) & 0xFF) as f32)) as u8;
    rgb[2] = (brightness * (((color) & 0xFF) as f32)) as u8;
}

impl Maschine for Mikro {
    fn get_fd(&self) -> io::RawFd {
        return self.dev;
    }

    fn write_lights(&mut self) {
        let _ = unistd::write(self.dev, &self.light_buf);
        let _ = unistd::write(self.dev, &self.light_buf2);
        let _ = unistd::write(self.dev, &self.light_buf3);
    }

    fn set_pad_light(&mut self, pad: usize, color: u32, brightness: f32) {
        // LED report is display-order (top-left first); input is bottom-up row-major.
        // PAD_DISPLAY_ORDER is its own inverse, so applying it remaps correctly in both directions.
        const PAD_LED_MAP: [usize; 16] = [12, 13, 14, 15, 8, 9, 10, 11, 4, 5, 6, 7, 0, 1, 2, 3];
        let offset = 1 + (PAD_LED_MAP[pad] * 3);
        let rgb = &mut self.light_buf[offset..(offset + 3)];

        set_rgb_light(rgb, color, brightness);
    }

    fn set_midi_note_base(&mut self, base: u8) {
        self.midi_note_base = base;
    }

    fn get_midi_note_base(&self) -> u8 {
        return self.midi_note_base;
    }

    fn set_roller_state(&mut self, state: usize, idx: usize) {
        self.roller_state[idx] = state;
    }

    fn get_roller_state(&self, idx: usize) -> usize {
        return self.roller_state[idx];
    }

    fn set_roller_status(&mut self, status: i32, idx: usize) {
        self.roller_status[idx] = status;
    }

    fn get_roller_status(&self, idx:usize) -> i32 {
        return self.roller_status[idx]
    }
    fn set_mod(&mut self, state: usize) {
        self.mod_state = state;
    }

    fn get_mod(&self) -> usize {
        return self.mod_state;
    }

    fn set_padmode(&mut self, state: usize) {
        if self.padmode < 3 && state == 1 {
            self.padmode += 1
        } else {
            self.padmode = 0;
        };
        println!("Padmode {}", self.padmode);
        if self.padmode == 2 {
            println!("This is Sequencer mode");
            println!("");
            println!("Tapping on pads activates them for the sequence.");
            println!("Tapping on a pad while holding shift, then pressing another pad");
            println!("will change the note of the pad you pressed first");
        }
    }

    fn get_padmode(&self) -> usize {
        return self.padmode;
    }

    fn get_seq_page(&self) -> usize { self.current_page }

    fn set_seq_page(&mut self, page: usize) {
        if page < 8 {
            self.current_page = page;
            self.selected_step = None;
        }
    }

    fn get_selected_step(&self) -> Option<usize> { self.selected_step }

    fn set_selected_step(&mut self, step: Option<usize>) { self.selected_step = step; }

    fn get_step_note(&self, step: usize) -> u8 {
        self.note[self.current_page][step]
    }

    fn set_step_note(&mut self, step: usize, note: u8) {
        self.note[self.current_page][step] = note;
    }

    fn get_step_vel(&self, step: usize) -> u8 {
        self.vel[self.current_page][step]
    }

    fn set_step_vel(&mut self, step: usize, vel: u8) {
        self.vel[self.current_page][step] = vel;
    }

    fn apply_euclidean(&mut self, hits: usize) {
        let pattern = crate::sequencer::euclidean_pattern(16, hits);
        for i in 0..16 {
            self.note_state[self.current_page][i] = if pattern[i] { 1 } else { 0 };
        }
    }

    fn set_playing(&mut self, state: usize) {
        if state == 1 {
            self.playing = true;
        } else {
            self.playing = false;
        }
    }

    fn get_playing(&self) -> bool {
        return self.playing;
    }

    fn clock_tick(&mut self) -> Option<usize> {
        self.clock_state.on_clock_tick(std::time::Instant::now())
    }

    fn clock_start(&mut self) {
        self.clock_state.on_start();
        self.playing = true;
    }

    fn clock_stop(&mut self) {
        self.clock_state.on_stop();
        self.playing = false;
    }

    fn get_clock_state(&self) -> &ClockState {
        &self.clock_state
    }

    fn set_clock_source(&mut self, source: ClockSource) {
        self.clock_state.source = source;
    }

    fn note_save(&mut self, pad_idx: usize, note: u8, vel: u8) {
        if self.noteset {
            self.vel[self.current_page][self.noteidx] = vel;
            self.note[self.current_page][self.noteidx] = note;
            println!("step: {}, note:{}, velocity:{}", self.noteidx,
                self.note[self.current_page][self.noteidx],
                self.vel[self.current_page][self.noteidx]);
            self.noteset = false;
        } else {
            self.noteidx = pad_idx;
            self.noteset = true;
        }
    }

    fn note_state(&mut self, pad_idx: usize, msg: usize) {
        self.note_state[self.current_page][pad_idx] = msg;
    }

    fn note_check(&self, pad_idx: usize) -> usize {
        self.note_state[self.current_page][pad_idx]
    }

    fn load_notes(&self, pad_idx: usize, context: usize) -> midi::Message {
        if context == 1 {
            Message::NoteOn(Ch2, self.note[self.current_page][pad_idx],
                                  self.vel[self.current_page][pad_idx])
        } else {
            Message::NoteOff(Ch2, self.note[self.current_page][pad_idx],
                                   self.vel[self.current_page][pad_idx])
        }
    }

    fn set_seq_speed(&mut self, status: usize) {
        self.speed = status as u64;
        println!("sequencer rate: {}", self.speed);
    }

    fn get_seq_speed(&self) -> u64 {
        return self.speed
    }

    fn set_button_light(&mut self, btn: MaschineButton, _color: u32, brightness: f32) {
        let mut idx = 0;
        let mut idx2 = 0;
        match btn {
            MaschineButton::F8 => idx = 1,
            MaschineButton::F7 => idx = 2,
            MaschineButton::F6 => idx = 3,
            MaschineButton::F5 => idx = 4,
            MaschineButton::F4 => idx = 5,
            MaschineButton::F3 => idx = 6,
            MaschineButton::F2 => idx = 7,
            MaschineButton::F1 => idx = 8,

            MaschineButton::Auto => idx = 9,
            MaschineButton::All => idx = 10,
            MaschineButton::Pageleft => idx = 11,
            MaschineButton::Pageright => idx = 12,

            MaschineButton::Sampling => idx = 13,

            MaschineButton::Noterepeat => idx = 14,
            MaschineButton::Enter => idx = 15,
            MaschineButton::Navright => idx = 16,
            MaschineButton::Navleft => idx = 17,
            MaschineButton::Tempo => idx = 18,
            MaschineButton::Swing => idx = 19,
            MaschineButton::Volume => idx = 20,

            MaschineButton::Mute => idx = 21,
            MaschineButton::Solo => idx = 22,
            MaschineButton::Select => idx = 23,
            MaschineButton::Duplicate => idx = 24,
            MaschineButton::Navigate => idx = 25,
            MaschineButton::Padmode => idx = 26,
            MaschineButton::Pattern => idx = 27,
            MaschineButton::Scene => idx = 28,
            MaschineButton::Control => idx = 29,
            MaschineButton::Step => idx = 30,
            MaschineButton::Browse => idx = 31,

            MaschineButton::GroupH => idx2 = 2,
            MaschineButton::GroupG => idx2 = 9,
            MaschineButton::GroupF => idx2 = 14,
            MaschineButton::GroupE => idx2 = 22,
            MaschineButton::GroupD => idx2 = 26,
            MaschineButton::GroupC => idx2 = 34,
            MaschineButton::GroupB => idx2 = 39,
            MaschineButton::GroupA => idx2 = 48,
            MaschineButton::Shift => idx2 = 47,
            MaschineButton::Erase => idx2 = 56,
            MaschineButton::Rec => idx2 = 54,
            MaschineButton::Play => idx2 = 53,
            MaschineButton::Grid => idx2 = 52,
            MaschineButton::Stepright => idx2 = 51,
            MaschineButton::Stepleft => idx2 = 50,
            MaschineButton::Restart => idx2 = 49,

            _ => return,
        };
        if idx != 0 {
            //println!("light this {}, brightness {}", idx, brightness);
            self.light_buf2[idx] = brightness as u8;
        } else {
            self.light_buf3[idx2] = brightness as u8;
        }
    }

    fn readable(&mut self, handler: &mut dyn MaschineHandler) {
        let mut buf = [0u8; 512];

        let nbytes = match unistd::read(self.dev, &mut buf) {
            Err(err) => panic!("read failed: {}", err.to_string()),
            Ok(nbytes) => nbytes,
        };

        let report_nr = buf[0];
        let buf = &buf[1..nbytes];

        match report_nr {
            0x01 => self.read_buttons(handler, &buf),
            0x20 => self.read_pads(handler, &buf),
            0x03 => handler.midi_in_received(self, buf),
            _ => println!(" :: {:2X}: got {} bytes", report_nr, nbytes),
        }
    }

    fn get_pad_pressure(&self, pad_idx: usize) -> Result<f32, ()> {
        match pad_idx {
            0..=15 => Ok(self.pads[pad_idx].get_pressure()),
            _ => Err(()),
        }
    }

    fn clear_screen(&mut self) {
        let mut screen_buf = [0u8; 1 + 8 + 512];
        let mut screen_buf2 = [0u8; 1 + 8 + 512];

        screen_buf[0] = 0xE0;
        //screen_buf[3] = 16;
        screen_buf[5] = 0x08;
        screen_buf[7] = 0x20;

        //screen_buf[16] = 0xFF;

        screen_buf2[0] = 0xE1;
        //screen_buf2[3] = 16;
        screen_buf2[5] = 0x08;
        screen_buf2[7] = 0x20;

        let mut k = 0;
        let mut t = 0;
        while k < 9 {
            screen_buf[1] = k * 4;
            screen_buf2[1] = k * 4;
            k += 1;

            if k == 8 {
                screen_buf[3] = t * 4;
                screen_buf2[3] = t * 4;
                if t < 8 {
                    k = 0;
                }
                t += 1;
            }
            let _ = unistd::write(self.dev, &screen_buf);
            let _ = unistd::write(self.dev, &screen_buf2);
        }

        println!("Screen clear done?");
    }

    fn write_screen(&mut self) {
        let png_path = "picturetest.png";
        if !std::path::Path::new(png_path).exists() {
            return;
        }
        let mut limits = png::Limits::default();
        limits.bytes = 10 * 1024;
        let decoder = png::Decoder::new_with_limits(File::open(png_path).unwrap(), limits);
        let mut reader = decoder.read_info().unwrap();
        let mut picture = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut picture).unwrap();
        let bytes = &picture[..info.buffer_size()];
        let mut screen_buf = [0u8; 1 + 8 + 512];
        //println!("{}", bytes.len());

        //let mut screen_buf2 = [0u8; 1 + 8+ 512];
        screen_buf[0] = 0xE0;
        screen_buf[5] = 0x08;
        screen_buf[7] = 0x20;

        screen_buf[1] = 0;
        screen_buf[3] = 0;

        let mut screen_writer = 9;
        let mut steps = 0;
        let mut bits = [0u8; 4097];
        let mut inc = 0;
        let mut ok = 0;
        let mut count2 = 0;

        let mut a1 = 0;
        let mut a2 = 0;
        let mut a3 = 0;
        let mut a4 = 0;
        let mut a5 = 0;
        let mut a6 = 0;
        let mut a7 = 0;
        let mut a8 = 0;

        for count in 0..bytes.len() {
            let c = 1 + 4 * count;
            let mut swap = 0;
            //if bytes[c] / 8 + bytes[c + 1] / 8  + bytes[c + 2] / 8 + bytes[c + 3] / 8  + bytes[c + 4] / 8  + bytes[c + 5] / 8  + bytes[c + 6] / 8  + bytes[c + 7] / 8 >= 32{
            if c < bytes.len() - 3 {
                if bytes[c] / 2 + bytes[c + 2] / 2 >= 128 {
                    swap = 1;
                } else {
                    swap = 0;
                }
                //println!("{}", swap);
            }
            let mut binary = [0u8; 4097];
            if c < 65534 {
                //print!("{}, ", bytes[count]);
                let intval;
                match inc {
                    0 => a1 = swap,
                    1 => a2 = swap,
                    2 => a3 = swap,
                    3 => a4 = swap,
                    4 => a5 = swap,
                    5 => a6 = swap,
                    6 => a7 = swap,
                    7 => a8 = swap,
                    _ => return,
                }
                inc += 1;
                if inc == 8 {
                    inc = 0;
                }
                ok += 1;
                if ok == 8 {
                    let combination = format!("{}{}{}{}{}{}{}{}", a1, a2, a3, a4, a5, a6, a7, a8);
                    intval = usize::from_str_radix(&combination, 2).unwrap();
                    ok = 0;
                    binary[count2] = intval as u8;
                    bits[count2] = binary[count2];
                    count2 += 1;
                    a1 = 0;
                    a2 = 0;
                    a3 = 0;
                    a4 = 0;
                    a5 = 0;
                    a6 = 0;
                    a7 = 0;
                    a8 = 0;
                }
            }
            //let intval = usize::from_str_radix(&combination, 4).unwrap();
            //println!("{}", combination)
        }

        for a in 0..bits.len() {
            if screen_writer == 10 {
                if steps <= 30 {
                    screen_buf[1] += 1;
                    steps += 1;
                    screen_writer = 9;
                    screen_buf[screen_writer] = bits[a];
                } else {
                    screen_buf[3] += 1;
                    screen_buf[1] = 0;
                    steps = 0;
                    screen_writer = 9;
                    screen_buf[screen_writer] = bits[a];
                }
            }
            //println!("{}", bits[a]);
            let _ = unistd::write(self.dev, &screen_buf);
            screen_writer += 1;
        }
        println!("RUNNING!");
    }

    fn write_display(&mut self) {
        // send_display_bits sends one byte per HID report — floods USB at 2048 writes/100ms.
        // Display disabled until send_display_bits is rewritten to use proper bulk chunks.
        return;

        #[allow(unreachable_code)]
        const SZ: usize = display::HEIGHT * display::STRIDE;

        let note_names = ["C-1","C0","C1","C2","C3","C4","C5","C6","C7","C8","C9"];
        let base = self.midi_note_base;
        let note_name = note_names.get((base / 12) as usize).unwrap_or(&"?");

        // Left display (0xE0): encoders 0-3
        let mut left = [0u8; SZ];
        display::draw_text(&mut left, 0, 0, " K1    K2    K3    K4");
        let v0 = self.roller_status[0].clamp(0, 127);
        let v1 = self.roller_status[1].clamp(0, 127);
        let v2 = self.roller_status[2].clamp(0, 127);
        let v3 = self.roller_status[3].clamp(0, 127);
        let line1 = format!("{:>3}   {:>3}   {:>3}   {:>3}", v0, v1, v2, v3);
        display::draw_text(&mut left, 0, 10, &line1);
        let base_line = format!("BASE:{}{}", note_name, base);
        display::draw_text(&mut left, 0, 20, &base_line);

        // Right display (0xE1): encoders 4-7
        let mut right = [0u8; SZ];
        display::draw_text(&mut right, 0, 0, " K5    K6    K7    K8");
        let v4 = self.roller_status[4].clamp(0, 127);
        let v5 = self.roller_status[5].clamp(0, 127);
        let v6 = self.roller_status[6].clamp(0, 127);
        let v7 = self.roller_status[7].clamp(0, 127);
        let line2 = format!("{:>3}   {:>3}   {:>3}   {:>3}", v4, v5, v6, v7);
        display::draw_text(&mut right, 0, 10, &line2);

        self.send_display_bits(0xE0, &left);
        self.send_display_bits(0xE1, &right);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::Maschine;

    fn make_mikro() -> Mikro { Mikro::new(0) }

    #[test]
    fn pages_are_independent() {
        let mut m = make_mikro();
        m.note_state(0, 1);          // page 0 step 0 = on
        m.set_seq_page(1);
        assert_eq!(m.note_check(0), 0); // page 1 step 0 = still off
    }

    #[test]
    fn set_seq_page_clears_selected_step() {
        let mut m = make_mikro();
        m.set_selected_step(Some(3));
        m.set_seq_page(1);
        assert_eq!(m.get_selected_step(), None);
    }

    #[test]
    fn step_vel_independent_per_page() {
        let mut m = make_mikro();
        m.set_step_vel(0, 100);
        assert_eq!(m.get_step_vel(0), 100);
        m.set_seq_page(1);
        assert_eq!(m.get_step_vel(0), 80); // default unchanged
    }

    #[test]
    fn step_note_independent_per_page() {
        let mut m = make_mikro();
        m.set_step_note(0, 60);
        assert_eq!(m.get_step_note(0), 60);
        m.set_seq_page(1);
        assert_eq!(m.get_step_note(0), 48); // default unchanged
    }

    #[test]
    fn apply_euclidean_4_hits() {
        let mut m = make_mikro();
        m.apply_euclidean(4);
        assert_eq!(m.note_check(0),  1);
        assert_eq!(m.note_check(4),  1);
        assert_eq!(m.note_check(8),  1);
        assert_eq!(m.note_check(12), 1);
        assert_eq!(m.note_check(1),  0);
    }

    #[test]
    fn clock_start_resets_state() {
        let mut m = make_mikro();
        m.clock_tick();
        m.clock_tick();
        m.clock_start();
        assert!(m.get_playing());
        let state = m.get_clock_state();
        assert_eq!(state.step, 0);
        assert_eq!(state.tick_counter, 0);
    }

    #[test]
    fn clock_stop_halts_playing() {
        let mut m = make_mikro();
        m.clock_start();
        m.clock_stop();
        assert!(!m.get_playing());
    }

    #[test]
    fn six_clock_ticks_return_step_on_sixth() {
        let mut m = make_mikro();
        m.clock_start();
        for i in 0..5 {
            let result = m.clock_tick();
            assert!(result.is_none(), "tick {} should not advance step", i);
        }
        let result = m.clock_tick();
        assert_eq!(result, Some(0));
    }
}
