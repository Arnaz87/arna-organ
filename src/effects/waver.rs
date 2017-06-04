
use helpers::*;

// TODO: El LFO Perlin no suena bien, debo cambiarlo por
// un LFO seno, más rápidos y talves más.

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
struct Phaser <T: Default> {
  // Sin feedback.
  ph: f32,
  a1: f32,
  min: f32, max: f32,
  //last: f32, fb: f32,
  stages: T,
  //stages: [Stage; STAGES],
}

impl <T: Default> Phaser<T> {
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
}

impl <'a, T: 'a + Default> Phaser<T>
  where &'a mut T: IntoIterator<Item=&'a mut Stage> {
  pub fn clock(&'a mut self, input: f32) -> f32 {
    let mut x = input;
    for stage in &mut self.stages {
      x = stage.clock(x, self.a1);
    }
    x
  }
}

// TODO: Optimizar.
#[derive(Default)]
struct SineLFO { dt: f32, ph: f32 }
impl SineLFO {
  pub fn set_freq (&mut self, freq: f32, fs: f32) { self.dt = 2.0*PI*freq/fs; }
  pub fn clock (&mut self) -> f32 { self.ph += self.dt; self.ph.sin()*0.5+0.5 }
}


// Frecuencia del perlin.
// Para el mejor efecto, deben ser fracciones menores que 2 Hz, con
// denominadores grandes, coprimos entre sí y con su numerador.
const FREQ1: f32 = 10.0/27.0; // 0.37
const FREQ2: f32 = 30.0/43.0; // 0.69
const FREQ3: f32 = 31.0/31.0; // 1.13

// La máxima frecuencia de la banda superior del phaser
const RG1: (f32, f32) = (200.0, 5000.0);
const RG2: (f32, f32) = (500.0, 1000.0);
const RG3: (f32, f32) = (4000.0, 9000.0);

pub struct Waver {
  depth: f32,
  lfo1: SineLFO,
  lfo2: SineLFO,
  lfo3: SineLFO,
  phaser1: Phaser<[Stage; 4]>,
  phaser2: Phaser<[Stage; 4]>,
  phaser3: Phaser<[Stage; 4]>,
}

impl Waver {
  pub fn new () -> Self {
    Waver {
      depth: 0.0,
      lfo1: Default::default(),
      lfo2: Default::default(),
      lfo3: Default::default(),
      phaser1: Default::default(),
      phaser2: Default::default(),
      phaser3: Default::default(),
    }
  }

  pub fn set_depth (&mut self, x: f32) { self.depth = x*0.5; }
  pub fn set_sample_rate (&mut self, fs: f32) {
    self.lfo1.set_freq(FREQ1, fs);
    self.lfo2.set_freq(FREQ2, fs);
    self.lfo3.set_freq(FREQ3, fs);
    self.phaser1.set_range(RG1.0, RG1.1, fs);
    self.phaser2.set_range(RG2.0, RG2.1, fs);
    self.phaser3.set_range(RG3.0, RG3.1, fs);
  }

  pub fn clock (&mut self, input: f32) -> f32 {
    if self.depth==0.0 { return input; }

    self.phaser1.set_phase(self.lfo1.clock());
    self.phaser2.set_phase(self.lfo2.clock());
    self.phaser3.set_phase(self.lfo3.clock());

    let mut s = input;
    s = lerp(s, self.phaser1.clock(s), self.depth);
    s = lerp(s, self.phaser2.clock(s), self.depth);
    s = lerp(s, self.phaser3.clock(s), self.depth);
    s
  }
}