
extern crate image;

extern crate winapi;
extern crate user32;
extern crate kernel32;
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
  MouseUp(MouseBtn, i32, i32),
  MouseDown(MouseBtn, i32, i32),
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
use self::windows::HandlerImpl;

#[derive(Clone)]
pub struct Handler {
  bx: Arc<Mutex<HandlerImpl>>
}

// TODO: Creo que este handler debería ser dos handlers diferentes, un
// InHandler para pasarle a la ventana para que controle cosas internas
// (repaint, capture, release), y un OutHandler para que controle la ventana
// desde afuera (open y close)
impl Handler {
  pub fn new () -> Self {
    Handler{
      bx: Arc::new(Mutex::new(HandlerImpl::new()))
    }
  }

  fn bx (&self) -> MutexGuard<HandlerImpl> { self.bx.lock().unwrap() }

  pub fn open (&self, ptr: *mut std::os::raw::c_void) { self.bx().open(ptr); }
  pub fn close (&self) { self.bx().close(); }

  pub fn repaint (&self) { self.bx().repaint(); }
  pub fn capture (&self) { self.bx().capture(); }
  pub fn release (&self) { self.bx().release(); }

  pub fn set_size (&self, w: usize, h: usize) { self.bx().set_size(w, h); }

  // W debería ser Send, pero no puedo hacerlo
  pub fn attach<T: Component + 'static> (&self, win: T) -> Arc<Mutex<T>> {
    let arc = Arc::new(Mutex::new(win));
    self.bx().attach(arc.clone());
    arc
  }
}
/*

  pub fn component (&self) -> Arc<Mutex<T>> {
    self.bx().component()
  }

  pub fn to_box (self) -> HandlerBox {
    HandlerBox{ bx: Box::new(self) }
  }
}

impl<T: Component + 'static> Clone for Handler<T> {
  fn clone (&self) -> Self {
    Handler { bx: self.bx.clone() }
  }
}

// Este trait solo es necesario para implementar HandlerBox
trait HandlerTrait {
  fn t_repaint (&self);
  fn t_capture (&self);
  fn t_release (&self);
}

impl<T: Component + 'static> HandlerTrait for Handler<T> {
  fn t_repaint (&self) { self.repaint(); }
  fn t_capture (&self) { self.capture(); }
  fn t_release (&self) { self.release(); }
}

#[derive(Clone)]
pub struct HandlerBox {
  bx: Box<HandlerTrait>
}

impl HandlerBox {
  fn repaint (&self) { self.bx.t_repaint(); }
  fn capture (&self) { self.bx.t_capture(); }
  fn release (&self) { self.bx.t_release(); }
}
*/