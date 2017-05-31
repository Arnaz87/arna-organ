
use helpers::*;

#[derive(Default)]
struct Noise { x: u16 }
impl Noise {
  pub fn with_seed (seed: u16) -> Self { Noise { x: seed } }
  pub fn clock (&mut self) -> f32 {
    self.x = self.x
      .wrapping_mul(4005) // 2*2*7*11*13 + 1
      .wrapping_add(165); // 3*5*11
    (self.x as f32) / (::std::u16::MAX as f32)
  }
}

struct Perlin {
  noise: Noise,
  delta: f32,
  s1: f32,
  s2: f32,
  t: f32
}

impl Perlin {
  pub fn new () -> Self { Self::with_seed(0) }
  pub fn with_seed (seed: u16) -> Self {
    let mut noise = Noise::with_seed(seed);
    let (s1, s2) = ( noise.clock(), noise.clock() );
    Perlin {
      noise: noise,
      delta: 0.0,
      s1: s1,
      s2: s2,
      t: 0.0
    }
  }
  pub fn set_freq (&mut self, freq: f32, fs: f32) { self.delta = freq/fs; }
  pub fn clock (&mut self) -> f32 {

    self.t += self.delta;
    if self.t >= 1.0 {
      self.s1 = self.s2;
      self.s2 = self.noise.clock();
      self.t -= 1.0;
    }

    let t = self.t;
    let fade = t*t*t*( t*(t*6.0 - 15.0) + 10.0 );

    lerp( self.s1*t, self.s2*(t-1.0), fade ) + 0.5
  }
}

const STAGES: usize = 4;

#[derive(Default)]
struct Stage { zm1: f32 }
impl Stage {
  fn clock (&mut self, input: f32, a1: f32) -> f32 {
    let output = self.zm1 - a1*input;
    self.zm1 = output*a1 + input;
    output
  }
}

#[derive(Default)]
struct Phaser {
  // Sin feedback.
  ph: f32,
  a1: f32,
  min: f32, max: f32,
  //last: f32, fb: f32,
  stages: [Stage; STAGES],
}
impl Phaser {
  pub fn set_range(&mut self, min: f32, max: f32, fs: f32) {
    self.min = min / (fs/2.0);
    self.max = max / (fs/2.0);
    self.calc_coeff();
  }
  pub fn set_phase(&mut self, ph: f32) {
    self.ph = ph; self.calc_coeff();
  }

  fn calc_coeff (&mut self) {
    let delay = lerp(self.min, self.max, self.ph);
    self.a1 = (1.0-delay)/(1.0+delay);
  }

  pub fn clock(&mut self, input: f32) -> f32 {
    let mut x = input;
    for stage in &mut self.stages {
      x = stage.clock(x, self.a1);
    }
    x
  }
}

// Frecuencia del perlin.
// Para el mejor efecto, deben ser fracciones menores que 2 Hz, con
// denominadores grandes, coprimos entre sí y con su numerador.
const FREQ1: f32 = 10.0/27.0; // 0.37
const FREQ2: f32 = 30.0/43.0; // 0.69
const FREQ3: f32 = 31.0/31.0; // 1.13

// La máxima frecuencia de la banda superior del phaser
const MAX1: f32 = 5000.0;
const MAX2: f32 = 1000.0;
const MAX3: f32 = 9000.0;

pub struct Waver {
  depth: f32,
  noise1: Perlin,
  noise2: Perlin,
  noise3: Perlin,
  phaser1: Phaser,
  phaser2: Phaser,
  phaser3: Phaser,
}

impl Waver {
  pub fn new () -> Self {
    Waver {
      depth: 0.0,
      noise1: Perlin::with_seed(36),
      noise2: Perlin::with_seed(127),
      noise3: Perlin::with_seed(1003),
      phaser1: Default::default(),
      phaser2: Default::default(),
      phaser3: Default::default(),
    }
  }

  pub fn set_depth (&mut self, x: f32) { self.depth = x*0.5; }
  pub fn set_sample_rate (&mut self, fs: f32) {
    self.noise1.set_freq(FREQ1, fs);
    self.noise2.set_freq(FREQ2, fs);
    self.noise3.set_freq(FREQ3, fs);
    self.phaser1.set_range(0.0, MAX1, fs);
    self.phaser2.set_range(0.0, MAX2, fs);
    self.phaser3.set_range(0.0, MAX3, fs);
  }

  pub fn clock (&mut self, input: f32) -> f32 {
    if self.depth==0.0 { return input; }

    self.phaser1.set_phase(self.noise1.clock());
    self.phaser2.set_phase(self.noise2.clock());
    self.phaser3.set_phase(self.noise3.clock());

    let mut s = input;
    s = lerp(s, self.phaser1.clock(s), self.depth);
    s = lerp(s, self.phaser2.clock(s), self.depth);
    s = lerp(s, self.phaser3.clock(s), self.depth);

    s
  }
}