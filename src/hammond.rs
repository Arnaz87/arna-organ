
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

// Todas estas están en segundos. Decay y Click indican el
// tiempo que se dura en llegar a -20dB = 0.1 de amplitud
const ATTACK: f32 = 0.005;
const MIN_DECAY: f32 = 0.005;
const MAX_DECAY: f32 = 1.0;
const CLICK:  f32 = 0.01;
// La voz se apaga cuando llega a 0.01 de amplitud, que son
// -40dB, y dura el doble del tiempo que dura en llegar a -20dB

const CLICK_START: f32 = 4.0;
const CLICK_END: f32 = 3.0;

const Q: f32 = 0.5;


#[derive(Default)]
pub struct StateVariable {
  q: f32, f: f32,
  l: f32, b: f32
}
impl StateVariable {
  pub fn set_params(&mut self, q: f32, fc: f32, fs: f32) {
    let f = fc.min(fs/4.0);
    self.q = q;
    self.f = 2.0*(PI*f/fs).sin();
  }
  // Devuelve un trío con (lowpass, bandpass, highpass)
  pub fn clock (&mut self, s: f32) -> (f32, f32, f32) {
    let h = s-self.l-self.b*self.q;
    self.b += self.f*h;
    self.l += self.f*self.b;
    (self.l, self.b, h)
  }
}

#[derive(Clone,Copy, PartialEq, Eq)]
pub enum State { Attack, Hold, Decay, Off }
impl Default for State { fn default () -> Self { State::Off } }

#[derive(Default)]
pub struct Osc {
  pub phase: f32,
  pub delta: f32,
  pub vol: f32,
  pub click: f32,
  pub state: State,
  pub filter: StateVariable,
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
    self.state != State::Off || self.click > 0.01
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

  noise: f32,
  gain_sum: f32,

  gains: [f32; WHEEL_COUNT],
  table: [f32; TABLE_SIZE],
}

impl Hammond {
  pub fn new () -> Self {
    Hammond {
      sample_rate: 0.0,
      sustain: 0.0,
      click: 0.0,

      attack: 0.0,
      decay: 0.0,
      click_gain: 0.0,

      noise: 0.0,
      gain_sum: 0.0,

      gains: [0.0; WHEEL_COUNT],
      table: [0.0; TABLE_SIZE],
    }
  }

  pub fn set_sample_rate (&mut self, sr: f32) {
    self.sample_rate = sr;
    self.attack = 1.0 / (ATTACK * sr);
    let sust = self.sustain;
    self.set_sustain(sust);
    self.click_gain = db2amp(-20.0).powf(1.0 / (CLICK * sr));
  }

  pub fn set_sustain (&mut self, value: f32) {
    self.sustain = value;
    let time = lerp(MIN_DECAY, MAX_DECAY, value);
    self.decay = db2amp(-20.0).powf(1.0 / (time * self.sample_rate));
  }

  pub fn set_noise (&mut self, value: f32) { self.noise = value-0.5; }

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
    self.gain_sum = self.gains.iter().zip(weights.iter())
      .map(|(a, b)| a*b ).sum();

    for i in 0..TABLE_SIZE {
      let i_ph = i as f32 / TABLE_SIZE as f32;
      let mut s = 0.0;
      for w in 0..WHEEL_COUNT {
        let ph = (i_ph * harmonics[w]) % 1.0;
        let vol = self.gains[w];
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
        osc.vol *= self.decay;
        // 0.0001 amplitud = -80dB
        if osc.vol <= 0.0001 {
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

    let (_, click, _) = osc.filter.clock(self.noise * osc.click);
    osc.click *= self.click_gain;

    sample*osc.vol + click
  }

  // Necesito recibir delta en vez de freq o note, porque
  // desde aquí no tengo el sample_rate
  pub fn note_on(&self, osc: &mut Osc, freq: f32) {
    // Los armónicos del hammond están descritos para el tw
    // de 16', pero supuestamente el fundamental es el de 8',
    // por eso debo bajar una octava.
    osc.delta = freq*0.5*F_TABLE_SIZE/self.sample_rate;
    osc.phase = 0.0;
    osc.vol = 0.0;
    osc.click = self.click * CLICK_START * self.gain_sum;
    osc.state = State::Attack;

    let click_f = 200.0+freq/2.0;
    osc.filter.set_params(Q, click_f, self.sample_rate);
  }

  pub fn note_off (&self, osc: &mut Osc) {
    osc.click =
      if self.sustain == 0.0
      { self.click * CLICK_END * self.gain_sum }
      else { 0.0 };
    osc.state = State::Decay;
  }

  pub fn set_gain(&mut self, index: usize, g: f32) {
    self.gains[index] = db2amp(10.0 * g.log2());
    self.regen();
  }
}