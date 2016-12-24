
use sample::*;
use helpers::*;
use effects::buffer::*;

// Fuerza del Vibrato en segundos.
const VIBRATO_STRENGTH: f32 = 1.0/64.0;

// TODO: Explicar cómo la fuerza en segundos se traduce a microtonos o Hertz

pub struct Vibrato {
  pub sample: f32,
  pub depth: f32,
  pub freq: f32,
  pub mix: f32,

  sample_rate: f32,
  buffer: Buffer<f32>,
}

impl Vibrato {
  pub fn new () -> Vibrato {
    Vibrato {
      sample: 0.0,
      depth: 0.0,
      freq: 0.0,
      mix: 0.0,

      sample_rate: 1.0,
      buffer: Buffer::new(),
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    // Create a buffer of "STRENGTH" seconds
    self.buffer.init(VIBRATO_STRENGTH, sample_rate);
  }

  pub fn run (&mut self, orig: f32) -> f32 {
    self.buffer.push(orig);

    // convert 0..1 to 1..20 Hz
    let freq = self.freq*19.0 + 1.0;

    let delta = freq/self.sample_rate;

    self.sample = mod1(self.sample + delta);
    
    let depth = self.depth * VIBRATO_STRENGTH;

    // Convertir (-1, 1) a (0, 1)
    let s = (sin01(self.sample)+1.0)/2.0;

    // La fuerza del vibrato también depende de la frecuencia
    let s = s*depth/freq;

    let delayed = self.buffer.interp(s);

    lerp(orig, delayed, self.mix)
  }
}
