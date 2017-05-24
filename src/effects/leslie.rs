
use sample::*;
use helpers::*;
use effects::buffer::*;

/*

El diseño del leslie está inspirado por el del AZR3, pero reideado por mí.
La parte de arriba tiene dos cornetas de espalda, y la de abajo tiene una
sola. Para cada corneta hay que calcular dos canales por cada micrófono.
Además propia cabina refleja el sonido de la corneta que apunta hacia
adentro.

*/

struct Filter {
  s: f32,
  x: f32,
}

impl Filter {
  pub fn new () -> Self { Filter { s: 0.0, x: 0.0 } }

  pub fn config (&mut self, cuttoff: f32, sample_rate: f32) {
    let x = 1.0 - (2.0*cuttoff/sample_rate);
    self.x = x*x;
  }

  pub fn run (&mut self, input: f32) -> f32 {
    self.s = lerp(input, self.s, self.x); self.s
  }
}

pub struct Leslie {
  pub stereo: f32,

  sample_rate: f32,
  h_buffer: Buffer<f32>,
  l_buffer: Buffer<f32>,

  h_delta: f32,
  l_delta: f32,

  h_phase: f32,
  l_phase: f32,

  lp1: Filter,
  lp2: Filter,

  damp: Filter,
}

/// Tamaño/diámetro del Leslie en metros.
const SIZE_M: f32 = 1.0;

/// Con el tamaño, puedo calcular cuanto tiempo tarda el sonido en llegar
/// al frente cuando la corneta está atrás. La fórmula es:
/// v = 900mps; t(s)= d/v;
const DELAY_S: f32 = SIZE_M / 900.0;

/// Frecuencia que separa las cornetas de arriba y de abajo.
const SPLIT_F: f32 = 880.0;

/// Volumen de la corneta cuando está atrás.
const MIN_VOL: f32 = 0.4;

/// Volumen del bajo cuando está atrás.
const LOW_MIN_VOL: f32 = 0.75;

/// Cuttoff de los reflejos de la caja.
const DAMP_F: f32 = 880.0;

/// Volumen de los reflejos de la caja.
const DAMP_VOL: f32 = 0.3;

const MIN_FREQ: f32 = 0.1;
const MAX_FREQ: f32 = 20.0;

impl Leslie {
  pub fn new () -> Leslie {
    Leslie {
      stereo: 0.0,

      sample_rate: 1.0,
      h_buffer: Buffer::new(),
      l_buffer: Buffer::new(),

      lp1: Filter::new(),
      lp2: Filter::new(),

      damp: Filter::new(),

      h_delta: 0.0,
      l_delta: 0.0,
      h_phase: 0.0,
      l_phase: 0.0,
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    self.h_buffer.init(DELAY_S, sample_rate);
    self.l_buffer.init(DELAY_S, sample_rate);

    self.lp1.config(SPLIT_F, sample_rate);
    self.lp2.config(SPLIT_F, sample_rate);

    self.damp.config(DAMP_F, sample_rate);
  }

  pub fn set_h_freq (&mut self, f: f32) {
    let freq = lerp(MIN_FREQ, MAX_FREQ, f);
    self.h_delta = freq / self.sample_rate;
  }

  pub fn set_l_freq (&mut self, f: f32) {
    let freq = lerp(MIN_FREQ, MAX_FREQ, f);
    self.l_delta = freq / self.sample_rate;
  }

  pub fn run (&mut self, orig: f32) -> (f32, f32) {
    // Coseno con rango [0, 1] y dominio [0, 1]
    fn cos01 (x: f32) -> f32 {
      use std::f32::consts::PI;
      ((2.0*PI*x).cos() + 1.0)/2.0
    }

    fn speak (buf: &mut Buffer<f32>, x: f32, min: f32) -> f32 {
      let x = cos01(x);
      let vol = lerp(min, 1.0, x);
      let phase = 1.0-x;
      buf.interp(phase*DELAY_S)*vol
    }

    // Dos filtros en serie para separar mejor las cornetas
    let tmp = self.lp1.run(orig);
    let lp = self.lp2.run(tmp);
    let hp = orig - lp;

    self.l_buffer.push(lp);
    self.h_buffer.push(hp);

    let h_phase = self.h_phase + self.h_delta;
    let l_phase = self.l_phase + self.l_delta;
    self.h_phase = mod1(h_phase);
    self.l_phase = mod1(l_phase);

    let stereo = self.stereo * 0.25;

    // Cada micrófono escucha un sonido diferente porque la corneta
    // se mueve hacia ellos en momentos diferentes.
    let left  = speak(&mut self.h_buffer, h_phase - stereo, MIN_VOL);
    let right = speak(&mut self.h_buffer, h_phase + stereo, MIN_VOL);

    // Esta señal sale de la mitad de la caja, lo uso para los
    // reflejos de la caja
    let delayed = self.h_buffer.get(DELAY_S*0.5);

    // El sonido rebota muchas veces dentro de la caja, y resulta
    // en un sonido "mojado".
    let damp = self.damp.run(delayed) * DAMP_VOL;

    // Corneta de los bajos, gira al contrario que la de arriba, y
    // va a una velocidad diferente.
    let low_l = speak(&mut self.l_buffer, l_phase + stereo, LOW_MIN_VOL);
    let low_r = speak(&mut self.l_buffer, l_phase - stereo, LOW_MIN_VOL);

    // Los ecos en el bajo casi no se escuchan así que no los proceso

    // Mezcla final del sonido.
    (left+damp+low_l, right+damp+low_r)
  }
}
