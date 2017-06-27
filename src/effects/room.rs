
use sample::*;
use effects::buffer::*;
use helpers::*;

// Pulse Stuff
  #[derive(Clone, Copy)]
  enum Side {L, R, NL, NR}
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
        delay: ( $delay - 7200.0 ) / 44100.0,
      }
    };
    ($side:ident, $delay:expr) => {
      pulse!($side, $delay, 1.0-($delay/23798.0))
    }
  }

  const PULSES: [Pulse; 14] = [
    //pulse!(NL, 2577.0, 0.7),
    //pulse!(NR, 2771.0, 0.7),

    //pulse!(L, 4252.0, 1.0),
    //pulse!(R, 4581.0, 1.0),

    pulse!(R, 7200.0, 0.6),
    pulse!(NL, 7988.0, 0.6),
    pulse!(NR, 10552.0, 0.3),
    pulse!(L, 11471.0, 0.3),
    pulse!(R, 13704.0, 0.1),
    pulse!(L, 14031.0, 0.1),
    pulse!(NL, 17102.0, 0.0),
    pulse!(NR, 17511.0, 0.0),
    pulse!(L, 20518.0, 0.0),
    pulse!(R, 20714.0, 0.0),
    pulse!(NL, 24257.0, 0.0),
    pulse!(NR, 24652.0, 0.0),
    pulse!(L, 24652.0 + 3398.0, 1.0),
    pulse!(R, 24652.0 + 3398.0 + 329.0, 1.0),
  ];

  const MAX_PULSE: f32 = 23798.0/44100.0;
// End Pulse Stuff

// Allpass
  // Basado en el Allpass de Freeverb

  struct Allpass {
    delay: f32,
    feedback: f32,
    size: f32,
    buffer: Buffer<Sample>,
  }

  impl Allpass {
    pub fn new (delay: f32, feedback: f32) -> Self {
      Allpass {
        delay: delay,
        feedback: feedback,
        size: 1.0,
        buffer: Buffer::new(),
      }
    }

    pub fn set_size (&mut self, size: f32) { self.size = size; }
    pub fn set_feedback (&mut self, fb: f32) { self.feedback = fb; }

    pub fn set_sample_rate (&mut self, sample_rate: f32) {
      self.buffer.init(self.delay, sample_rate);
    }

    pub fn clock (&mut self, input: Sample) -> Sample {
      let bufout = self.buffer.get(self.delay * self.size);
      self.buffer.push(input + bufout.scale(self.feedback));
      bufout - input
    }
  }

// End Allpass

pub struct Room {
  pub size: f32,

  pub diff: f32,

  pub delay: f32,

  pub mix: f32,

  sample_rate: f32,

  /// Early reflections
  er_buf: Buffer<Sample>,

  /// Mono buffer for the repeating pulses
  pulse_buf: Buffer<f32>,

  ap1: Allpass,
  ap2: Allpass,
  ap3: Allpass,
  ap4: Allpass,

  gains: [f32; 7],
  rep_gain: f32,
  rep_delay: f32,
}

impl Room {
  pub fn new () -> Room {
    Room {
      size: 1.0,
      delay: 0.0,
      diff: 1.0,
      mix: 0.0,

      sample_rate: 1.0,
      pulse_buf: Buffer::new(),

      er_buf: Buffer::new(),

      // FL Reverb:  313 835 1148 - 235 610 845

      // FL Studio:     235 313 610 835
      // Freeverb:      225 341 441 556 (El peor)
      // Mda Ambiance:  107 142 227 379
      // Mda Amb. x 2:  214 284 454 758 (El mejor)

      /* Factores de Mda Ambiance:
        107: 107
        142: 2*71
        227: 227
        379: 379

        x4 +3

        431: 431
        571: 571
        901: 17*53
        1519: 7*7*31
      */

      ap1: Allpass::new(431.0/44100.0, 0.6),
      ap2: Allpass::new(571.0/44100.0, 0.6),
      ap3: Allpass::new(901.0/44100.0, 0.55),
      ap4: Allpass::new(1519.0/44100.0, 0.5),
      
      gains: [0.0; 7],
      rep_gain: 0.0,
      rep_delay: 0.0,
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    //self.pulse_buf.init(23798.0/44100.0, sample_rate);
    self.pulse_buf.init(MAX_PULSE, sample_rate);

    self.er_buf.init(7200.0/44100.0, sample_rate);

    self.ap1.set_sample_rate(sample_rate);
    self.ap2.set_sample_rate(sample_rate);
    self.ap3.set_sample_rate(sample_rate);
    self.ap4.set_sample_rate(sample_rate);

    self.recalc_delay();
  }

  pub fn set_size (&mut self, sz: f32) {
    self.size = sz;

    self.ap1.set_size(sz);
    self.ap2.set_size(sz);
    self.ap3.set_size(sz);
    self.ap4.set_size(sz);

    self.recalc_delay();
  }

  pub fn set_diffuse (&mut self, df: f32) {
    self.diff = df*df;
    self.ap1.set_feedback( 0.6 * df.powi(4) );
    self.ap2.set_feedback( 0.6 * df.powi(2) );
    self.ap3.set_feedback( 0.55 * df.powi(1) );
    self.ap4.set_feedback( 0.5 * df.powf(0.5) );
  }

  pub fn recalc_delay (&mut self) {
    // Delay indica cuanto tarda el eco en llegar a -20db,
    // va desde 0.1 hasta 3 segundos.
    let time = lerp(0.1, 3.0, self.delay);

    // Tiempo que dura el último pulso en sonar, en segundos.
    let rep_time = self.size * MAX_PULSE;

    // Si cada t reduzco 0.1 (-20db) (ej: t=0.5), para cuando llegue a 1 seg,
    // habré reducido 1/t veces (ej: 2), entonces para cuando llegue a rep_time
    // (r, ej: r=1.5) habré reducido r/t (=1.5/0.5=3), que es la cantidad
    // de veces que multipliqué 0.1.
    let rep_gain = 0.1_f32.powf(rep_time / time);

    // Si ese es el valor al final, y me toma 7 pasos llegar hasta allá
    // (son 14 pulsos, 7 pares), debo hallar el número que multiplicado
    // 7 veces me dé ese valor, la raíz séptima del número.
    let step_gain = rep_gain.powf(1.0 / 7.0);

    // Ahora sí, cada par debe multiplicar ese valor n veces
    for (i, g) in self.gains.iter_mut().enumerate() {
      *g = step_gain.powi(i as i32);
    }

    self.rep_gain = rep_gain;
    self.rep_delay = rep_time;
  }

  fn pulse (&self, pulse: Pulse) -> Sample {

    //let pulse_gain = 1.0-( pulse.delay / (self.size*23798.0/44100.0) );
    //let vol = self.delay + pulse_gain*(1.0-self.delay);

    let s = self.pulse_buf.get(pulse.delay * self.size);

    match pulse.side {
      Side::L => Sample::new(s, 0.0),
      Side::R => Sample::new(0.0, s),
      Side::NL => Sample::new(-s, 0.0),
      Side::NR => Sample::new(0.0, -s),
    }
  }
  
  pub fn clock (&mut self, orig_l: f32, orig_r: f32) -> (f32, f32) {
    self.er_buf.push( Sample::new(orig_l, orig_r) );

    let (er, mono) = {
      macro_rules! er_pulse {
        ($x:expr) => { self.er_buf.get(($x - 1519.0) * self.size / 44100.0) };
        ($x:expr, $l:expr, $r:expr) => { er_pulse!($x).stereo_scale($l, $r) }
      }

      let a = er_pulse!(2577.0, -0.7, 0.0);
      let b = er_pulse!(2771.0, 0.0, -0.7);
      let c = er_pulse!(4252.0, 1.0, 0.0);
      let d = er_pulse!(4581.0, 0.0, 1.0);

      let mono = er_pulse!(7200.0).get_mono();

      (a + b + c + d, mono)
    };

    let repeat_pulse = self.pulse_buf.get(self.rep_delay) * self.rep_gain;
    self.pulse_buf.push(mono * self.gains[0] + repeat_pulse);
    
    //= Fase de pulsos =/
    let pulsed = {
      let mut s = er;
      for i in 0 .. 7 {
        let g = self.gains[i];
        s = s +
          self.pulse(PULSES[i*2]).scale(g) +
          self.pulse(PULSES[i*2+1]).scale(g);
      }
      s
    };

    let diffused = {
      // Allpass paralelos
      let ap1 = self.ap1.clock(pulsed);
      let ap2 = self.ap2.clock(ap1);
      let ap3 = self.ap3.clock(ap2);
      let ap4 = self.ap4.clock(ap3);
      ap4.scale(0.25)
    };

    let diffused = pulsed.lerp(diffused, self.diff).scale(0.5);

    Sample::new(orig_l, orig_r).lerp(diffused, self.mix).to_tuple()
  }
}