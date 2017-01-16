
use std::f32::EPSILON;
use helpers::*;

const TABLE_SIZE: usize = 128;

fn tension (x: f32, t: f32) -> f32 {
  if t < EPSILON { x }
  else { sigm(x*t)/sigm(t) }
}

fn tensinv (x: f32, t: f32) -> f32 {
  if x < 0.0 {
    tension(x+1.0,t)-1.0
  }
  else {
    tension(x-1.0,t)+1.0
  }
}

fn tens01 (x: f32, t: f32) -> f32 {
  // tension((x+1.0)/2.0, t)*2.0 - 1.0
  (tension(2.0*x - 1.0, t)+1.0)/2.0
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
  Off, Attack, Sustain, Release
}

impl Default for State {
  fn default () -> State { State::Off }
}

#[derive(Default, Copy, Clone)]
pub struct Osc {
  pub sample: f32,
  pub vol: f32,
  pub state: State,
}

pub struct Pipe {
  pub harm: f32,
  pub gain: f32,

  pub warm: f32,
  pub cold: f32,

  // Number of seconds the attack/release should last
  // between 0 and 1 second
  pub attack: f32,
  pub release: f32,

  table: [f32; TABLE_SIZE],
}

impl Default for Pipe {
  fn default () -> Pipe {
    Pipe {
      harm: 1.0,
      gain: 0.0,
      warm: 0.0,
      cold: 0.0,
      attack: 0.0,
      release: 0.0,
      table: [0.0; TABLE_SIZE],
    }
  }
}

impl Clone for Pipe {
  fn clone (&self) -> Pipe {
    let mut other = Pipe {
      harm: self.harm,
      gain: self.gain,
      warm: self.warm,
      cold: self.cold,
      attack: self.attack,
      release: self.release,
      table: [0.0; TABLE_SIZE]
    };
    {
      let iterator = (&mut other.table).iter_mut().zip(self.table.iter());
      for (other_sample, &self_sample) in iterator {
        *other_sample = self_sample;
      };
    }
    other
  }
}

impl Copy for Pipe {}

impl Pipe {
  fn synthesize (&self, mut s: f32) -> f32 {
    s = tens01(s, self.warm*20.0);
    let mut x = sin01(s);
    x = tensinv(x, self.cold*50.0);
    x
  }

  pub fn sample (&self, _si: f32) -> f32 {
    let si = _si*(TABLE_SIZE as f32);

    let i = (si) as usize;
    let j = (i+1)%TABLE_SIZE;

    let t = si - (i as f32);

    let sa = self.table[i];
    let sb = self.table[j];

    (1.0-t)*sa + t*sb

    //self.synthesize(_si)
  }

  pub fn regen (&mut self) {
    /*
    for (index, sample) in (&mut self.table).iter_mut().enumerate() {
      let s: f32 = (index as f32)/(TABLE_SIZE as f32);
      // No me deja porque ya pedí &mut self para iterarlo
      *sample = self.synthesize(s);
    }
    */

    for i in 0..TABLE_SIZE {
      let s: f32 = (i as f32)/(TABLE_SIZE as f32);
      self.table[i] = self.synthesize(s);
    }
  }

  pub fn envelope (&self, osc: &mut Osc, samplerate: f32) {
    match osc.state {
      State::Off => osc.vol = 0.0,
      State::Sustain => osc.vol = 1.0,
      State::Attack => {
        // number of samples the attack phase should last
        let samples = samplerate * self.attack;
        // volume the attack should increase per sample
        let delta = 1.0/samples;
        osc.vol = osc.vol + delta;
        if osc.vol >= 1.0 {
          osc.vol = 1.0;
          osc.state = State::Sustain;
        }
      }
      State::Release => {
        // number of samples the release phase should last
        let samples = samplerate * self.release;
        // volume the release should increase per sample
        let delta = 1.0/samples;
        osc.vol = osc.vol - delta;
        if osc.vol <= 0.0 {
          osc.vol = 0.0;
          osc.state = State::Off;
        }
      }
    }
  }

  pub fn set_harm(&mut self, h: f32) {
    // 32 armonicas posibles
    self.harm = (h*31.0 + 1.0).floor();
  }
}

impl Osc {
  pub fn on (&mut self) {
    self.sample = 0.0;
    self.vol = 0.0;
    self.state = State::Attack;
  }

  pub fn release (&mut self) {
    self.state = State::Release;
  }

  pub fn is_active (&self) -> bool { self.state != State::Off }
}