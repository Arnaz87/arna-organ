
use helpers::*;

const TABLE_SIZE: usize = 128;
const F_TABLE_SIZE: f32 = TABLE_SIZE as f32;
pub const WHEEL_COUNT: usize = 8;

const harmonics: [f32; WHEEL_COUNT] = [
  1.0, 3.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0//, 16.0
];

const weights: [f32; WHEEL_COUNT] = [
  1.5, 1.0, 0.8, 0.8, 0.8, 0.8, 0.8, 0.6//, 0.6
];

/*
  Algoritmo de generación de ruido.

  fuentes:
  http://www.musicdsp.org/archive.php?classid=1#217
  http://www.musicdsp.org/showArchiveComment.php?ArchiveID=217

  Fórmula: frac((x+A)^2)
    con A siendo cualquier número no entero mayor a 1

  const A: f32 = 19.191919; // Cualquier número no entero mayor que 1
  state = {
    let x = state+A;
    (x*x).fract()
  }
*/

#[derive(Clone,Copy, PartialEq, Eq)]
pub enum State { Attack, Hold, Decay, Off }
impl Default for State { fn default () -> Self { State::Off } }

#[derive(Clone, Copy, Default)]
pub struct Osc {
  pub phase: f32,
  pub delta: f32,
  pub vol: f32,
  pub click: f32,
  pub state: State,
}

impl Osc {
  pub fn on (&mut self, delta: f32, click: f32) {
    // delta se calcula en A4 en 440Hz, pero eso es demasiado
    // agudo para el hammond, así que lo bajo dos octavas.
    self.delta = delta*0.25;
    self.phase = 0.0;
    self.vol = 0.0;
    self.click = click;
    self.state = State::Attack;
  }

  pub fn release (&mut self, click: f32) {
    self.click = click;
    self.state = State::Decay;
  }

  pub fn is_active (&self) -> bool {
    self.state != State::Off && self.click < 0.01
  }
}

pub struct Hammond {
  sample_rate: f32,

  sustain: f32,
  click: f32,

  // Deltas
  attack: f32,
  decay: f32,
  click_gain: f32,

  gains: [f32; WHEEL_COUNT],
  table: [f32; TABLE_SIZE],
}

// Todas estas están en segundos
const ATTACK: f32 = 0.005;
const CLICK:  f32 = 0.01;
const MIN_DECAY: f32 = 0.005;
const MAX_DECAY: f32 = 1.0;

impl Hammond {
  pub fn new () -> Self {
    Hammond {
      sample_rate: 0.0,

      sustain: 0.0,
      click: 0.0,

      attack: 0.0,
      decay: 0.0,
      click_gain: 0.0,

      gains: [0.0; WHEEL_COUNT],
      table: [0.0; TABLE_SIZE],
    }
  }

  pub fn set_sample_rate (&mut self, sr: f32) {
    self.sample_rate = sr;
    self.attack = 1.0 / (ATTACK * sr);
    let sust = self.sustain;
    self.set_sustain(sust);
    self.click_gain = 0.1_f32.powf(1.0 / (CLICK * sr));
  }

  pub fn set_sustain (&mut self, value: f32) {
    self.sustain = value;
    let time = lerp(MIN_DECAY, MAX_DECAY, value);
    let samples = time * self.sample_rate;
    self.decay = 1.0 / samples;
  }

  pub fn set_click (&mut self, value: f32) { self.click = value; }

  fn sample (&self, phase: f32) -> f32 {
    // La misma forma que usa AZR3
    0.5 * (
      ( 2.0*PI*phase).sin() +
      ( 8.0*PI*phase).sin()*0.03 +
      (12.0*PI*phase).sin()*0.01
    )
  }

  fn regen (&mut self) {
    for i in 0..TABLE_SIZE {
      let i_ph = i as f32 / TABLE_SIZE as f32;
      let mut s = 0.0;
      for w in 0..WHEEL_COUNT {
        let ph = mod1(i_ph * harmonics[w]);
        let vol = self.gains[w] * weights[w];
        s += self.sample(ph) * vol;
      }
      self.table[i] = s;
    }
  }

  pub fn run (&mut self, osc: &mut Osc) -> f32 {
    match osc.state {
      State::Attack => {
        osc.vol += self.attack;
        if osc.vol >= 1.0 {
          osc.vol = 1.0;
          osc.state = State::Hold;
        }
      },
      State::Decay => {
        osc.vol -= self.decay;
        if osc.vol <= 0.0 {
          osc.vol = 0.0;
          osc.state = State::Off;
        }
      },
      _ => {}
    }

    let phase = osc.phase;

    let i = phase as usize;
    let j = (i+1) % TABLE_SIZE;
    let t = phase - i as f32;
    let sample = lerp(self.table[i], self.table[j], t);

    osc.phase += osc.delta;
    if osc.phase >= F_TABLE_SIZE {
      osc.phase -= F_TABLE_SIZE
    }

    sample * osc.vol
  }

  // Necesito recibir delta en vez de freq o note, porque
  // desde aquí no tengo el sample_rate
  pub fn note_on(&self, osc: &mut Osc, freq: f32) {
    // delta está calculado para A4 en 440Hz, pero eso es muy
    // agudo para un Hammond, así que le bajo 2 octavas.
    osc.delta = freq*0.25*F_TABLE_SIZE/self.sample_rate;
    osc.phase = 0.0;
    osc.vol = 0.0;
    osc.click = self.click;
    osc.state = State::Attack;
  }

  pub fn note_off (&self, osc: &mut Osc) {
    osc.click =
      if self.sustain == 0.0
      { self.click } else { 0.0 };
    osc.state = State::Decay;
  }

  pub fn set_gain(&mut self, index: usize, gain: f32) {
    self.gains[index] = gain;
    self.regen();
  }
}