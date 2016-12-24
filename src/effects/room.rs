
use sample::*;
use helpers::*;
use effects::buffer::*;

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
    ($side:ident, $delay:expr) => {
      Pulse{
        side: Side::$side,
        gain: 1.0-($delay/23798.0),
        // Restar los samples que la difusión retrasa, y convertirlo a Segundos
        delay: ($delay-4252.0)/44100.0,
      }
    }
  }

  const PULSES: [Pulse; 14] = [
    //pulse!(L, 1569.0, 0.7),
    //pulse!(R, 1966.0, 0.7),
    pulse!(L, 4252.0),
    pulse!(R, 4581.0),
    pulse!(R, 7200.0),
    pulse!(NL, 7988.0),
    pulse!(NR, 10552.0),
    pulse!(L, 11471.0),
    pulse!(R, 13704.0),
    pulse!(L, 14031.0),
    pulse!(NL, 17102.0),
    pulse!(NR, 17511.0),
    pulse!(L, 20518.0),
    pulse!(R, 20714.0),
    pulse!(NL, 24257.0),
    pulse!(NR, 24652.0),
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
    pub fn set_feedback (&mut self, fb: f32) { self.feedback = fb; }

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

  feedback: f32,

  pub delay: f32,

  pub mix: f32,

  sample_rate: f32,

  orig_buf: Buffer<f32>,

  delay_buf: Buffer<Sample>,

  fb1: Feedback,
  fb2: Feedback,
  fb3: Feedback,
  fb4: Feedback,

  fb_buf: Buffer<Sample>,
}

impl Room {
  pub fn new () -> Room {
    Room {
      size: 1.0,
      diff: 1.0,
      feedback: 0.6,
      delay: 0.0,
      mix: 0.0,

      sample_rate: 1.0,
      orig_buf: Buffer::new(),

      delay_buf: Buffer::new(),

      fb1: Feedback::new(235.0/44100.0, 0.6),
      fb2: Feedback::new(313.0/44100.0, 0.6),
      fb3: Feedback::new(610.0/44100.0, 0.6),
      fb4: Feedback::new(835.0/44100.0, 0.6),

      fb_buf: Buffer::new(),
    }
  }

  pub fn set_sample_rate (&mut self, sample_rate: f32) {
    self.sample_rate = sample_rate;

    // Un buffer de un tercio de segundo
    self.orig_buf.init(23798.0/44100.0, sample_rate);

    // Un buffer de 835 samples
    self.delay_buf.init(835.0/44100.0, sample_rate);

    self.fb1.set_sample_rate(sample_rate);
    self.fb2.set_sample_rate(sample_rate);
    self.fb3.set_sample_rate(sample_rate);
    self.fb4.set_sample_rate(sample_rate);

    self.fb_buf.init(835.0/44100.0, sample_rate);
  }

  pub fn set_feedback (&mut self, fb: f32) {
    self.feedback = fb;

    self.fb1.set_feedback(fb);
    self.fb2.set_feedback(fb);
    self.fb3.set_feedback(fb);
    self.fb4.set_feedback(fb);
  }

  fn pulse (&self, pulse: &Pulse) -> Sample {

    let pulse_gain = 1.0-( pulse.delay / (23798.0/44100.0) );
    let vol = self.delay + pulse_gain*(1.0-self.delay);

    let s = vol * self.orig_buf.get(pulse.delay * self.size);
    match pulse.side {
      Side::L => Sample::new(s, 0.0),
      Side::R => Sample::new(0.0, s),
      Side::NL => Sample::new(-s, 0.0),
      Side::NR => Sample::new(0.0, -s),
    }
  }
  
  pub fn clock (&mut self, orig_l: f32, orig_r: f32) -> (f32, f32) {
    let mono = (orig_l + orig_r)/2.0;

    let repeat_pulse = self.orig_buf.get(23798.0/44100.0) * self.delay;
    self.orig_buf.push(mono + repeat_pulse);
    
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

    let diffused =  {
      // Base, con delay porque no es el primero en sonar, hay 2 pre-ecos
      let base = self.delay_buf.get(self.size * 835.0/44100.0);

      // Pre-ecos: 313 y 835 samples de delay repsectivamente
      let pre1 = self.delay_buf.get(self.size * (835.0-313.0)/44100.0);
      let pre2 = pulsed; // Sin Delay porque en realidad es el primero que suena

      // Filtros Comb paralelos
      let fb1 = self.fb1.clock(base);
      let fb2 = self.fb2.clock(base);
      let fb3 = self.fb3.clock(base);
      let fb4 = self.fb4.clock(base);

      let body = (-pre1 + -pre2 + fb1 + fb2 + fb3 + fb4);

      // TODO: Averiguar la combinación de controles para que el volumen
      // no cambie al cambiar los parámetros del eco
      base.lerp(body, self.diff*0.5)
    };

    //= Final =//

    Sample::new(orig_l, orig_r).lerp(diffused, self.mix).to_tuple()
  }
}