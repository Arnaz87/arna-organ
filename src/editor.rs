
use ::arnaudio::editor::*;
use ::arnaudio::gui;

pub struct Gui {
  group: gui::widget::Group
}

impl Editor for Gui {
  fn size () -> (usize, usize) { (550, 330) }

  fn new (synth: Channel, win: gui::Handler) -> Gui {
    let knob_img = gui::Image::load("assets/knob.png").unwrap();
    let drawbar_img = gui::Image::load("assets/drawbar-red.png").unwrap();

    macro_rules! knob {
      ($x:expr, $y:expr, $i:expr) => {
        Box::new(gui::widget::Slider::new(
          $x, 330 - ($y + 22), 22, 22,
          100.0,
          win.clone(),
          gui::widget::SliderStyle::Vertical,
          gui::widget::SeqPaint::new(
            knob_img.clone(),
            (0, 0), // Posición
            (22, 22), // Tamaño
            gui::widget::SeqDirection::Vertical,
            22, // Distancia
            49, // Cantidad
          ),
          {
            let synth = synth.clone();
            move |v: f32| {
              synth.set_param($i, v);
            }
          }
        ))
      }
    }

    macro_rules! drawbar {
      ($x:expr, $y:expr, $i:expr) => {
        Box::new(gui::widget::Slider::new(
          $x, 330 - ($y + 91), 16, 92,
          74.0,
          win.clone(),
          gui::widget::SliderStyle::VerticalInverse,
          gui::widget::SeqPaint::new(
            drawbar_img.clone(),
            (0, 0), // Posición
            (16, 92), // Tamaño
            gui::widget::SeqDirection::Horizontal,
            16, // Distancia
            9, // Cantidad
          ),
          {
            let synth = synth.clone();
            move |v: f32| {
              synth.set_param($i, v);
            }
          }
        ))
      }
    }

    Gui {
      group: gui::widget::Group {
        children: vec![
          // Tonewheels
          knob!( 55, 225, 0),
          knob!( 95, 225, 2),
          knob!(135, 225, 3),
          knob!(175, 225, 20), // Click

          // Distort

          // Vibrato
          knob!(155, 30, 5),
          knob!(195, 30, 4),
          knob!(235, 30, 6),

          // Leslie

          // Room
          knob!(475, 150, 14),
          knob!(475, 110, 15),
          knob!(475,  70, 18), // Decay, por ahora diff2, o delay (?
          knob!(475,  30, 19),

          // TW Drawbars
          drawbar!(55, 89, 21),
          drawbar!(80, 89, 22),
          drawbar!(105, 89, 23),
          drawbar!(130, 89, 24),
          drawbar!(155, 89, 25),
          drawbar!(180, 89, 26),
          drawbar!(205, 89, 27),
          drawbar!(230, 89, 28),
          //drawbar!(255, 89, 29),
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


