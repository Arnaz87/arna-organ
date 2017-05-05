
extern crate image;

extern crate winapi;
extern crate user32;
extern crate gdi32;

#[macro_use]
extern crate lazy_static;

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

/// Una ventana (o sección de una) que se muestra en la pantalla y responde a eventos.
pub trait Window {
  /// El tamaño que ocupa esta ventana en la pantalla en píxeles.
  fn get_size (&self) -> (u32, u32);

  /// Recibe un Canvas que representa la sección visible de esta ventana,
  /// y pinta el contenido de la ventana sobre él.
  fn paint (&self, ctx: &mut Canvas);

  /// Recibe eventos y puede cambiar el estado de la ventana
  /// y del programa en base a ellos.
  fn event (&mut self, ev: Event);

  fn is_invalid (&self) -> bool;
}

#[cfg(windows)]
pub use self::windows::{register_window, Image, Canvas};
