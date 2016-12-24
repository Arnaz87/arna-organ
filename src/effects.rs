
use helpers::*;
use sample::*;

struct Buffer<T: Default + Copy> {
  secs: f32,
  sample_rate: f32,
  size: usize,
  pub pos: usize,
  pub data: Vec<T>
}

impl<T: Default + Copy> Buffer<T> {
  pub fn new () -> Buffer<T> {
    Buffer {
      secs: 0.0,
      sample_rate: 0.0,
      size: 0,
      pos: 0,
      data: Vec::new(),
    }
  }

  pub fn init (&mut self, secs: f32, sample_rate: f32) {
    self.secs = secs;

    self.sample_rate = sample_rate;
    let size = (secs*sample_rate).ceil() as usize + 1;
    self.size = size;
    self.data = vec![Default::default(); size];
  }

  pub fn push (&mut self, value: T) {
    self.pos = {
      let p = self.pos+1;
      if p < self.size { p }
      else { p - self.size }
    };
    self.data[self.pos] = value;
  }

  fn index (&self, i: usize) -> T {
    debug_assert!(i <= self.size);
    let i = {
      // All of this needs to be done with i32, because
      // subtracting overflows to the negatives

      let i = (self.pos as i32) - (i as i32);
      if i >= 0 { i }
      else { i + (self.size as i32) }
    } as usize;

    self.data[i]
  }

  pub fn get (&self, s: f32) -> T {
    debug_assert!(s <= self.secs);

    self.index((s*self.sample_rate) as usize)
  }
}

impl Buffer<f32> {
  pub fn interp (&self, s: f32) -> f32 {
    let si = s * self.sample_rate;

    let i = si as usize;
    let j = (i+1) % self.size;

    let t = si - (i as f32);

    let a = self.index(i);
    let b = self.index(j);

    lerp(a, b, t)
  }
}

//=== Vibrato Effect ===//

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

//=== Leslie Effect ==//

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

//=== Room Effect ===//

// Pulse Stuff
  #[derive(Clone, Copy)]
  enum Side {L, R}
  #[derive(Clone, Copy)]
  struct Pulse {
    side: Side,
    gain: f32,
    delay: f32,
  }

  macro_rules! pulse {
    ($side:ident, $delay:expr, $gain:expr) => {
      Pulse{
        side: Side::$side,
        gain: $gain,
        // Restar los samples que la difusión retrasa, y convertirlo a Segundos
        delay: ($delay-835.0)/44100.0,
      }
    }
  }

  const PULSES: [Pulse; 8] = [
    pulse!(L, 1569.0, 0.7),
    pulse!(R, 1966.0, 0.7),
    pulse!(L, 4252.0, 1.0),
    pulse!(R, 4581.0, 1.0),
    pulse!(R, 7200.0, 0.6),
    pulse!(L, 7988.0, -0.6),
    pulse!(R, 10552.0, -0.3),
    pulse!(L, 11471.0, 0.3),
  ];
// End Pulse Stuff

// Feedback Stuff
  struct Feedback {
    delay: f32,
    feedback: f32,
    size: f32,
    buffer: Buffer<Sample>,
  }

  impl Feedback {
    pub fn new (delay: f32, feedback: f32) -> Feedback {
      Feedback {
        delay: delay,
        feedback: feedback,
        size: 1.0,
        buffer: Buffer::new(),
      }
    }

    pub fn set_size (&mut self, size: f32) { self.size = size; }

    pub fn set_sample_rate (&mut self, sample_rate: f32) {
      self.buffer.init(self.delay, sample_rate);
    }

    pub fn clock (&mut self, orig: Sample) -> Sample {
      let bufout = self.buffer.get(self.delay * self.size);
      let current = orig + bufout.scale(self.feedback);
      self.buffer.push(current);
      // No puedo devolver la señal actual porque ahí está la original,
      // y si lo hago todos los feedbacks sumarían su propia original
      bufout
    }
  }
// End Feedback Stuff

pub struct Room {
  pub size: f32,
  pub diff: f32,

  pub mix: f32,

  sample_rate: f32,

  orig_buf: Buffer<f32>,

  delay_buf: Buffer<Sample>,

  fb1: Feedback,
  fb2: Feedback,
  fb3: Feedback,
  fb4: Feedback,
}

impl Room {
  pub fn new () -> Room {
    Room {
      size: 1.0,
      diff: 1.0,
      mix: 0.0,

      sample_rate: 1.0,
      orig_buf: Buffer::new(),

      delay_buf: Buffer::new(),

      fb1: Feedback::new(235.0/44100.0, 0.6),
      fb2: Feedback::new(313.0/44100.0, 0.6),
      fb3: Feedback::new(610.0/44100.0, 0.6),
      fb4: Feedback::new(835.0/44100.0, 0.6),
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    // Un buffer de un tercio de segundo
    self.orig_buf.init(0.3, sample_rate);

    // Un buffer de 835 samples
    self.delay_buf.init(835.0/44100.0, sample_rate);

    self.fb1.set_sample_rate(sample_rate);
    self.fb2.set_sample_rate(sample_rate);
    self.fb3.set_sample_rate(sample_rate);
    self.fb4.set_sample_rate(sample_rate);
  }

  fn pulse (&self, pulse: &Pulse) -> Sample {
    let s = pulse.gain * self.orig_buf.get(pulse.delay * self.size);
    match pulse.side {
      Side::L => Sample::new(s, 0.0),
      Side::R => Sample::new(0.0, s),
    }
  }
  
  pub fn clock (&mut self, orig_l: f32, orig_r: f32) -> (f32, f32) {
    let mono = (orig_l + orig_r)/2.0;
    self.orig_buf.push(mono);
    
    //= Pulses Phase =/

    let pulsed = {
      let mut s = Sample::zero();
      for pulse in PULSES.iter() {
        s = s + self.pulse(pulse);
      }
      s
    };


    //= Difusion Phase =//

    self.delay_buf.push(pulsed);

    let diffused = {
      // Base, con delay porque no es el primero en sonar, hay 2 pre-ecos
      let base = self.delay_buf.get(self.size * 835.0/44100.0);

      // Pre-ecos: 313 y 835 samples de delay repsectivamente
      let pre1 = self.delay_buf.get(self.size * (835.0-313.0)/44100.0);
      let pre2 = pulsed; // Sin Delay porque en realidad es el primero que suena

      // Ecos con Feedback
      let fb1 = self.fb1.clock(base);
      let fb2 = self.fb2.clock(base);
      let fb3 = self.fb3.clock(base);
      let fb4 = self.fb4.clock(base);

      let sum = -pre1 + -pre2 + fb1 + fb2 + fb3 + fb4;

      // TODO: Averiguar la combinación de controles para que el volumen
      // no cambie al cambiar los parámetros del eco
      base.lerp(sum, self.diff*0.5)
    };

    //= Final =//

    Sample::new(orig_l, orig_r).lerp(diffused, self.mix).to_tuple()
  }
}