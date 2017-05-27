
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
    best.age += 1; // La primera no pasa por el bucle de arriba

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
    // Apagar todas las voces que tocan esta nota
    // En realidad, solo debería haber una sola voz por nota
    self.voices.iter_mut().find(
      |ref cont| cont.note == note
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

/*struct IterMut <'a, T: 'a + Voice> {
  pub iter: ::std::iter::Map
      <::std::iter::Filter
        <::std::slice::IterMut<'a, Container<T>>,
        fn (&&mut Container<T>) -> bool>,
      fn (&mut Container<T>) -> &'a mut T>,
}

impl <'a, T: 'a + Voice> Iterator for IterMut<'a, T> {
  type Item = &'a mut T;
  #[inline] fn next (&mut self) -> Option<Self::Item> { self.iter.next() }
  #[inline] fn size_hint (&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

impl<'a, T: Voice> IntoIterator for &'a mut Manager<T> {
  type Item = &'a mut T;
  //type IntoIter = Filter<IterMut<'a, T>, fn(&&mut T)->bool>;
  /*type IntoIter =
    Map
      <Filter
        <IterMut<'a, Container<T>>,
        fn (&&mut Container<T>) -> bool>,
      fn (&mut Container<T>) -> &'a mut T>;*/
  type IntoIter = FilterMap
    <IterMut<'a, Container<T>>,
    fn(&mut Container<T>) ->
      Option<&mut T>>;

  fn into_iter (self) ->  Self::IntoIter {
    fn mymap <T: Voice> (cont: &mut Container<T>) -> Option<&mut T> {
      if cont.v.is_active() { Some(&mut cont.v) } else { None }
    }
    self.voices.iter_mut().filter_map(mymap)
    /*fn is_active<T: Voice> (cont: &&mut Container<T>) -> bool { cont.v.is_active() }
    fn get_voice<T: Voice> (cont: &mut Container<T>) -> &mut T { &mut cont.v }
    self.voices.iter_mut().filter(is_active).map(get_voice)*/
  }
}*/

/*impl<'a, T: Voice> IntoIterator for &'a Manager<T> {
  type Item = &'a T;
  type IntoIter = Filter<Iter<'a, T>, fn(&& T)->bool>;

  fn into_iter (self) -> Self::IntoIter {
    fn voice_is_active<T: Voice> (cont: && Container<T>) -> bool { cont.v.is_active() }
    self.voices.iter().filter(voice_is_active)
  }
}*/

