
use sample::*;
use helpers::*;

pub struct Buffer<T: Default + Copy> {
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
