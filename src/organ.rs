
use synth::*;
use pipe::*;
use effects::*;
use helpers::*;
use voice;

const WHEEL_COUNT: usize = 8;
const PIPE_COUNT: usize = 3;
const PIPE_PARAMS: usize = 6;

const wheel_harmonics: [f32; WHEEL_COUNT] = [
  1.0, 3.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0
];

#[derive(Default, Copy, Clone)]
struct Voice {
  pub note: u8,
  pub vel: u8,

  pub freq: f32,
  pub sample: f32,

  pub main_osc: Osc,
  pub pipe_oscs: [Osc; PIPE_COUNT],
}

impl voice::Voice for Voice {
  fn is_active(&self) -> bool {
    self.main_osc.is_active() || 
    self.pipe_oscs.iter().any(|osc| osc.is_active())
  }
  fn is_note(&self, note: u8) -> bool { self.note == note }
  fn note_on(&mut self, note: u8, vel: u8) {
    self.sample = 0.0;
    self.note = note;
    self.vel = vel;
    self.freq = 440.0 * 2_f32.powf(((note as f32)-57_f32)/12_f32);
    self.main_osc.on();
    for osc in self.pipe_oscs.iter_mut() {
      osc.on()
    }
  }
  fn note_off(&mut self) {
    self.sample = 0.0;
    self.main_osc.release();
    for osc in self.pipe_oscs.iter_mut() {
      osc.release()
    }
  }
}

pub struct Organ {
  sample_rate: f32,

  gain: f32,

  wheel_gains: [f32; WHEEL_COUNT],
  main_pipe: Pipe,
  pipes: [Pipe; PIPE_COUNT],

  voices: voice::Manager<Voice>,

  vibrato: Vibrato,
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
  fn get_info() -> Info {
    Info {
      name: "Basic Plugin".to_string(),
      author: "Arnaud".to_string(),
      id: 42,
      params: (5+WHEEL_COUNT+ PIPE_COUNT*PIPE_PARAMS) as u16,
    }
  }
  
  fn new () -> Organ {
    Organ {
      sample_rate: 44200_f32,

      gain: 1_f32,
      main_pipe: Default::default(),

      wheel_gains: [0.0; WHEEL_COUNT],
      pipes: [Default::default(); PIPE_COUNT],

      voices: Default::default(),

      vibrato: Vibrato::new(),
    }
  }

  fn arch_change(&mut self, arch: Architecture) {
    self.sample_rate = arch.sample_rate;
    self.vibrato.set_sample_rate(arch.sample_rate);
  }

  fn run(&mut self, left: &mut [f32], right: &mut [f32], events: Vec<Event>) {
    self.voices.add_events(events);

    for (lsample, rsample) in zip!(mut left, mut right) {
      self.voices.process_sample();

      let mut smpl = 0_f32;

      for voice in &mut self.voices {
        let mut v_smpl = 0.0;

        // Voice sample
        let s = {
          let delta = voice.freq / self.sample_rate;
          let s = mod1(voice.sample + delta);
          voice.sample = s;
          s // Return local s
        };

        self.main_pipe.envelope(&mut voice.main_osc, self.sample_rate);

        for (gain, harm) in zip!(self.wheel_gains, wheel_harmonics) {
          let s = mod1(s * harm);
          let vol = voice.main_osc.vol * gain;
          v_smpl += self.main_pipe.sample(s) * vol;
        }

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

        smpl += v_smpl * (voice.vel as f32/256.0) * self.gain;
      }

      smpl = smpl*self.gain;

      smpl = self.vibrato.run(smpl);

      *lsample = smpl;
      *rsample = smpl;
    }
  }

  fn param_name (index: u16) -> String {
    match index {
      0 => "Warm".to_string(),
      1 => "Cold".to_string(),
      2 => "Attack".to_string(),
      3 => "Release".to_string(),

      4 => "Vibrato Depth".to_string(),
      5 => "Vibrato Freq".to_string(),
      6 => "Vibrato Mix".to_string(),
      _ => {
        let i = (index-7) as usize;
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

  fn set_param (&mut self, index: u16, value: f32) {
    match index {
      0 => {self.main_pipe.warm = value; self.main_pipe.regen();},
      1 => {self.main_pipe.cold = value; self.main_pipe.regen();},
      2 => self.main_pipe.attack = value,
      3 => self.main_pipe.release = value,

      4 => self.vibrato.depth = value,
      5 => self.vibrato.freq = value,
      6 => self.vibrato.mix = value,
      _ => {
        let i = (index-7) as usize;
        if i < WHEEL_COUNT {
          self.wheel_gains[i] = value;
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
