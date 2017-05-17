
use ::arnaudio::editor::*;
use ::arnaudio::gui;

pub struct Gui {
  group: gui::widget::Group
}

impl Editor for Gui {
  fn size () -> (usize, usize) { (550, 330) }

  fn new (synth: Channel, win: gui::Handler) -> Gui {
    Gui {
      group: gui::widget::Group {
        children: vec![
          Box::new(gui::widget::Slider::new(
            55, 84, 22, 22,
            100.0,
            win.clone(),
            gui::widget::SliderStyle::Vertical,
            gui::widget::SeqPaint::new(
              gui::Image::load("assets/knob.png").unwrap(),
              22, // height
              48, // count
              // TODO: En realidad son 49, pero no sé por qué
              // no funciona si pongo eso
            ),
            {
              let synth = synth.clone();
              move |v: f32| {
                synth.set_param(21, v);
              }
            }
          ))
        ],
        bg: match gui::Image::load("assets/background.png") {
          Some(img) => Some((0, 0, img)),
          None => None
        },
      }
    }
  }
}

impl gui::Component for Gui {
  fn event (&mut self, ev: gui::Event) {
    self.group.event(ev);
  }

  fn paint (&self, canvas: &mut gui::Canvas) {
    self.group.paint(canvas);
  }
}


