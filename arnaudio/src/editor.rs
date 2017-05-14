use vst2::editor::{Editor as VstEditor};
use vst2::plugin::{HostCallback};
use std::sync::{Arc, Mutex, mpsc};

use ParamEvent;

pub struct Editor {
  width: u32,
  height: u32,
  isopen: bool,
  handler: Option<::gui::Handler>,
  host: Arc<Mutex<HostCallback>>,
  sender: mpsc::Sender<ParamEvent>,
}

impl Editor {
  pub fn new (host: HostCallback, sender: mpsc::Sender<ParamEvent>) -> Editor {
    Editor {
      width: 300,
      height: 100,
      isopen: false,
      handler: None,
      host: Arc::new(Mutex::new(host)),
      sender: sender,
    }
  }
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool { self.isopen }

  fn open (&mut self, ptr: *mut ::libc::c_void) {
    let handler = match self.handler.clone() {
      None => {
        let handler = ::gui::Handler::new();

        let window = ::gui::widget::Group {
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
        };
        handler.attach(window);
        handler.set_size(self.width, self.height);

        self.handler = Some(handler.clone());
        handler
      },
      Some(handler) => handler
    };
    handler.open(ptr as *mut ::std::os::raw::c_void);
    self.isopen = true;
  }

  fn close (&mut self) {
    match self.handler.clone() {
      Some(handler) => {
        handler.close();
        self.handler = None;
      },
      None => {}
    }
    self.isopen = false;
  }
}