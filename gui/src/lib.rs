
extern crate image;

extern crate winapi;
extern crate user32;
extern crate gdi32;

#[macro_use]
extern crate lazy_static;

use std::sync::{Arc, Mutex, MutexGuard};

#[cfg(windows)]
mod windows;

pub mod widget;

/// Color en componentes RGBA de 8 bits.
#[derive(Debug)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

impl Color {
  /// Construye un color simple sin alfa dados los componentes RGB.
  pub fn rgb (r:u8, g:u8, b:u8) -> Color {
    Color{r:r, g:g, b:b, a:255}
  }

  /// Construye un color sin alfa dada su representación hexadecimal.
  pub fn hex (x: u32) -> Color {
    Color{
      r: ((x >>16) & 0xff) as u8,
      g: ((x >> 8) & 0xff) as u8,
      b: ((x >> 0) & 0xff) as u8,
      a: 255,
    }
  }
}

#[derive(Debug, Copy, Clone)]
pub enum MouseBtn {R, L, M}

#[derive(Debug, Copy, Clone)]
pub enum Event {
  MouseMove(i32, i32),
  MouseUp(MouseBtn),
  MouseDown(MouseBtn),
}

pub trait Component {
  /// Recibe eventos y puede cambiar el estado de la ventana
  /// y del programa en base a ellos.
  fn event (&mut self, ev: Event);

  /// Recibe un Canvas que representa la sección visible de esta ventana,
  /// y pinta el contenido de la ventana sobre él.
  fn paint (&self, ctx: &mut Canvas);
}

#[cfg(windows)]
pub use self::windows::{Image, Canvas};

#[cfg(windows)]
use self::windows::HandlerBox;

#[derive(Clone)]
pub struct Handler {
  bx: Arc<Mutex<HandlerBox>>
}

// TODO: Creo que este handler debería ser dos handlers diferentes, un
// InHandler para pasarle a la ventana para que controle cosas internas
// (repaint, capture, release), y un OutHandler para que controle la ventana
// desde afuera (open y close)
impl Handler {
  pub fn new () -> Handler {
    Handler{
      bx: Arc::new(Mutex::new(HandlerBox::new()))
    }
  }

  fn bx (&self) -> MutexGuard<HandlerBox> { self.bx.lock().unwrap() }

  pub fn open (&self, ptr: *mut std::os::raw::c_void) { self.bx().open(ptr); }
  pub fn close (&self) { self.bx().close(); }

  pub fn repaint (&self) { self.bx().repaint(); }
  pub fn capture (&self) { self.bx().capture(); }
  pub fn release (&self) { self.bx().release(); }

  pub fn set_size (&self, w: usize, h: usize) { self.bx().set_size(w, h); }

  // W debería ser Send, pero no puedo hacerlo
  pub fn attach <W: Component + 'static> (&self, win: W) {
    self.bx().attach(win);
  }
}
