use wasm_bindgen::prelude::*;

use crate::controller::web_key;
use crate::emulator::Emulator;
use crate::instance::Instance;
#[cfg(not(feature = "wasm-miniapp"))]
use crate::render::webgl::GlWrapper;


#[wasm_bindgen]
pub fn wasm_main() -> Result<(), JsValue> {
  wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
  if web_sys::window().is_none() {
    log::error!("no window");
    return Err(JsValue::NULL);
  }
  web_key::keyboard_listen()?;

  Ok(())
}

#[wasm_bindgen]
pub struct WebNes {
  #[cfg(not(feature = "wasm-miniapp"))]
  pub(crate) gl_wrapper: GlWrapper,
  pub(crate) emulator: Emulator,
  pub audio_sample_rate: f32,
  pub(crate) instance: Instance,
}

#[wasm_bindgen]
impl WebNes {
  pub fn default() -> Self {
    let data = include_bytes!("../../assets/mario.nes");
    let ele = "canvas";
    let audio_sample_rate = 44100.0;
    let data = js_sys::Uint8Array::from(data.as_ref());

    #[cfg(not(feature = "wasm-miniapp"))]
    {
      let ele = JsValue::from_str(ele);
      WebNes::new(data, ele, audio_sample_rate).unwrap()
    }
    #[cfg(feature = "wasm-miniapp")]
    {
      WebNes::new(data, audio_sample_rate).unwrap()
    }
  }

  pub fn audio_callback(&mut self, buf_size: usize, out: &mut [f32]) {
    let audio = self
      .instance
      .apu
      .as_ref()
      .lock()
      .unwrap()
      .audio_frame(buf_size);
    match audio {
      Ok(audio) => {
        out.copy_from_slice(audio.as_slice());
      }
      Err(_e) => out.fill(0.0),
    }
  }
}
