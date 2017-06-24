
use std::f32::EPSILON;
use helpers::*;

const harmonics: [f32; 23] = [
  0.25, 0.375, 0.5, 0.625, 0.75, // 5
  1.0, 1.25, 1.5, 1.75, // 4
  2.0, 2.5, 2.75, 3.0, 3.25, 3.5, // 6
  4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, // 7
  8.0 // 1
];

const WARM: f32 = 60.0;
const COLD: f32 = 300.0;

// Todas en segundos
const MIN_ATTACK: f32 = 0.02;
const MAX_ATTACK: f32 = 0.6;
const MIN_RELEASE: f32 = 0.02;
const MAX_RELEASE: f32 = 1.5;

fn sigm (x: f32) -> f32 {
  // para que la derivada en 0 sea 1, hay que usar 4 como punto máximo

  if x >  3.4 { return  1.0; }
  if x < -3.4 { return -1.0; }

  let mut x = x/3.4;
  x = (x.abs() - 2.0)*x;
  x = (x.abs() - 2.0)*x;

  return x;
}

fn tension (x: f32, t: f32) -> f32 {
  if t < EPSILON { x }
  else { sigm(x*t)/sigm(t) }
}

fn tensinv (x: f32, t: f32) -> f32 {
  if x >= 0.0 { x.powf(t+1.0) }
  else { -(-x).powf(t+1.0) }
  /*if x < 0.0 {
    tension(x+1.0,t)-1.0
  } else {
    tension(x-1.0,t)+1.0
  }*/
}

fn tens01 (x: f32, t: f32) -> f32 {
  // tension((x+1.0)/2.0, t)*2.0 - 1.0
  (tension(2.0*x - 1.0, t)+1.0)/2.0
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State { Off, Attack, Sustain, Release }
impl Default for State { fn default () -> State { State::Off } }

#[derive(Copy, Clone)]
pub enum Form { Sine, Warm (f32), Cold (f32) }

#[derive(Default)]
pub struct Osc {
  pub phase: f32,
  pub delta: f32,

  pub bright: f32,

  pub vol: f32,
  pub state: State,
}

pub struct Pipe {
  pub gain: f32,
  harm: f32,

  //pub color: f32,
  form: Form,

  pub attack: f32,
  pub release: f32,

  a_delta: f32,
  r_delta: f32,
}

impl Default for Pipe {
  fn default () -> Pipe {
    Pipe {
      harm: 1.0,
      gain: 0.0,

      //color: 0.5,
      attack: 0.0,
      release: 0.0,

      form: Form::Sine,

      a_delta: 0.0,
      r_delta: 0.0,
    }
  }
}

impl Pipe {

  pub fn set_color (&mut self, color: f32) {
    self.form = match color {
      x if x > 0.5 => Form::Warm(x-0.5),
      x if x < 0.5 => Form::Cold(0.5-x),
      _ => Form::Sine
    };
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
    // de calcular es el sample.
    if self.gain == 0.0 { return 0.0; }

    let bright = osc.bright * (0.1*osc.vol + 0.9);

    let ph = osc.phase;

    let sample = match self.form {
      Form::Warm(x) => sin01(tens01(ph, x*WARM*bright)),
      Form::Cold(x) => tensinv(sin01(ph), x*COLD*bright),
      Form::Sine => sin01(ph),
    };

    osc.phase += osc.delta * self.harm;
    if osc.phase >= 1.0 { osc.phase -= 1.0; }

    sample * osc.vol * self.gain
  }

  pub fn note_on (&self, osc: &mut Osc, freq: f32, fs: f32) {
    // Color indica el brillo con el que suena el tubo que se activa cuando se
    // toca C3 (la nota más baja en un órgano real). La frecuencia que me dan
    // como parámetro asume el tono fundamental (lo cual no es inconveniente
    // porque el tubo siempre es armónico del fundamental, y se puede calcular
    // la frecuencia real con una simple fracción), en cuyo caso C3 sonaría
    // a 64Hz (http://www.die-orgelseite.de/fusszahlen_e.htm).

    // Nota: el link de arriba dice que los órganos reales se afinan con C3
    // a 64Hz, pero la frecuencia que recibo es con A5 a 440Hz, para que suene
    // afinado con el resto de los instrumentos.

    osc.phase = 0.0;
    osc.vol = 0.0;
    osc.delta = freq/fs;
    osc.bright = 64.0/freq;
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