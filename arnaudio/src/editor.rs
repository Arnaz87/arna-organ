use vst2::editor::{Editor as VstEditor};

pub struct Editor {
  width: u32,
  height: u32,
  handler: Option<::gui::Handler>
}

impl Editor {
  pub fn new () -> Editor {
    Editor {
      width: 300,
      height: 100,
      handler: None
    }
  }
}

impl VstEditor for Editor {
  fn size (&self) -> (i32, i32) { (self.width as i32, self.height as i32) }
  fn position (&self) -> (i32, i32) { (0, 0) }
  fn is_open (&mut self) -> bool {
    self.handler.clone().map_or(false, |h| h.is_open())
  }

  fn open (&mut self, ptr: *mut ::libc::c_void) {

    if self.is_open() { return; }

    let handler = match self.handler.clone() {
      None => {
        let handler = ::gui::Handler::new();

        let window = ::gui::widget::Window {
          size: (self.width, self.height),
          main: ::gui::widget::Group {
            children: vec![
              Box::new(::gui::widget::Slider::new(
                0, 0, 16, 80,
                handler.clone(),
                ::gui::widget::SliderStyle::Vertical,
                Box::new(::gui::widget::SeqPaint::new(
                  ::gui::Image::load("cknob.png").unwrap(),
                  44, // height
                  40, // count
                ))
              ))
            ],
            bg: match ::gui::Image::load("craft.png") {
              Some(img) => Some((0, 0, img)),
              None => None
            },
          }
        };
        handler.attach(window);
        self.handler = Some(handler.clone());
        handler
      },
      Some(handler) => handler
    };

    handler.open(
      ptr as *mut ::std::os::raw::c_void,
      self.width as i32,
      self.height as i32
    );
  }

  fn close (&mut self) {
    match self.handler.clone() {
      Some(handler) => {
        handler.close();
        self.handler = None;
      },
      None => {}
    }
  }
}