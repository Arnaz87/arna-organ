
use ::arnaudio::editor::*;
use ::arnaudio::gui;

pub struct Gui {
  controls: Vec<(usize, Box<gui::widget::Control<Value=f32>>)>,
  //group: gui::widget::Group
  background: gui::Image
}

// Restricciones:
// - Cada push debe ser un índice mayor que el anterior
// - Solo puede haber un valor por cada índice
fn get_index<T>(vec: &Vec<(usize, T)>, index: usize) -> Option<&mut T> {
  unimplemented!()
}

impl Editor for Gui {
  fn size () -> (usize, usize) { (550, 330) }

  fn new (synth: Channel, win: gui::Handler) -> Gui {
    /*let background = gui::Image::load("assets/background.png").unwrap();
    let knob_img = gui::Image::load("assets/knob.png").unwrap();
    let drawbar_img = gui::Image::load("assets/drawbar-red.png").unwrap();

    macro_rules! knob {
      ($x:expr, $y:expr, $i:expr) => {
        ($i, Box::new(gui::widget::Slider::new(
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
        )))
      }
    }

    macro_rules! drawbar {
      ($x:expr, $y:expr, $i:expr) => {
        ($i, Box::new(gui::widget::Slider::new(
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
        )))
      }
    }

    Gui {
      controls: vec![
        // Deben estar en orden de índice del parámetro.

        // Tonewheels
        knob!( 55, 225, 0),
        knob!( 95, 225, 2),
        knob!(135, 225, 3),
        // Click va en este grupo

        // Distort

        // Vibrato
        knob!(195, 30, 4),
        knob!(155, 30, 5),
        knob!(235, 30, 6),

        // Leslie

        // Room
        knob!(475, 150, 14),
        knob!(475, 110, 15),
        knob!(475,  70, 18), // Decay, por ahora diff2, o delay (?
        knob!(475,  30, 19),

        knob!(175, 225, 20), // Click va en el primer grupo

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
      background: background,
    }*/

    Gui {
      controls: vec![],
      background: unsafe { ::std::mem::uninitialized() }
    }
  }

  fn set_param (&mut self, index: usize, value: f32) {
    for &mut (i, ref mut control) in &mut self.controls {
      if i == index {
        // update_value, a diferencia de set_value, no ejecuta acciones
        // que cambian el estado fuera del componente, lo cual es lo que
        // queremos porque el valor del synth ya se modificó
        control.update_value(value);
      }
    }
  }
}

impl gui::Component for Gui {
  fn event (&mut self, ev: gui::Event) {
    for &mut (_, ref mut control) in &mut self.controls {
      control.event(ev);
    }
  }

  fn paint (&self, canvas: &mut gui::Canvas) {
    canvas.fill_image((0,0), &self.background);
    for &(_, ref control) in &self.controls {
      control.paint(canvas);
    }
  }
}


