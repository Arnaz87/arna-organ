

pub use std::f32::consts::PI;

#[inline]
pub fn sin01 (x: f32) -> f32 { (x*2.0*PI).sin() }
//pub fn sigm (x: f32) -> f32 { x/(1.0 + x.abs()) }
//pub fn sigm (x: f32) -> f32 { x.tanh() }

#[allow(dead_code)]
pub fn fsin (x: f32) -> f32 {
  // Fast sin(2*PI*x)
  
  fn half (x: f32) -> f32 {
    let a = x - x*x; a*(3.1 + a*3.6)
  }

  let x = x*2.0;
  if x < 1.0 { half(x) }
  else { -half(x - 1.0) }
}

#[inline(always)]
pub fn lerp(a: f32, b: f32, t: f32) -> f32 { (1.0-t)*a + t*b }

pub fn mod1(mut x: f32) -> f32 {
  while x>=1.0 { x-=1.0; }; x
}

#[inline]
pub fn db2amp (x: f32) -> f32 { 10_f32.powf(x/20.0) }

#[inline]
pub fn gain2amp (x: f32) -> f32 {
  if x==0.0 { 0.0 } else
  { db2amp(20.0 * x.log2()) }
  //{ db2amp((x-1.0) * 80.0) }
}
