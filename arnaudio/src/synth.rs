
use vst2::plugin::{Info as VstInfo, Plugin, Category, HostCallback};
use vst2::editor::{Editor as VstEditor};
use vst2::buffer::AudioBuffer;
use vst2::event::{Event as VstEvent};

use vst2::plugin::CanDo;
use vst2::api::Supported;

use editor::PluginEditor;

use std::sync::{Arc, Mutex, MutexGuard, mpsc};

use ParamEvent;

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
pub trait Synth : Send {
  //const id: i32;

  type Editor: ::editor::Editor;

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

/*fn param_thread <T: Synth + 'static> (
  receiver: mpsc::Receiver<ParamEvent>,
  synth_mutex: Arc<Mutex<T>>
) {
  ::std::thread::spawn(move || {
    while let Ok(first) = receiver.recv() {
      let mut synth = synth_mutex.lock().unwrap();
      synth.set_param(first.index, first.value);

      while let Ok(ev) = receiver.try_recv() {
        synth.set_param(ev.index, ev.value);
      }
    }
  });
}*/

pub struct SynthPlugin<T: Synth> {
  synth: Arc<Mutex<T>>,
  params: Arc<Mutex< Vec<f32> >>,
  events: Vec<Event>,
  arch: Architecture,
  editor: PluginEditor<T::Editor>,
  sender: mpsc::Sender<ParamEvent>,
  receiver: mpsc::Receiver<ParamEvent>,
}

impl<T: Synth + 'static> Plugin for SynthPlugin<T> {
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

    let (sender, receiver) = mpsc::channel();

    let arch = Architecture{sample_rate: 44000_f32};

    let params = Arc::new(Mutex::new(vec![0_f32; info.params]));

    let editor = PluginEditor::new(::editor::Channel{
      host: Arc::new(Mutex::new(host)),
      sender: sender.clone(),
      params: params.clone(),
    });

    synth.arch_change(arch);

    {
      let mut params = params.lock().unwrap();

      for i in 0..(info.params-1) {
        let value = T::param_default(i);
        synth.set_param(i, value);
        editor.set_param(i, value);
        params[i] = value;
      }
    }

    let mutex = Arc::new(Mutex::new(synth));

    //param_thread(receiver, mutex.clone());

    SynthPlugin{
      synth: mutex,
      params: params,
      events: Vec::new(),
      arch: arch,
      editor: editor,
      sender: sender,
      receiver: receiver,
    }
  }

  fn can_be_automated(&self, _: i32) -> bool { true }

  fn get_parameter(&self, index: i32) -> f32 {
    self.params.lock().unwrap()[index as usize]
  }

  fn set_parameter(&mut self, index: i32, value: f32) {
    let index = index as usize;
    self.params.lock().unwrap()[index] = value;
    self.sender.send(ParamEvent{index: index, value: value});
    self.editor.set_param(index, value);
  }

  fn get_parameter_name(&self, index: i32) -> String { T::param_name(index as usize) }

  fn set_sample_rate(&mut self, rate: f32) {
    self.arch.sample_rate = rate;
    self.synth.lock().unwrap().arch_change(self.arch);
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
    // Pero no puedo porque take consume el iterador.
    // Así lo hace MDA Piano
    */

    let mut events = self.events.drain(..);

    let mut synth = self.synth.lock().unwrap();

    // NOTA: Esto no debería estar en el thread de audio, primero cualquier
    // thread puede cambiar parámetros en cualquier momento, y el
    // sintetizador debería actualizarce inmediatamente, y segundo algunos
    // parámetros son lentos para cambiarse, y no debería detenerse el thread
    // de audio para cambiarlos.
    // El primer problema se puede resolver si muevo esto dentro del bucle,
    // y reviso cambios en cada sample, pero empeoraría el segundo problema.

    if let Ok(first) = self.receiver.try_recv() {
      let mut params = self.params.lock().unwrap();

      params[first.index as usize] = first.value;
      synth.set_param(first.index, first.value);

      while let Ok(ev) = self.receiver.try_recv() {
        params[ev.index as usize] = ev.value;
        synth.set_param(ev.index, ev.value);
      }
    }

    let mut last_event = events.next();
    for (i, (lsample, rsample)) in iterator.enumerate() {

      // NOTE: Esto está mal porque si hay dos eventos en el mismo sample
      // uno se va a dejar para el siguiente (aunque casi no se nota)
      match last_event.clone() {
        Some( Event{ sample, data } ) if (sample as usize)<=i => {
          match data[0] {
            0x90 => synth.note_on(data[1], data[2]),
            0x80 => synth.note_off(data[1]),
            _ => {}
          }
          last_event = events.next();
        }
        _ => {}
      }

      let (l, r) = synth.clock();
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
    None
    //Some(&mut self.editor)
  }
}

impl<T: Synth + 'static> Default for SynthPlugin<T> {
  fn default () -> SynthPlugin<T> {
    SynthPlugin::new(Default::default())
  }
}