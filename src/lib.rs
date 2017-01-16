
#[macro_use]
extern crate vst2;
extern crate libc;
extern crate user32;
extern crate winapi;

mod editor;
mod sample;
mod synth;
mod voice;

mod pipe;
mod organ;
mod effects;
mod helpers;

#[macro_export]
macro_rules! synth_main {
  ($x:ty) => {
    plugin_main!(synth::SynthPlugin<$x>);
  }
}

synth_main!(organ::Organ);