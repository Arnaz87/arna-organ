use vst2::editor::{Editor as VstEditor};
use vst2::plugin::{HostCallback};
use std::sync::{Arc, Mutex, mpsc};

use ParamEvent;

#[derive(Clone)]
pub struct Channel {
  sender: mpsc::Sender<ParamEvent>,
  host: Arc<Mutex<HostCallback>>,
}

impl Channel {
  pub fn set_param (&self, index: usize, value: f32) {
    self.sender.send(ParamEvent{index: index, value: value});
  }
}

pub trait Editor : ::gui::Component + 'static {
  fn new (synth: Channel, win: ::gui::Handler) -> Self;
  fn size () -> (usize, usize);
}

pub struct PluginEditor <T: Editor> {
  width: usize,
  height: usize,
  isopen: bool,
  handler: ::gui::Handler,

  // Esto es para que el compilador no se queje de que no uso T,
  // pero en realidad sí uso T, en new
  phantom: ::std::marker::PhantomData<T>
}

impl<T: Editor> PluginEditor<T> {
  pub fn new (host: HostCallback, sender: mpsc::Sender<ParamEvent>) -> PluginEditor<T> {

    let handler = ::gui::Handler::new();
    let channel = Channel {
      host: Arc::new(Mutex::new(host)),
      sender: sender,
    };

    let editor = T::new(channel, handler.clone());
    handler.attach(editor);

    let (width, height) = T::size();
    handler.set_size(width, height);

    PluginEditor {
      width: width,
      height: height,
      isopen: false,
      handler: handler,
      phantom: ::std::marker::PhantomData,
    }
  }
}

impl <T: Editor> VstEditor for PluginEditor<T> {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {
    /*let handler = match self.handler.clone() {
      None => {
        let handler = ::gui::Handler::new();

        /*let window = ::gui::widget::Group {
          children: vec![
            Box::new(::gui::widget::Slider::new(
              10, 10, 44, 44,
              100.0,
              handler.clone(),
              ::gui::widget::SliderStyle::Vertical,
              ::gui::widget::SeqPaint::new(
                ::gui::Image::load("cknob.png").unwrap(),
                44, // height
                40, // count
              ),
              {
                let sender = self.sender.clone();
                move |v: f32| {
                  sender.send(ParamEvent{index: 21, value: v});
                }
              }
            ))
          ],
          bg: match ::gui::Image::load("craft.png") {
            Some(img) => Some((0, 0, img)),
            None => None
          },
        };*/

        let editor = T::new();
        handler.attach(window);
        handler.set_size(self.width, self.height);

        self.handler = Some(handler.clone());
        handler
      },
      Some(handler) => handler
    };*/
    self.handler.open(ptr as *mut ::std::os::raw::c_void);
    self.isopen = true;
  }

  fn close (&mut self) {
    self.handler.close();
    self.isopen = false;
  }
}