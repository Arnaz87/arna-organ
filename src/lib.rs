
#[macro_use]
extern crate arnaudio;

mod sample;
mod pipe;
mod hammond;
mod organ;
mod effects;
mod helpers;
mod editor;

synth_main!(organ::Organ);