use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;

use crate::controller::key_binding_parser;
use crate::emulator::Emulator;

use super::wasm::WebNes;

impl WebNes {
  pub fn new(
    data: js_sys::Uint8Array,
    audio_sample_rate: f32,
  ) -> Result<WebNes, JsValue> {
    let data = data.to_vec();
    // Init data
    let (p1_key, p2_key) = key_binding_parser::default_key_binding();
    let emulator = Emulator::new(2.0, ".".into(), p1_key, p2_key);

    let instance = emulator.create_instance_from_data(data.as_slice());
    // Create gl environment.
    Ok(WebNes {
      emulator,
      audio_sample_rate,
      instance,
    })
  }

  pub fn do_frame_and_pull_data(&mut self) -> Result<Uint8Array, JsValue> {
    if let Some(frame) = self.emulator.frame(&mut self.instance) {
      unsafe {
        let array = js_sys::Uint8Array::view(frame.into_vec().as_slice());
        return Ok(array);
      }
    } else {
      log::debug!("no frame yet");
      return Ok(Uint8Array::default());
    }
  }
}