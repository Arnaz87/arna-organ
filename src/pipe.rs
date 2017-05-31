
use std::f32::EPSILON;
use helpers::*;

const TABLE_SIZE: usize = 128;
const F_TABLE_SIZE: f32 = TABLE_SIZE as f32;

const harmonics: [f32; 23] = [
  0.25, 0.375, 0.5, 0.625, 0.75, // 5
  1.0, 1.25, 1.5, 1.75, // 4
  2.0, 2.5, 2.75, 3.0, 3.25, 3.5, // 6
  4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, // 7
  8.0 // 1
];

const WARM: f32 = 20.0;
const COLD: f32 = 50.0;

// Todas en segundos
const MIN_ATTACK: f32 = 0.02;
const MAX_ATTACK: f32 = 0.6;
const MIN_RELEASE: f32 = 0.02;
const MAX_RELEASE: f32 = 1.5;

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

/*
  Algoritmo para calcular samples:
  let s = tens01(phase, warm*20.0);
  let x = tensinv(sin01(s), cold*50.0);
  return x;
*/

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State { Off, Attack, Sustain, Release }
impl Default for State { fn default () -> State { State::Off } }

#[derive(Default)]
pub struct Osc {
  pub phase: f32,
  pub delta: f32,

  pub vol: f32,
  pub state: State,
}

pub struct Pipe {
  pub gain: f32,
  harm: f32,

  pub color: f32,

  pub attack: f32,
  pub release: f32,

  a_delta: f32,
  r_delta: f32,

  table: [f32; TABLE_SIZE],
}

impl Default for Pipe {
  fn default () -> Pipe {
    Pipe {
      harm: 1.0,
      gain: 0.0,

      color: 0.5,
      attack: 0.0,
      release: 0.0,

      a_delta: 0.0,
      r_delta: 0.0,

      table: [0.0; TABLE_SIZE],
    }
  }
}

impl Pipe {

  pub fn regen (&mut self) {
    let (warm, cold) = match self.color {
      x if x > 0.5 =>     ( (x-0.5) * 2.0 * WARM, 0.0),
      x if x < 0.5 => (0.0, (0.5-x) * 2.0 * COLD ),
      _ => (0.0, 0.0)
    };
    let (warm, cold) = (warm*warm, cold*cold);

    for i in 0..TABLE_SIZE {
      let s = i as f32 / F_TABLE_SIZE;
      let s = tens01(s, warm);
      let x = tensinv(sin01(s), cold);
      self.table[i] = x;
    }
  }

  pub fn calc_params (&mut self, fs: f32) {
    let a = lerp(MIN_ATTACK, MAX_ATTACK, self.attack);
    self.a_delta = 1.0 / (a*fs);

    let r = lerp(MIN_RELEASE, MAX_RELEASE, self.release);
    self.r_delta = db2amp(-20.0).powf(1.0 / (r*fs));
  }

  pub fn clock (&self, osc: &mut Osc) -> f32 {
    match osc.state {
      State::Attack => {
        osc.vol += self.a_delta;
        if osc.vol >= 1.0 {
          osc.vol = 1.0;
          osc.state = State::Sustain;
        }
      },
      State::Release => {
        osc.vol *= self.r_delta;
        if osc.vol*self.gain < db2amp(-80.0) {
          osc.vol = 0.0;
          osc.state = State::Off;
        }
      },
      _ => {}
    }

    // No puedo dejar de calcular el envelope porque el gain puede
    // cambiar en la mitad de una nota, pero lo que sí puedo dejar
    // de hacer es calcular el sample.
    if self.gain == 0.0 { return 0.0; }

    let phase = osc.phase;

    let i = phase as usize;
    let j = (i+1) % TABLE_SIZE;
    let t = phase - i as f32;
    let sample = lerp(self.table[i], self.table[j], t);

    osc.phase += osc.delta;
    if osc.phase >= F_TABLE_SIZE {
      osc.phase -= F_TABLE_SIZE
    }

    sample * osc.vol * self.gain
  }

  pub fn note_on (&self, osc: &mut Osc, delta: f32) {
    osc.phase = 0.0;
    osc.vol = 0.0;
    osc.delta = delta*self.harm*F_TABLE_SIZE;
    osc.state = State::Attack;
  }

  pub fn set_harm(&mut self, h: f32) {
    let high = (harmonics.len() - 1) as f32;
    let i = (h*high).min(high) as usize;
    self.harm = harmonics[i];
  }

  pub fn set_gain(&mut self, g: f32) {
    self.gain = gain2amp(g);
  }
}

impl Osc {
  pub fn release (&mut self) { self.state = State::Release; }
  pub fn is_active (&self) -> bool { self.state != State::Off }
}