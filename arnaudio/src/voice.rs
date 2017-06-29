
const NUM_VOICES: usize = 8;

pub trait Voice : Default {
  fn is_active(&self) -> bool;
}

#[derive(Default)]
struct Container <T: Voice> {voice: T, note: u8, age: u16}

#[derive(Default)]
pub struct Manager<T: Voice> {
  voices: [Container<T>; NUM_VOICES],
}

impl<T: Voice> Manager<T> {

  // TODO: Si hay una voz tocando esta nota, debería devolver esa
  // misma voz, y que el caller se encarge del retrigger
  pub fn note_on (&mut self, note: u8) -> &mut T {

    let mut iter = self.voices.iter_mut();

    let mut exact = None;
    let mut best = iter.next().unwrap();
    best.age += 1;

    for curr in iter {
      // Debe hacerse al principio, para que todas las
      // voces tengan la misma edad al procesarse
      curr.age += 1;

      if curr.note == note {
        exact = Some(curr);
      } else if 
        !curr.voice.is_active() ||
        best.voice.is_active() &&
        curr.age > best.age
      { best = curr; }
    }

    let last = exact.unwrap_or(best);
    last.note = note;
    last.age = 0;
    &mut last.voice
  }

  pub fn note_off (&mut self, note: u8) -> Option<&mut T> {
    // Debería haber máximo una voz por nota
    self.voices.iter_mut().find(
      |ref cont| cont.voice.is_active() && cont.note == note
    ).map(|cont| &mut cont.voice)
  }

  pub fn iter<'a> (&'a self) -> impl Iterator<Item=&'a T> {
    self.voices.iter().filter_map(
      |ref cont| if cont.voice.is_active()
      { Some(&cont.voice) } else { None }
    )
  }

  pub fn iter_mut<'a> (&'a mut self) -> impl Iterator<Item=&'a mut T> {
    self.voices.iter_mut().filter_map(
      |mut cont| if cont.voice.is_active()
      { Some(&mut cont.voice) } else { None }
    )
  }
}