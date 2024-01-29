use std::sync::mpsc::Receiver;

use glfw::{Context, Glfw, Window, WindowEvent};

use crate::emulator::APP_NAME;

// glfw: initialize and configure
pub(crate) fn init_glfw() -> Glfw {
  // ------------------------------
  let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
  glfw.window_hint(glfw::WindowHint::ContextVersion(3, 3));
  glfw.window_hint(glfw::WindowHint::OpenGlProfile(
    glfw::OpenGlProfileHint::Core,
  ));
  #[cfg(target_os = "macos")]
  glfw.window_hint(glfw::WindowHint::OpenGlForwardCompat(true));

  glfw
}

pub(crate) fn init_window_and_gl(
  glfw: &Glfw,
  width: u32,
  height: u32,
) -> (Window, Receiver<(f64, WindowEvent)>) {
  let (mut window, events) = glfw
    .create_window(width, height, APP_NAME, glfw::WindowMode::Windowed)
    .expect("Failed to create GLFW window");

  window.make_current();
  window.set_key_polling(true);
  window.set_framebuffer_size_polling(true);

  gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

  (window, events)
}
