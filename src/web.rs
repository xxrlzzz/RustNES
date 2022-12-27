use js_sys::JsString;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use std::cell::RefCell;
use std::ops::Add;
use std::rc::Rc;
use std::sync::Mutex;

use crate::controller::web_key;
use crate::emulator::Emulator;
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

fn request_animation_frame(f: &Closure<dyn FnMut()>) -> i32 {
  window()
    .request_animation_frame(f.as_ref().unchecked_ref())
    .expect("should register `requestAnimationFrame` OK")
}

lazy_static! {
  pub static ref EMULATOR: Mutex<Option<Emulator>> = Mutex::new(None);
  pub static ref HANDLE: Mutex<i32> = Mutex::new(0);
  pub static ref INSTANCE_CNT: Mutex<u32> = Mutex::new(0);
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn wasm_main() -> Result<(), JsValue> {
  wasm_logger::init(wasm_logger::Config::new(log::Level::Info));
  web_key::keyboard_listen()?;

  // emulator.gl_wrapper = Some(gl_wrapper);
  // EMULATOR.lock().unwrap().replace(emulator);
  Ok(())
}

#[wasm_bindgen]
pub fn start(data: js_sys::Uint8Array, ele: JsValue) -> Result<(), JsValue> {
  let data = data.to_vec();
  let ele = ele.dyn_ref::<JsString>().unwrap().as_string().unwrap();
  let v = {
    let v = INSTANCE_CNT.lock().unwrap().add(1);
    inner_start(data, ele, v)?;
    v
  };
  *INSTANCE_CNT.lock().unwrap() = v;
  log::info!("start {}", v);
  Ok(())
}

fn inner_start(data: Vec<u8>, ele: String, instance_id: u32) -> Result<(), JsValue> {
  // log::info!("data size: {}, mario size: {}", data.len(), mario.len());
  // Init data
  let (p1_key, p2_key) = crate::controller::key_binding_parser::default_key_binding();
  let mut emulator = crate::emulator::Emulator::new(2.0, ".".into(), p1_key, p2_key);

  let mut instance = emulator.create_instance_from_data(data.as_slice());
  // Create gl environment.
  let gl_wrapper = create_gl(ele.as_str())?;

  if !HANDLE.lock().unwrap().cmp(&0).is_eq() {
    let _ = window().cancel_animation_frame(*HANDLE.lock().unwrap());
  }

  // Animation loop.
  let f = Rc::new(RefCell::new(None));
  let g = f.clone();
  *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
    if INSTANCE_CNT.lock().unwrap().cmp(&instance_id).is_ne() {
      return;
    }
    // Simulation one frame and render result.
    let frame = emulator.frame(&mut instance);
    if frame.is_some() {
      // log::info!("render at {}", instance_id);
      let r = gl_wrapper.clone().render(frame.unwrap());
      if r.is_err() {
        return;
      }
    }
    let handle = request_animation_frame(f.borrow().as_ref().unwrap());
    *HANDLE.lock().unwrap() = handle;
  }) as Box<dyn FnMut()>));
  let handle = request_animation_frame(g.borrow().as_ref().unwrap());
  *HANDLE.lock().unwrap() = handle;
  Ok(())
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
