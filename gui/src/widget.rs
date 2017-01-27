
/// El componente lógico principal en el que se dividen las secciones de una ventana.
pub trait Widget {
  /// Posición con (0,0) en la esquina superior izquierda del widget contenedor
  fn position (&self) -> (i32, i32);

  /// Pintar en un canvas cuya posición (0,0) es la posicón de este widget
  fn paint (&self, canvas: &mut ::Canvas);

  fn event (&mut self, ev: ::Event);
}

/// Un grupo de widgets
pub struct Group {
  pub pos: (i32, i32),
  pub children: Vec<Box<Widget>>,
  pub bg: Option<::Image>,
}

impl Widget for Group {
  fn position (&self) -> (i32, i32) { self.pos }
  fn paint (&self, canvas: &mut ::Canvas) {
    match self.bg {
      Some(ref bg) => canvas.fill_image((0,0), &bg),
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

pub struct Window {
  pub size: (u32, u32),
  pub main: Group
}

impl ::Window for Window {
  fn get_size (&self) -> (u32, u32) { self.size }
  fn paint(&self, canvas: &mut ::Canvas) { self.main.paint(canvas) }
  fn event(&mut self, ev: ::Event) { self.main.event(ev) }
}

pub struct SliderControl {
  x: i32,
  y: i32,
  w: i32,
  h: i32,
  pub value: f32,
  mouse_down: bool,
  mouse_x: i32,
  mouse_y: i32,
}

impl SliderControl {
  pub fn new (w: i32, h: i32) -> SliderControl {
    SliderControl {
      x: 0,
      y: 0,
      w: w,
      h: h,
      value: 0.0,
      mouse_down: false,
      mouse_x: 0,
      mouse_y: 0,
    }
  }

  pub fn event(&mut self, ev: ::Event) {
    match ev {
      ::Event::MouseMove(x, y) => {
        if self.mouse_down {
          // Cuanto se ha movido el mouse en y en píxeles
          let ydif = -(y - self.mouse_y);

          // Cuánto se ha mivodo relativo a su tamaño, en 0..1
          let yrel = (ydif as f32) / (self.h as f32);

          // y va hacia abajo, pero necesitamos la distancia hacia arriba,
          // por eso lo resto en vez de sumar
          let value = self.value + yrel;

          self.value = // clamp(value, 0, 1)
            if value > 1.0 { 1.0 }
            else if value < 0.0 { 0.0 }
            else { value };
        }
        self.mouse_x = x;
        self.mouse_y = y;
      },
      ::Event::MouseDown(::MouseBtn::L) => {
        self.mouse_down = true;
      },
      ::Event::MouseUp(::MouseBtn::L) => {
        self.mouse_down = false;
      },
      _ => {}
    }
  }
}

/// Pinta una sección diferente de la imagen por cada valor.
pub struct SeqPaint {
  pub img: ::Image,
  pub value: f32,
  /// Altura de cada sección.
  pub height: u16,
  /// Cantidad de secciones.
  pub count: u16,
}


impl SeqPaint {

  pub fn new (img: ::Image, height: u16, count: u16) -> SeqPaint {
    SeqPaint {
      img: img,
      height: height,
      count: count,
      value: 0.0,
      //section: img.clone,
    }
  }

  pub fn set_value (&mut self, value: f32) {
    self.value = value;
  }

  pub fn paint(&self, canvas: &mut ::Canvas) {
    let i = (self.value * (self.count as f32)).floor() as u16;

    let y = (i * self.height) as i32;

    let img = &self.img.clone().crop(0, y as i32, self.img.width as i32, self.height as i32);

    canvas.fill_image((0,0), &img);
  }
}


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
