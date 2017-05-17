
use Component;

/// Un grupo de widgets
pub struct Group {
  pub children: Vec<Box<Component>>,
  pub bg: Option<(i32, i32, ::Image)>,
}

impl Component for Group {
  fn paint (&self, canvas: &mut ::Canvas) {
    match self.bg {
      Some((x, y, ref bg)) => canvas.fill_image((x,y), &bg),
      None => {}
    }
    for widget in &self.children {
      widget.paint(canvas);
    }
  }

  fn event (&mut self, ev: ::Event) {
    for widget in &mut self.children {
      widget.event(ev);
    }
  }
}

pub trait Painter {
  fn set_value(&mut self, value: f32);
  fn paint(&self, canvas: &mut ::Canvas, x: i32, y: i32);
}

pub enum SliderStyle {Vertical, VerticalInverse}

pub struct Slider<P: Painter, F: Fn(f32)> {
  x: i32,
  y: i32,
  w: i32,
  h: i32,
  sensitivity: f32,
  handler: ::Handler,
  style: SliderStyle,
  painter: P,
  callback: F,

  // Estado
  value: f32,
  active: bool,
  mouse_x: i32,
  mouse_y: i32
}

impl<P: Painter, F: Fn(f32)> Slider<P, F> {
  pub fn new (
    x: i32, y: i32,
    w: i32, h: i32,
    sens: f32,
    handler: ::Handler,
    style: SliderStyle,
    painter: P,
    callback: F,
  ) -> Slider<P, F> {
    Slider {
      x: x, y: y, w: w, h: h,
      sensitivity: sens,
      handler: handler,
      style: style,
      painter: painter,
      callback: callback,
      value: 0.0,
      active: false,
      mouse_x: 0,
      mouse_y: 0,
    }
  }
}

impl<P: Painter, F: Fn(f32)> Component for Slider<P, F> {
  fn paint (&self, canvas: &mut ::Canvas) {
    self.painter.paint(canvas, self.x, self.y);
  }
  fn event (&mut self, ev: ::Event) {
    match ev {
      ::Event::MouseMove(x, y) => {
        if self.active {
          // Cuanto se ha movido el mouse en píxeles
          // y aumenta hacia abajo, x aumenta hacia la derecha
          let xmov = x - self.mouse_x;
          let ymov = y - self.mouse_y;

          // Cuantos píxeles se ha movido el mouse en la dirección deseada.
          let mov = match self.style {
            SliderStyle::Vertical => -ymov,
            SliderStyle::VerticalInverse => ymov
          };

          // Cuanto cambiar el valor
          let dif = mov as f32 / self.sensitivity;

          let value = {
            let v = self.value + dif;

            // clamp(v, 0, 1)
            if v > 1.0 { 1.0 }
            else if v < 0.0 { 0.0 }
            else { v }
          };

          if (value != self.value) {
            (self.callback)(value);
            self.painter.set_value(value);
            self.value = value;
            self.handler.repaint();
          }
        }
        self.mouse_x = x;
        self.mouse_y = y;
      },
      ::Event::MouseDown(::MouseBtn::L) => {
        if (!self.active &&
          self.mouse_x > self.x &&
          self.mouse_x < self.x + self.w &&
          self.mouse_y > self.y &&
          self.mouse_y < self.y + self.h
        ) {
          self.handler.capture();
          self.active = true;
        }
      },
      ::Event::MouseUp(::MouseBtn::L) => {
        if self.active {
          self.handler.release();
          self.active = false;
        }
      },
      _ => {}
    }
  }
}

pub enum SeqDirection {Horizontal, Vertical}

/// Pinta una sección diferente de la imagen por cada valor.
pub struct SeqPaint {
  /// Imagen original
  img: ::Image,

  /// Posición de la primera sección en la imagen original
  pos: (u16, u16),

  /// Tamaño de cada sección
  size: (u16, u16),

  /// Orientación de la secuencia de imágenes
  dir: SeqDirection,

  /// Distancia entre las secciones (usualmente mayor que w)
  distance: u16,

  /// Último índice de la sección
  last: u16,

  /// Sección actual, depende de set_value
  section: ::Image,
}

impl Painter for SeqPaint {
  fn set_value(&mut self, value: f32) {
    // value va de 0 a 1 inclusivo
    let i = (value * (self.last as f32)).floor() as u16;
    let add = i * self.distance;
    let (x, y) = match self.dir {
      SeqDirection::Horizontal => (self.pos.0 + add, self.pos.1),
      SeqDirection::Vertical   => (self.pos.0, self.pos.1 + add)
    };
    self.section = self.img.clone().crop(
      x as i32, y as i32,
      self.size.0 as i32,
      self.size.1 as i32
    );
  }

  fn paint(&self, canvas: &mut ::Canvas, x: i32, y: i32) {
    canvas.fill_image((x,y), &self.section);
  }
}


impl SeqPaint {
  pub fn new (
    img: ::Image,
    pos: (u16, u16),
    size: (u16, u16),
    dir: SeqDirection,
    dist: u16,
    count: u16
  ) -> SeqPaint {
    // Simula set_value(0)
    let section = img.clone().crop(
      pos.0 as i32, pos.1 as i32,
      size.0 as i32, size.1 as i32
    );
    SeqPaint {
      img: img,
      pos: pos,
      size: size,
      dir: dir,
      distance: dist,
      last: count-1,
      section: section
    }
  }
}

/*
pub struct RotPaint {
  img: ::Image,
  value: f32,
  /// Ángulo en vueltas (1 vuelta = 360°) cuando el valor es 0
  start: f32,
  /// Ángulo en vueltas (1 vuelta = 360°) cuando el valor es 1
  end: f32,
}

impl RotPaint {
  pub fn new (img: ::Image, start: f32, end: f32) -> RotPaint {
    RotPaint {
      img: img,
      start: start,
      end: end,
      value: 0.0,
    }
  }

  pub fn set_value (&mut self, value: f32) {
    self.value = value;
  }

  pub fn paint(&self, canvas: &mut ::Canvas) {
    // Lerp
    let angle =
      self.start * self.value +
      self.end * (1.0-self.value);

    let img = self.img.clone().rotate(angle);

    canvas.fill_image((32,32), &img);
  }
}
*/
