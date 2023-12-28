use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::controller::key_binding_parser;
use crate::emulator::Emulator;
use crate::render::webgl::GlWrapper;

use super::wasm::WebNes;


#[wasm_bindgen]
impl WebNes {
  pub fn new(
    data: js_sys::Uint8Array,
    ele: JsValue,
    audio_sample_rate: f32,
  ) -> Result<WebNes, JsValue> {
    let data = data.to_vec();
    // Init data
    let (p1_key, p2_key) = key_binding_parser::default_key_binding();
    let emulator = Emulator::new(2.0, ".".into(), p1_key, p2_key);

    let instance = emulator.create_instance_from_data(data.as_slice());
    // Create gl environment.
    let ctx = ele.dyn_into::<web_sys::WebGl2RenderingContext>()?;
    let gl_wrapper = GlWrapper::init_webgl(&ctx)?;
    Ok(WebNes {
      gl_wrapper,
      emulator,
      audio_sample_rate,
      instance,
    })
  }
  pub fn do_frame_and_draw(&mut self) {
    let frame = self.emulator.frame(&mut self.instance);
    if frame.is_some() {
      // log::info!("render at {}", instance_id);
      let r = self.gl_wrapper.render(frame.unwrap());
      if r.is_err() {
        log::warn!("render error: {:?}", r);
      }
    } else {
      log::debug!("no frame yet");
    }
  }
}
