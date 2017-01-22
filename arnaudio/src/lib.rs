
extern crate vst2;
extern crate libc;

pub extern crate gui;

pub mod synth;
pub mod voice;
pub mod editor;

//pub use gui;
pub use vst2::{main as vstmain, api as vstapi};

// Esto es básicamente copiado del código de vst2, de plugin_main!.
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