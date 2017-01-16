
use std::ops::*;
use std::iter::*;

#[derive(Default, PartialEq, Clone, Copy)]
pub struct Sample { l: f32, r: f32 }

#[allow(dead_code)]
impl Sample {
  pub fn new (l: f32, r: f32) -> Sample { Sample {l: l, r: r} }
  pub fn zero () -> Sample { Sample::new(0.0, 0.0) }
  pub fn from_tuple (tpl: (f32, f32)) -> Sample {
    let (l, r) = tpl;
    Sample {l: l, r: r}
  }
  pub fn to_tuple (self) -> (f32, f32) { (self.l, self.r) }

  pub fn scale (self, x: f32) -> Sample {
    Sample {l: self.l * x, r: self.r * x}
  }

  pub fn lerp (self, other: Sample, x: f32) -> Sample {
    let y = 1.0-x;
    Sample {
      l: y*self.l + x*other.l,
      r: y*self.r + x*other.r,
    }
  }
}

impl Add for Sample {
  type Output = Sample;
  fn add (self, other: Sample) -> Sample {
    Sample {
      l: self.l + other.l,
      r: self.r + other.r,
    }
  }
}

impl Sub for Sample {
  type Output = Sample;
  fn sub (self, other: Sample) -> Sample {
    Sample {
      l: self.l - other.l,
      r: self.r - other.r,
    }
  }
}

impl Mul for Sample {
  type Output = Sample;
  fn mul (self, other: Sample) -> Sample {
    Sample {
      l: self.l + other.l,
      r: self.r + other.r,
    }
  }
}

impl Neg for Sample {
  type Output = Sample;
  fn neg (self) -> Sample {
    Sample {
      l: -self.l,
      r: -self.r,
    }
  }
}

impl Sum for Sample {
  fn sum<I: Iterator<Item=Sample>>(iter: I) -> Sample {
    iter.fold(Sample::zero(), |acc, x| acc + x)
  }
}
