
use arnaudio::synth::*;
use helpers::*;
use arnaudio::voice;

use effects::vibrato::Vibrato;
use effects::leslie::Leslie;
use effects::room::Room;

use hammond::{Hammond, Osc as HOsc};
use pipe::{Pipe, Osc as POsc};

const WHEEL_COUNT: usize = 8;
const PIPE_COUNT: usize = 0;
const PIPE_PARAMS: usize = 6;
const FIRST_PARAMS: usize = 21;

const wheel_harmonics: [f32; WHEEL_COUNT] = [
  1.0, 3.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0
];

static mut rand: u16 = 0;
// el módulo es 2^16

// Debe ser coprimo de 2^16
static mut C: u16 = 165; // 3*5*11

#[derive(Default)]
struct Noise { x: u16 }
impl Noise {
  // Debe ejecutarse máximo una vez por sample,
  // porque tiene un periodo de 1.48 segundos
  fn clock (&mut self) -> f32 {
    self.x = self.x
      .wrapping_mul(4005) // 2*2*7*11*13 + 1
      .wrapping_add(165); // 3*5*11
    (self.x as f32) / (::std::u16::MAX as f32)
  }
}

#[derive(Default)]
struct Voice {
  pub gain: f32,
  pub freq: f32,
  pub sample: f32,

  pub main_osc: HOsc,
  pub pipe_oscs: [POsc; PIPE_COUNT],
}

impl voice::Voice for Voice {
  fn is_active(&self) -> bool {
    self.main_osc.is_active() || 
    self.pipe_oscs.iter().any(|osc| osc.is_active())
  }
}

pub struct Organ {
  sample_rate: f32,

  gain: f32,

  wheel_gains: [f32; WHEEL_COUNT],
  hammond: Hammond,
  pipes: [Pipe; PIPE_COUNT],

  voices: voice::Manager<Voice>,

  vibrato: Vibrato,
  leslie: Leslie,
  room: Room,

  noise: Noise,
}

macro_rules! zip {
  (mut $a:expr, mut $b:expr) => {
    $a.iter_mut().zip($b.iter_mut())
  };
  (mut $a:expr, $b:expr) => {
    $a.iter_mut().zip($b.iter())
  };
  ($a:expr, $b:expr) => {
    $a.iter().zip($b.iter())
  };
}

impl Synth for Organ {
  type Editor = ::editor::Gui;

  fn get_info() -> Info {
    Info {
      name: "Basic Plugin".to_string(),
      author: "Arnaud".to_string(),
      id: 42,
      params: FIRST_PARAMS + WHEEL_COUNT + PIPE_COUNT*PIPE_PARAMS,
    }
  }
  
  fn new () -> Organ {
    Organ {
      sample_rate: 44200_f32,

      gain: 1_f32,
      hammond: Hammond::new(),

      wheel_gains: [0.0; WHEEL_COUNT],
      pipes: [Default::default(); PIPE_COUNT],

      voices: Default::default(),

      vibrato: Vibrato::new(),
      leslie: Leslie::new(),
      room: Room::new(),

      noise: Default::default(),
    }
  }

  fn arch_change(&mut self, arch: Architecture) {
    self.sample_rate = arch.sample_rate;
    self.hammond.set_sample_rate(arch.sample_rate);
    self.vibrato.set_sample_rate(arch.sample_rate);
    self.leslie.set_sample_rate(arch.sample_rate);
    self.room.set_sample_rate(arch.sample_rate);
  }

  fn clock(&mut self) -> (f32, f32) {
    let mut smpl = 0_f32;

    let noise = self.noise.clock();
    self.hammond.set_noise(noise);

    for voice in self.voices.iter_mut() {
      let mut v_smpl = 0.0;

      v_smpl += self.hammond.run(&mut voice.main_osc);

      // Voice sample
      let s = {
        let delta = voice.freq / self.sample_rate;
        let s = mod1(voice.sample + delta);
        voice.sample = s;
        s // Return local s
      };

      for (osc, pipe) in zip!(mut voice.pipe_oscs, self.pipes) {
        // IMPORTANTE:
        // Creo que Rust tiene un bug.
        // Si activo está linea, el programa se pone raro y deja de sonar

        //if pipe.gain > 0.0 {
          pipe.envelope(osc, self.sample_rate);
          let s = mod1(s * pipe.harm);
          let vol = pipe.gain * osc.vol;
          v_smpl += pipe.sample(s) * vol;
        //}
      }

      smpl += v_smpl * voice.gain;
    }

    smpl = smpl*self.gain;

    smpl = self.vibrato.run(smpl);
    
    let (l, r) = self.leslie.run(smpl);
    let (l, r) = self.room.clock(l, r);
    (l, r)
  }

  fn note_on(&mut self, note: u8, vel: u8) {
    let mut voice = self.voices.note_on(note);

    // A4 es la nota midi 57, y está estandarizado como 440Hz
    let freq = 440.0 * 2_f32.powf((note as f32 - 57.0) / 12.0);

    voice.gain = vel as f32 / 256.0;
    self.hammond.note_on(&mut voice.main_osc, freq);

    for osc in voice.pipe_oscs.iter_mut() {
      osc.on()
    }
  }

  fn note_off(&mut self, note: u8) {
    match self.voices.note_off(note) {
      Some(voice) => {
        self.hammond.note_off(&mut voice.main_osc);
        for osc in voice.pipe_oscs.iter_mut() {
          osc.release()
        }
      }, _ => {}
    }
  }

  fn param_default(index: usize) -> f32 {
    match index {
      //14 => 1.0, // Room Size
      //19 => 1.0, // Room Mix
      //20 => 0.1, // Click

      0 => 0.2, // Warm

      /*21 => 1.0,
      22 => 0.95,
      23 => 0.9,
      24 => 0.85,
      25 => 0.8,
      26 => 0.7,
      27 => 0.6,
      28 => 0.5,*/

      20 => 1.0,

      21 => 0.4,
      22 => 0.1,

      _ => 0.0
    }
  }

  fn param_name (index: usize) -> String {
    match index {
      0 => "Warm".to_string(),
      1 => "Cold".to_string(),
      2 => "Attack".to_string(),
      3 => "Release".to_string(),

      4 => "Vibrato Depth".to_string(),
      5 => "Vibrato Freq".to_string(),
      6 => "Vibrato Mix".to_string(),

      7 => "Leslie Upper Freq".to_string(),
      8 => "Leslie Lower Freq".to_string(),
      9 => "Leslie Spread".to_string(),

      10 => "Leslie Tremolo Separation".to_string(),
      11 => "Leslie Vibrato Separation".to_string(),
      12 => "Leslie Stereo".to_string(),
      13 => "Leslie Mix".to_string(),

      14 => "Room Size".to_string(),
      15 => "Room Diff 1".to_string(),
      16 => "Room Diff 2".to_string(),
      17 => "Room Feedback".to_string(),
      18 => "Room Delay".to_string(),
      19 => "Room Mix".to_string(),

      20 => "Click".to_string(),
      _ => {
        let i = index - FIRST_PARAMS;
        if i < WHEEL_COUNT {
          format!("Wheel {}", i+1)
        } else {
          let i = i-WHEEL_COUNT;
          let pipe = (i/PIPE_PARAMS)+1;
          format!("Pipe {} {}", pipe, match i%PIPE_PARAMS {
            0 => "Gain",
            1 => "Harm",
            2 => "Warm",
            3 => "Cold",
            4 => "Attack",
            5 => "Release",
            _ => unreachable!()
          })
        }
      }
    }
  }

  fn set_param (&mut self, index: usize, value: f32) {
    match index {
      0 => {},//{self.main_pipe.warm = value; self.main_pipe.regen();},
      1 => {},//{self.main_pipe.cold = value; self.main_pipe.regen();},
      2 => {},//self.main_pipe.attack = value,
      3 => self.hammond.set_sustain(value),

      4 => self.vibrato.depth = value,
      5 => self.vibrato.freq = value,
      6 => self.vibrato.mix = value,

      7 => self.leslie.set_h_freq(value),
      8 => self.leslie.set_l_freq(value),
      9 => self.leslie.stereo = value,
      10 => {},//self.leslie.vol_sep = value,
      11 => {},//self.leslie.vib_sep = value,
      12 => {},//self.leslie.stereo = value,
      //12 => self.leslie.set_split(value),
      13 => {},//self.leslie.mix = value,

      14 => self.room.set_size(value),
      15 => self.room.diff1 = value,
      16 => self.room.diff2 = value,
      17 => self.room.set_feedback(value),
      18 => self.room.delay = value,
      19 => self.room.mix = value,

      20 => self.hammond.set_click(value),
      _ => {
        let i = index - FIRST_PARAMS;
        if i < WHEEL_COUNT {
          self.hammond.set_gain(i, value);
          //self.wheel_gains[i] = value;
        } else {
          let i = i - WHEEL_COUNT;
          let pipe = &mut self.pipes[i/PIPE_PARAMS];
          match i%PIPE_PARAMS {
            0 => pipe.gain = value,
            1 => pipe.set_harm(value),
            2 => {pipe.warm = value; pipe.regen();},
            3 => {pipe.cold = value; pipe.regen();},
            4 => pipe.attack = value,
            5 => pipe.release = value,
            _ => unreachable!()
          }

        }
      }
    }
  }
}
