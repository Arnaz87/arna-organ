#[cfg(windows)]
mod windows;

#[derive(Debug)]
pub struct Color {
  r: u8,
  g: u8,
  b: u8,
  a: u8,
}

impl Color {
  pub fn rgb (r:u8, g:u8, b:u8) -> Color {
    Color{r:r, g:g, b:b, a:255}
  }

  pub fn hex (x: u32) -> Color {
    Color{
      r: ((x >>16) & 0xff) as u8,
      g: ((x >> 8) & 0xff) as u8,
      b: ((x >> 0) & 0xff) as u8,
      a: 255,
    }
  }
}

pub enum InputEvent {}

pub trait Window {
  fn get_size (&self) -> (u32, u32);
  fn paint (&self, ctx: &mut Canvas);
  fn input (&mut self, ev: InputEvent);
}

#[cfg(windows)]
pub use self::windows::{register_window, Image, Canvas};