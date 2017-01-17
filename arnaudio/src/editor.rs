use vst2::editor::{Editor as VstEditor};

//use libc::c_void;

pub struct Editor {
  isopen: bool,
  window: EditorWindow,
}

impl Editor {
  pub fn new () -> Editor {
    Editor {
      isopen: false,
      window: EditorWindow {
        width: 300,
        height: 100,
      }
    }
  }
}

struct EditorWindow {
  width: u32,
  height: u32,
}

impl ::graphics::Window for EditorWindow {
  fn get_size (&self) -> (u32, u32) { (self.width, self.height) }
  fn paint (&self, ctx: &mut ::graphics::Context) {}
  fn input (&mut self, ev: ::graphics::InputEvent) {}
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) {
    (self.window.width as i32,
    self.window.height as i32)
  }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {
    ::graphics::register_window(&mut self.window, ptr);
    self.isopen = true;
  }
}