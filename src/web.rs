use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::rc::Rc;

use crate::controller::web_key;
use crate::render::webgl::GlWrapper;

#[wasm_bindgen]
extern "C" {
  // Use `js_namespace` here to bind `console.log(..)` instead of just
  // `log(..)`
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);

  // The `console.log` is quite polymorphic, so we can bind it with multiple
  // signatures. Note that we need to use `js_name` to ensure we always call
  // `log` in JS.
  #[wasm_bindgen(js_namespace = console, js_name = log)]
  fn log_u32(a: u32);
}

fn window() -> web_sys::Window {
  web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
  window()
    .request_animation_frame(f.as_ref().unchecked_ref())
    .expect("should register `requestAnimationFrame` OK");
}

#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn wasm_main() -> Result<(), JsValue> {
  wasm_logger::init(wasm_logger::Config::default());

  // Init data
  let (p1_key, p2_key) = crate::controller::key_binding_parser::default_key_binding();
  let mut emulator = crate::emulator::Emulator::new(2.0, ".".into(), p1_key, p2_key);
  let mario = include_bytes!("../assets/mario.nes");
  let mut instance = emulator.create_instance_from_data(mario);
  // Create gl environment.
  let gl_data = start()?;

  // Animation loop.
  let f = Rc::new(RefCell::new(None));
  let g = f.clone();
  *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
    // Simulation one frame and render result.
    let frame = emulator.frame(&mut instance);
    if frame.is_some() {
      log::info!("render frame");
      let r = gl_data.clone().render(frame.unwrap());
      if r.is_err() {
        return;
      }
    }
    request_animation_frame(f.borrow().as_ref().unwrap());
  }) as Box<dyn FnMut()>));
  request_animation_frame(g.borrow().as_ref().unwrap());
  Ok(())
}

pub fn start() -> Result<GlWrapper, JsValue> {
  web_key::keyboard_listen()?;
  let document = web_sys::window().unwrap().document().unwrap();
  let canvas = document
    .get_element_by_id("canvas")
    .unwrap()
    .dyn_into::<web_sys::HtmlCanvasElement>()?;

  let context = canvas
    .get_context("webgl2")?
    .unwrap()
    .dyn_into::<web_sys::WebGl2RenderingContext>()?;
  Ok(GlWrapper::init_webgl(&context)?)
}
