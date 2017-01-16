
#[macro_use]
extern crate arnaudio;

mod sample;
mod pipe;
mod organ;
mod effects;
mod helpers;

synth_main!(organ::Organ);