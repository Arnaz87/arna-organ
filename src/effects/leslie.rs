
use sample::*;
use helpers::*;
use effects::buffer::*;

pub struct Leslie {
  pub sample: f32,

  pub freq: f32,
  pub mix: f32,
  pub stereo: f32,

  pub vib_depth: f32,
  pub vol_depth: f32,
  pub vib_sep: f32,
  pub vol_sep: f32,

  sample_rate: f32,
  buffer: Buffer<f32>,
}

// Fuerza del vibrato del Leslie en segundos
const LESLIE_STRENGTH: f32 = 1.0/512.0;

impl Leslie {
  pub fn new () -> Leslie {
    Leslie {
      sample: 0.0,

      freq: 0.0,
      mix: 0.0,
      stereo: 0.0,

      vib_depth: 0.0,
      vol_depth: 0.0,
      vib_sep: 0.0,
      vol_sep: 0.0,

      sample_rate: 1.0,
      buffer: Buffer::new(),
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    // Crear un buffer de "STRENGTH" segundos
    self.buffer.init(LESLIE_STRENGTH, sample_rate);
  }

  fn speak (&self, vib_sep: f32, vol_sep: f32) -> f32 {

    // Aplicar modulo 1 y convertir de (-1..1) a (0..1)
    fn _sin (sample: f32) -> f32 { (sin01(mod1(sample))+1.0)/2.0 }

    let vib_s = _sin(self.sample + vib_sep);
    let vol_s = _sin(self.sample + vol_sep);

    let vib = vib_s * self.vib_depth * LESLIE_STRENGTH;
    let vol = (1.0 - self.vol_depth) + vol_s * self.vol_depth;

    self.buffer.interp(vib) * vol
  }

  pub fn run (&mut self, orig: f32) -> (f32, f32) {
    self.buffer.push(orig);

    if self.vib_depth == 0.0 && self.vol_depth == 0.0 {
      return (orig, orig);
    }

    // Convertir 0..1 a 0.5..15 Hz
    let freq = self.freq*14.5 + 0.5;
    let delta = freq/self.sample_rate;

    self.sample = mod1(self.sample + delta);

    let vib_sep = self.vib_sep - 0.5;
    let vol_sep = self.vol_sep - 0.5;

    let _l = self.speak( vib_sep,  vol_sep);
    let _r = self.speak(-vib_sep, -vol_sep);

    let l = lerp(_l, _r, self.stereo);
    let r = lerp(_r, _l, self.stereo);

    (l, r)
  }
}
