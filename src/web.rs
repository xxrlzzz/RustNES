use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::controller::key_binding_parser;
use crate::controller::web_key;
use crate::emulator::Emulator;
use crate::instance::Instance;
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
  gl_wrapper: GlWrapper,
  emulator: Emulator,
  pub audio_sample_rate: f32,
  instance: Instance,
}

#[wasm_bindgen]
impl WebNes {
  #[cfg(not(feature = "wasm-miniapp"))]
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

  #[cfg(feature = "wasm-miniapp")]
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

  pub fn default() -> Self {
    let data = include_bytes!("../assets/mario.nes");
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

  #[cfg(feature = "wasm-miniapp")]
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

  #[cfg(not(feature = "wasm-miniapp"))]
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
