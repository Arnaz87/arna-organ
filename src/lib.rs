
#[macro_use]
extern crate vst2;

mod synth;
mod voice;
mod pipe;
mod organ;
mod effects;
mod helpers;
mod sample;

#[macro_export]
macro_rules! synth_main {
  ($x:ty) => {
    plugin_main!(synth::SynthPlugin<$x>);
  }
}

synth_main!(organ::Organ);