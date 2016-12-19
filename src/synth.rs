
use vst2::plugin::{Info as VstInfo, Plugin, Category, HostCallback};
use vst2::buffer::AudioBuffer;
use vst2::event::{Event as VstEvent};

use vst2::plugin::CanDo;
use vst2::api::Supported;

pub struct Info {
  pub name: String,
  pub author: String,
  pub id: u32,
  pub params: u16,
}

#[derive(Clone,Copy)]
pub struct Event {
  pub sample: u32,
  pub data: [u8; 3],
}

#[derive(Default,Clone,Copy)]
pub struct Architecture {
  pub sample_rate: f32,
}

// Para usar constantes, en vez del struct info,
// hay que poner al principio del archivo:
// #![feature(associated_consts)]

#[allow(unused_variables)]
pub trait Synth {
  //const id: i32;

  fn get_info() -> Info {
    Info {
      name: "Unnamed Synth".to_string(),
      author: "Annonymous".to_string(),
      id: 1,
      params: 0,
    }
  }
  fn new() -> Self;
  fn run(&mut self, left: &mut [f32], right: &mut [f32], events: Vec<Event>) {}

  fn set_param(&mut self, index: u16, value: f32) {}
  fn param_name(index: u16) -> String { format!("Parameter {}", index) }
  fn param_default(index: u16) -> f32 { 0.0f32 }
  fn param_label(index: u16, value: f32) -> String { format!("{}", value) }

  fn arch_change(&mut self, arch: Architecture) {}
}

pub struct SynthPlugin<T: Synth> {
  synth: T,
  params: Vec<f32>,
  events: Vec<Event>,
  arch: Architecture,
}

impl<T: Synth> Default for SynthPlugin<T> {
  fn default () -> SynthPlugin<T> {
    let info = T::get_info();
    let mut synth = T::new();

    let arch = Architecture{sample_rate: 44000_f32};

    synth.arch_change(arch);

    let mut params = vec![0_f32; info.params as usize];
    for i in 0..(info.params-1) {
      let value = T::param_default(i);
      synth.set_param(i, value);
      params[i as usize] = value;
    }

    SynthPlugin{
      synth: synth,
      params: params,
      events: Vec::new(),
      arch: arch,
    }
  }
}

impl<T: Synth> Plugin for SynthPlugin<T> {
  fn get_info(&self) -> VstInfo {
    let sinf = T::get_info();
    VstInfo {
      name: sinf.name,
      vendor: sinf.author,
      unique_id: sinf.id as i32,
      parameters: sinf.params as i32,

      category: Category::Synth,
      inputs: 0,
      outputs: 2,

      ..Default::default()
    }
  }

  fn can_do(&self, cd: CanDo) -> Supported {
    match cd {
      CanDo::ReceiveEvents => Supported::Yes,
      CanDo::ReceiveMidiEvent => Supported::Yes,
      _ => Supported::No,
    }
  }

  fn new (_: HostCallback) -> SynthPlugin<T> {
    Default::default()
  }

  fn can_be_automated(&self, _: i32) -> bool { true }

  fn get_parameter(&self, index: i32) -> f32 { self.params[index as usize] }

  fn set_parameter(&mut self, index: i32, value: f32) {
    self.params[index as usize] = value;
    self.synth.set_param(index as u16, value);
  }

  fn get_parameter_name(&self, index: i32) -> String { T::param_name(index as u16) }

  fn set_sample_rate(&mut self, rate: f32) {
    self.arch.sample_rate = rate;
    self.synth.arch_change(self.arch);
  }

  fn process(&mut self, buffer: AudioBuffer<f32>){
    use std::mem::replace;

    let (_, mut outputs) = buffer.split();

    let (mut hd, mut tl) = outputs.split_at_mut(1);
    let left: &mut [f32] = hd[0];
    let right: &mut [f32] = tl[0];

    let events = replace(&mut self.events, Vec::new());

    self.synth.run(left, right, events);
  }

  fn process_events(&mut self, events: Vec<VstEvent>) {
    for event in events {
      match event {
        VstEvent::Midi{delta_frames, data, ..} => {
          self.events.push(Event{
            sample: delta_frames as u32,
            data: data,
          })
        },
        _ => {}
      }
    }
  }
}