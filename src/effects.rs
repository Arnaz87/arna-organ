
use helpers::*;


pub struct Buffer {
  sample_rate: f32,
  size: usize,
  pub pos: usize,
  pub data: Vec<f32>
}

impl Buffer {
  pub fn new () -> Buffer {
    Buffer {
      sample_rate: 0.0,
      size: 0,
      pos: 0,
      data: Vec::new(),
    }
  }

  pub fn init (&mut self, secs: f32, sample_rate: f32) {
    self.sample_rate = sample_rate;
    let size = (secs*sample_rate).ceil() as usize + 1;
    self.size = size;
    self.data = vec![0.0; size];
  }

  pub fn push (&mut self, value: f32) {
    self.pos = {
      let p = self.pos+1;
      if p < self.size { p }
      else { p - self.size }
    };
    self.data[self.pos] = value;
  }

  fn iget (&self, i: usize) -> f32 {
    let _i = i;

    let i = {
      // All of this needs to be done with i32, because
      // subtracting overflows to the negatives

      let i = (self.pos as i32) - (i as i32);
      if i >= 0 { i }
      else { i + (self.size as i32) }
    } as usize;

    self.data[i]
  }

  pub fn get (&self, s: f32) -> f32 {
    let si = s * self.sample_rate;

    let i = si.floor() as usize;
    let j = (i+1) % self.size;

    let t = si - (i as f32);

    let a = self.iget(i);
    let b = self.iget(j);

    lerp(a, b, t)
  }
}

// Strength of the Vibrato in seconds.
const VIBRATO_STRENGTH: f32 = 1.0/64.0;

// One second is just too much

pub struct Vibrato {
  pub sample: f32,
  pub depth: f32,
  pub freq: f32,
  pub mix: f32,

  sample_rate: f32,
  buffer: Buffer,
}

impl Vibrato {
  pub fn new () -> Vibrato {
    Vibrato {
      sample: 0.0,
      depth: 0.0,
      freq: 1.0,
      mix: 0.0,

      sample_rate: 1.0,
      buffer: Buffer::new(),
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    // Create a buffer of 1 second
    self.buffer.init(VIBRATO_STRENGTH, sample_rate);
  }

  pub fn run (&mut self, orig: f32) -> f32 {
    self.buffer.push(orig);

    // convert 0..1 to 1..20 Hz
    let freq = self.freq*19.0 + 1.0;

    let delta = freq/self.sample_rate;

    self.sample = mod1(self.sample + delta);


    // No entiendo esta fórmula...
    //let s = ((sin01(self.sample)+1.0)*self.depth*(self.sample_rate/128.0))/freq;
    
    let depth = self.depth * VIBRATO_STRENGTH;
    let s = ((sin01(self.sample)+1.0)/2.0)*depth/freq;

    let delayed = self.buffer.get(s);

    lerp(orig, delayed, self.mix)
  }
}

// TODO: Leslie y Pseudo-Reverb
