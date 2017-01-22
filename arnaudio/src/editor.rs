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
  fn input (&mut self, ev: ::gui::InputEvent) {}
}

impl Drop for EditorWindow {
  fn drop (&mut self) {
    println!("Dropping Editor Window");
  }
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {
    let window = EditorWindow{
      width: self.width,
      height: self.height,
      img: ::gui::Image::load("cpu.png").unwrap()
    };
    ::gui::register_window(window, ptr as *mut ::std::os::raw::c_void);
    self.isopen = true;
  }
}