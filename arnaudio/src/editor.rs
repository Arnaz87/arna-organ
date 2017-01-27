use vst2::editor::{Editor as VstEditor};

//use libc::c_void;

pub struct Editor {
  isopen: bool,
  width: u32,
  height: u32
}

impl Editor {
  pub fn new () -> Editor {
    Editor {
      isopen: false,
      width: 300,
      height: 100,
    }
  }
}

struct EditorWindow {
  width: u32,
  height: u32,
  img: ::gui::Image,
}

impl ::gui::Window for EditorWindow {
  fn get_size (&self) -> (u32, u32) { (self.width, self.height) }
  fn paint (&self, canvas: &mut ::gui::Canvas) {
    canvas.fill_image((0,0), &self.img);
    canvas.fill_rect((20,20), (80,60), ::gui::Color::hex(0x77ccff));
  }
  fn event (&mut self, ev: ::gui::Event) {}
}

impl Drop for EditorWindow {
  fn drop (&mut self) {
    println!("Dropping Editor Window");
  }
}

struct MyKnob {
  value: f32,
  control: ::gui::widget::SliderControl,
  painter: ::gui::widget::SeqPaint,
}

impl MyKnob {
  pub fn new () -> MyKnob {
    MyKnob {
      value: 0.0,
      control: ::gui::widget::SliderControl::new(32, 32),
      painter: ::gui::widget::SeqPaint::new(
        ::gui::Image::load("cknob.bmp").unwrap(),
        44, // height
        40, // count
      ),
    }
  }
}

impl ::gui::widget::Widget for MyKnob {
  fn position (&self) -> (i32, i32) { (0,0) }
  fn paint (&self, canvas: &mut ::gui::Canvas) {
    &self.painter.paint(canvas);
  }
  fn event (&mut self, ev: ::gui::Event) {
    let old_val = self.control.value;
    &self.control.event(ev);
    let new_val = self.control.value;

    if old_val != new_val {
      &self.painter.set_value(new_val);
      println!("{}", new_val);
    }
  }
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {

    let window = ::gui::widget::Window {
      size: (self.width, self.height),
      main: ::gui::widget::Group {
        pos: (0,0),
        children: vec![
          Box::new(MyKnob::new())
        ],
        bg: ::gui::Image::load("craft.png"),
      }
    };

    ::gui::register_window(window, ptr as *mut ::std::os::raw::c_void);
    self.isopen = true;
  }
}