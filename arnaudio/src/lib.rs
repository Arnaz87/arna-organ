
#[macro_use]
extern crate vst2;
extern crate libc;
extern crate user32;
extern crate winapi;

pub mod editor;
pub mod synth;
pub mod voice;

pub use vst2::{main as vstmain, api as vstapi};

// Esto es basado en el código de vst2, en el macro plugin_main!.
// Le quité lo del callback de windows y mac, no sé si sea tan necesario.
#[macro_export]
macro_rules! synth_main {
  ($t:ty) => {
    #[allow(non_snake_case)]
    #[no_mangle]
    pub extern "C" fn VSTPluginMain(callback: $crate::vstapi::HostCallbackProc) -> *mut $crate::vstapi::AEffect {
      $crate::vstmain::<$crate::synth::SynthPlugin<$t>>(callback)
    }
  }
}