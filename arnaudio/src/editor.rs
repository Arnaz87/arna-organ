use vst2::editor::{Editor as VstEditor};
use vst2::plugin::{HostCallback};
use vst2::host::Host;

use std::sync::{Arc, Mutex, mpsc};

use ParamEvent;

#[derive(Clone)]
pub struct Channel {
  pub params: Arc<Mutex< Vec<f32> >>,
  pub sender: mpsc::Sender<ParamEvent>,
  pub host: Arc<Mutex<HostCallback>>,
}

impl Channel {
  pub fn set_param (&self, index: usize, value: f32) {
    self.sender.send(ParamEvent{index: index, value: value});
    self.params.lock().unwrap()[index] = value;
    self.host.lock().unwrap().automate(index as i32, value as f32);
  }
}

pub trait Editor : ::gui::Component + 'static {
  fn new (synth: Channel, win: ::gui::Handler) -> Self;
  fn size () -> (usize, usize);
  fn set_param (&mut self, index: usize, value: f32);
}

pub struct PluginEditor <T: Editor> {
  width: usize,
  height: usize,
  isopen: bool,
  handler: ::gui::Handler,
  editor: Arc<Mutex<T>>,

  // Esto es para que el compilador no se queje de que no uso T,
  // pero en realidad sí uso T, en new
  phantom: ::std::marker::PhantomData<T>
}

impl<T: Editor> PluginEditor<T> {
  pub fn new (channel: Channel) -> PluginEditor<T> {

    let handler = ::gui::Handler::new();

    let editor = T::new(channel, handler.clone());
    let arc = handler.attach(editor);

    let (width, height) = T::size();
    handler.set_size(width, height);

    PluginEditor {
      width: width,
      height: height,
      isopen: false,
      handler: handler,
      editor: arc,
      phantom: ::std::marker::PhantomData,
    }
  }

  pub fn set_param (&self, index: usize, value: f32) {
    self.editor.lock().unwrap().set_param(index, value);
  }
}

impl <T: Editor> VstEditor for PluginEditor<T> {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {
    self.handler.open(ptr as *mut ::std::os::raw::c_void);
    self.isopen = true;
  }

  fn close (&mut self) {
    self.handler.close();
    self.isopen = false;
  }
}