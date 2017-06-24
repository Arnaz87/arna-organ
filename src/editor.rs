
use ::arnaudio::editor::*;
use ::arnaudio::gui;

struct Tab {
  btn_img: gui::Image,

  btn_pos: (i32, i32),
  btn_size: (i32, i32),

  sec1_pos: (i32, i32), // para false
  sec2_pos: (i32, i32), // para true

  btn_sec: gui::Image,

  children: Vec<usize>,
}

impl Tab {
  pub fn new (
    img: gui::Image,
    btn_pos: (i32, i32),
    btn_size: (i32, i32),
    sec1_pos: (i32, i32),
    sec2_pos: (i32, i32),
    children: Vec<usize>
  ) -> Tab {
    let mut tab = Tab {
      btn_sec: img.clone(),
      btn_img: img,
      btn_pos: btn_pos,
      btn_size: btn_size,
      sec1_pos: sec1_pos,
      sec2_pos: sec2_pos,
      children: children,
    };
    tab.update_value(false);
    tab
  }

  pub fn update_value (&mut self, value: bool) {
    let (x, y) = if value { self.sec2_pos } else {self.sec1_pos};
    self.btn_sec = self.btn_img.clone().crop(
      x as i32, y as i32,
      self.btn_size.0 as i32,
      self.btn_size.1 as i32,
    );
  }

  pub fn in_btn_area (&mut self, x: i32, y: i32) -> bool {
    x > self.btn_pos.0 &&
    y > self.btn_pos.1 &&
    x < self.btn_pos.0 + self.btn_size.0 &&
    y < self.btn_pos.1 + self.btn_size.1
  }

  pub fn paint_btn (&self, canvas: &mut gui::Canvas) {
    canvas.fill_image(self.btn_pos, &self.btn_sec);
  }

  pub fn event_children ( &mut self, ev: gui::Event,
    controls: &mut Vec<Option<Control>>) {
    for &i in &self.children {
      match controls[i] {
        Some(ref mut control) => control.event(ev),
        None => {}
      }
    }
  }

  pub fn paint_children (&self, canvas: &mut gui::Canvas, controls: &Vec<Option<Control>>) {
    for &i in &self.children {
      match controls[i] {
        Some(ref control) => control.paint(canvas),
        None => {}
      }
    }
  }
}

struct TabGroup {
  active: usize,
  tabs: Vec<Tab>,
  win: gui::Handler,
}

impl TabGroup {
  pub fn new (win: gui::Handler, tabs: Vec<Tab>) -> Self {
    let mut grp = TabGroup { active: 0, tabs: tabs, win: win };
    grp.set_active(0);
    grp
  }

  fn set_active (&mut self, index: usize) {
    self.tabs[self.active].update_value(false);
    self.tabs[index].update_value(true);
    self.active = index;
    self.win.repaint();
  }

  pub fn event (&mut self, ev: gui::Event, controls: &mut Vec<Option<Control>>) {
    let mut new_active = None;
    for (i, tab) in self.tabs.iter_mut().enumerate() {
      match ev {
        gui::Event::MouseDown(gui::MouseBtn::L, x, y) => {
          if tab.in_btn_area(x, y) {
            new_active = Some(i)
          }
        }, _ => {}
      }
    }
    match new_active {
      Some(i) => self.set_active(i),
      None => {}
    }
    self.tabs[self.active].event_children(ev, controls)
  }

  pub fn paint (&self, canvas: &mut gui::Canvas, controls: &Vec<Option<Control>>) {
    for tab in self.tabs.iter() { tab.paint_btn(canvas); }
    self.tabs[self.active].paint_children(canvas, controls);
  }
}

type Control = Box<gui::widget::Control<Value=f32>>;

pub struct Gui {
  /// Array propietario de todos los controles
  controls: Vec<Option<Control>>,

  /// Índices de los controles que siempre están visibles
  main_controls: [usize; 28],

  pipe_tabs: TabGroup,

  //controls: Vec<(usize, Box<gui::widget::Control<Value=f32>>)>,
  //group: gui::widget::Group
  background: gui::Image
}

impl Editor for Gui {
  fn size () -> (usize, usize) { (550, 330) }

  fn new (synth: Channel, win: gui::Handler) -> Gui {
    let background = gui::Image::load("assets/background.png").unwrap();
    let knob_img = gui::Image::load("assets/knob.png").unwrap();
    let drawbar_img = gui::Image::load("assets/drawbar-red.png").unwrap();
    let btn_img = gui::Image::load("assets/btns.png").unwrap();

    let mut controls = {
      let mut vec: Vec<Option<Control>> = Vec::with_capacity(200);
      for _ in 0..200 { vec.push(None); }
      vec
    };

    /*let mut knob = |x: i32, y: i32, i: usize| {
      controls[i] = Some(
        Box::new(gui::widget::Slider::new(
          x, 330 - (y + 22), 22, 22,
          100.0,
          win.clone(),
          gui::widget::SliderStyle::Vertical,
          gui::widget::SeqPaint::new(
            knob_img.clone(),
            (0, 0), // Posición
            (22, 22), // Tamaño
            gui::widget::SeqDirection::Vertical,
            22, // Distancia
            49, // Cantidad
          ),
          {
            let synth = synth.clone();
            move |v: f32| {
              synth.set_param(i, v);
            }
          }
        ))
      );
      return i;
    };*/

    // Macros porque las funciones no pueden usar variables locales

    macro_rules! knob {
      ($x:expr, $y:expr, $i:expr) => {
        {controls[$i] = Some(
          Box::new(gui::widget::Slider::new(
            $x, 330 - ($y + 22), 22, 22,
            100.0,
            win.clone(),
            gui::widget::SliderStyle::Vertical,
            gui::widget::SeqPaint::new(
              knob_img.clone(),
              (0, 0), // Posición
              (22, 22), // Tamaño
              gui::widget::SeqDirection::Vertical,
              22, // Distancia
              49, // Cantidad
            ),
            {
              let synth = synth.clone();
              move |v: f32| {
                synth.set_param($i, v);
              }
            }
          ))
        ); $i}
      }
    }

    macro_rules! drawbar {
      ($x:expr, $y:expr, $i:expr) => {
        {controls[$i] = Some(
          Box::new(gui::widget::Slider::new(
            $x, 330 - ($y + 91), 16, 92,
            74.0,
            win.clone(),
            gui::widget::SliderStyle::VerticalInverse,
            gui::widget::SeqPaint::new(
              drawbar_img.clone(),
              (0, 0), // Posición
              (16, 92), // Tamaño
              gui::widget::SeqDirection::Horizontal,
              16, // Distancia
              9, // Cantidad
            ),
            {
              let synth = synth.clone();
              move |v: f32| {
                synth.set_param($i, v);
              }
            }
          ))
        ); $i}
      }
    }

    let mut base_controls = [

        // Tonewheels
        knob!( 55, 225, 0),
        knob!( 95, 225, 2),
        knob!(135, 225, 3),
        knob!(175, 225, 20),

        // Distort

        // Vibrato
        knob!(195, 30, 4),
        knob!(155, 30, 5),
        knob!(235, 30, 6),

        // Leslie
        knob!(295, 30, 7),
        knob!(335, 30, 8),
        knob!(415, 30, 9),
        //knob!(375, 30, 10), // Dummy

        // Room
        knob!(475, 150, 14),
        knob!(475, 110, 15),
        knob!(475,  70, 18), // Decay, por ahora diff2, o delay (?
        knob!(475,  30, 19),

        // TW Drawbars
        drawbar!(55, 89, 21),
        drawbar!(80, 89, 22),
        drawbar!(105, 89, 23),
        drawbar!(130, 89, 24),
        drawbar!(155, 89, 25),
        drawbar!(180, 89, 26),
        drawbar!(205, 89, 27),
        drawbar!(230, 89, 28),
        drawbar!(255, 89, 29),

        // Wheel Drawbars
        drawbar!(325, 89, 30),
        drawbar!(350, 89, 36),
        drawbar!(375, 89, 42),
        drawbar!(400, 89, 48),
        drawbar!(425, 89, 54),
    ];

    macro_rules! pipe_tab {
      ($i:expr) => {
        Tab::new(
          btn_img.clone(),
          (322+ $i*25, 135), (18, 12),
          (18*$i, 0), (18*$i, 12),
          vec![
            knob!(315, 225, $i*6 + 31),
            knob!(355, 225, $i*6 + 32),
            knob!(435, 225, $i*6 + 34),
            knob!(475, 225, $i*6 + 35),
          ]
        )
      }
    }

    let mut pipe_tabs = TabGroup::new(win.clone(), vec![
      pipe_tab!(0),
      pipe_tab!(1),
      pipe_tab!(2),
      pipe_tab!(3),
      pipe_tab!(4),
    ]);

    Gui {
      controls: controls,
      main_controls: base_controls,
      pipe_tabs: pipe_tabs,
      background: background,
    }

    /*Gui {
      controls: vec![],
      background: unsafe { ::std::mem::uninitialized() }
    }*/
  }

  fn set_param (&mut self, index: usize, value: f32) {
    match self.controls[index] {
      Some(ref mut control) => control.update_value(value),
      None => {}
    }
    /*for &mut (i, ref mut control) in &mut self.controls {
      if i == index {
        // update_value solo modifica el widget, no ejecuta el callback
        control.update_value(value);
      }
    }*/
  }
}

impl gui::Component for Gui {
  fn event (&mut self, ev: gui::Event) {
    for &i in &self.main_controls {
      match self.controls[i] {
        Some(ref mut control) => control.event(ev),
        None => {}
      }
    }
    self.pipe_tabs.event(ev, &mut self.controls);
  }

  fn paint (&self, canvas: &mut gui::Canvas) {
    canvas.fill_image((0,0), &self.background);
    for &i in &self.main_controls {
      match self.controls[i] {
        Some(ref control) => control.paint(canvas),
        None => {}
      }
    }
    self.pipe_tabs.paint(canvas, &self.controls);
  }
}


