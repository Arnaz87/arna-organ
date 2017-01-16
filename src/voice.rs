
use std::collections::vec_deque::VecDeque;

use synth::{Event as SynthEvent};

use std::iter::{Filter};
use std::slice::{Iter, IterMut};

const NUM_VOICES: usize = 8;

#[derive(Clone, Copy)]
struct Note {
  sample: u32,
  note: u8,
  vel: u8,
}

pub trait Voice {
  fn is_active(&self) -> bool;
  fn is_note(&self, note: u8) -> bool;
  fn note_on(&mut self, note: u8, vel: u8);
  fn note_off(&mut self);
}

#[derive(Default)]
pub struct Manager<T: Voice> {
  buffer: VecDeque<Note>,
  front: Option<Note>,

  sample: u32,
  voices: [T; NUM_VOICES],
}


impl<T: Voice> Manager<T> {
  pub fn add_events(&mut self, events: Vec<SynthEvent>) {
    self.buffer.clear();

    self.sample = 0;

    for event in events {
      match event {
        SynthEvent {sample, data} => {
          match data[0] {
            // Note on
            0x90 => {
              self.buffer.push_back(Note{
                sample: sample,
                note: data[1],
                vel: data[2],
              })
            },
            // Note off
            0x80 => {
              self.buffer.push_back(Note{
                sample: sample,
                note: data[1],
                vel: 0,
              })
            },
            _ => {}
          }
        }
      }
    }

    self.front = self.buffer.pop_front();
  }

  pub fn note_on(&mut self, note: u8, vel: u8) {
    for voice in self.voices.iter_mut() {
      if !voice.is_active() {
        voice.note_on(note, vel);
        break;
      }
    }
  }

  pub fn note_off(&mut self, note: u8) {
    // Apagar todas las voces que tocan esta nota
    for voice in self.voices.iter_mut() {
      if voice.is_note(note) { voice.note_off() }
    }
  }

  pub fn process_sample(&mut self) {
    use std::mem::replace;

    self.sample = self.sample + 1;

    let mut pop = false;
    match self.front.clone() {
      Some(ref mut event) => if event.sample <= self.sample {
        if event.vel == 0 {
          self.note_off(event.note);
        } else {
          self.note_on(event.note, event.vel);
        }
        pop = true;
      },
      None => {}
    }

    if pop {
      replace(&mut self.front, self.buffer.pop_front());
    }
  }
}

impl<'a, T: Voice> IntoIterator for &'a mut Manager<T> {
  type Item = &'a mut T;
  type IntoIter = Filter<IterMut<'a, T>, fn(&&mut T)->bool>;

  fn into_iter (self) ->  Self::IntoIter {
    fn voice_is_active<T: Voice> (voice: &&mut T) -> bool { voice.is_active() }
    self.voices.iter_mut().filter(voice_is_active)
  }
}

impl<'a, T: Voice> IntoIterator for &'a Manager<T> {
  type Item = &'a T;
  type IntoIter = Filter<Iter<'a, T>, fn(&& T)->bool>;

  fn into_iter (self) -> Self::IntoIter {
    fn voice_is_active<T: Voice> (voice: && T) -> bool { voice.is_active() }
    self.voices.iter().filter(voice_is_active)
  }
}

