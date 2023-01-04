use js_sys::JsString;
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
  web_key::keyboard_listen()?;

  Ok(())
}

#[wasm_bindgen]
pub struct WebNes {
  gl_wrapper: GlWrapper,
  emulator: Emulator,
  pub audio_sample_rate: f32,
  instance: Instance,
}

#[wasm_bindgen]
impl WebNes {
  pub fn new(
    data: js_sys::Uint8Array,
    ele: JsValue,
    audio_sample_rate: f32,
  ) -> Result<WebNes, JsValue> {
    let data = data.to_vec();
    let ele = ele.dyn_ref::<JsString>().unwrap().as_string().unwrap();
    // Init data
    let (p1_key, p2_key) = key_binding_parser::default_key_binding();
    let emulator = Emulator::new(2.0, ".".into(), p1_key, p2_key);

    let instance = emulator.create_instance_from_data(data.as_slice());
    // Create gl environment.
    let gl_wrapper = create_gl(ele.as_str())?;

    Ok(WebNes {
      gl_wrapper,
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
    let ele = JsValue::from_str(ele);
    WebNes::new(data, ele, audio_sample_rate).unwrap()
  }

  pub fn do_frame(&mut self) {
    let frame = self.emulator.frame(&mut self.instance);
    if frame.is_some() {
      // log::info!("render at {}", instance_id);
      let r = self.gl_wrapper.render(frame.unwrap());
      if r.is_err() {
        log::warn!("render error: {:?}", r);
      }
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

pub fn create_gl(ele: &str) -> Result<GlWrapper, JsValue> {
  let document = web_sys::window().unwrap().document().unwrap();
  let canvas = document.get_element_by_id(ele);
  if canvas.is_none() {
    return Err(JsValue::from_str("Canvas not found"));
  }
  let canvas = canvas.unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;

  let context = canvas
    .get_context("webgl2")?
    .unwrap()
    .dyn_into::<web_sys::WebGl2RenderingContext>()?;
  Ok(GlWrapper::init_webgl(&context)?)
}
