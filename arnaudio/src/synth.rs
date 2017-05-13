
use vst2::plugin::{Info as VstInfo, Plugin, Category, HostCallback};
use vst2::editor::{Editor as VstEditor};
use vst2::buffer::AudioBuffer;
use vst2::event::{Event as VstEvent};

use vst2::plugin::CanDo;
use vst2::api::Supported;

use editor::Editor;

pub struct Info {
  pub name: String,
  pub author: String,
  pub id: u32,
  pub params: usize,
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

  //fn process(&mut self, left: &mut [f32], right: &mut [f32], events: Vec<Event>) {}

  fn clock(&mut self) -> (f32, f32) {(0.0,0.0)}

  fn events(&mut self, events: Vec<Event>) {}
  //fn event(&mut self, event: Event) {}

  fn note_on(&mut self, note: u8, vel: u8) {}
  fn note_off(&mut self, note: u8) {}

  fn set_param(&mut self, index: usize, value: f32) {}
  fn param_name(index: usize) -> String { format!("Parameter {}", index) }
  fn param_default(index: usize) -> f32 { 0.0f32 }
  fn param_label(index: usize, value: f32) -> String { format!("{}", value) }

  fn arch_change(&mut self, arch: Architecture) {}
}

pub struct SynthPlugin<T: Synth> {
  synth: T,
  params: Vec<f32>,
  events: Vec<Event>,
  arch: Architecture,
  editor: Editor,
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

  fn new (host: HostCallback) -> SynthPlugin<T> {
    let info = T::get_info();
    let mut synth = T::new();

    let arch = Architecture{sample_rate: 44000_f32};

    synth.arch_change(arch);

    let mut params = vec![0_f32; info.params];
    for i in 0..(info.params-1) {
      let value = T::param_default(i);
      synth.set_param(i, value);
      params[i] = value;
    }

    let editor = Editor::new(host);

    SynthPlugin{
      synth: synth,
      params: params,
      events: Vec::new(),
      arch: arch,
      editor: editor,
    }
  }

  fn can_be_automated(&self, _: i32) -> bool { true }

  fn get_parameter(&self, index: i32) -> f32 { self.params[index as usize] }

  fn set_parameter(&mut self, index: i32, value: f32) {
    self.params[index as usize] = value;
    self.synth.set_param(index as usize, value);
  }

  fn get_parameter_name(&self, index: i32) -> String { T::param_name(index as usize) }

  fn set_sample_rate(&mut self, rate: f32) {
    self.arch.sample_rate = rate;
    self.synth.arch_change(self.arch);
  }

  fn process(&mut self, buffer: AudioBuffer<f32>){
    let (_, mut outputs) = buffer.split();

    let (mut hd, mut tl) = outputs.split_at_mut(1);
    let left: &mut [f32] = hd[0];
    let right: &mut [f32] = tl[0];

    //let events = ::std::mem::replace(&mut self.events, Vec::new());

    let iterator = left.iter_mut().zip(right.iter_mut());

    //self.synth.events(events);

    /*
    // Quería hacer algo como:
    let mut last_event = 0;
    for event in self.events.drain() {
      let next_event = event.sample - last_event;
      for (lsample, rsample) iterator.take(next_event) {
        //LOLOLOL
      }
    }
    // Pero no puedo porque take consume el iterador
    */

    let mut events = self.events.drain(..);
    let mut last_event = events.next();
    for (i, (lsample, rsample)) in iterator.enumerate() {

      // NOTE: Esto está mal porque si hay dos eventos en el mismo sample
      // uno se va a dejar para el siguiente (aunque casi no se nota)
      match last_event.clone() {
        Some( Event{ sample, data } ) if (sample as usize)<=i => {
          match data[0] {
            0x90 => self.synth.note_on(data[1], data[2]),
            0x80 => self.synth.note_off(data[1]),
            _ => {}
          }
          last_event = events.next();
        }
        _ => {}
      }

      let (l, r) = self.synth.clock();
      *lsample = l;
      *rsample = r;
    }
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

  fn get_editor (&mut self) -> Option<&mut VstEditor> {
    Some(&mut self.editor)
  }
}

impl<T: Synth> Default for SynthPlugin<T> {
  fn default () -> SynthPlugin<T> {
    SynthPlugin::new(Default::default())
  }
}