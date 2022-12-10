use std::thread;
use std::time::Instant;

use log::{error, info};

use super::{Emulator, RuntimeConfig};

use crate::emulator::FRAME_DURATION;
use crate::instance::Instance;

impl Emulator {
  pub fn run(&mut self, mut instance: Instance) {
    use std::{cell::RefCell, rc::Rc};

    use glfw::Context;

    use crate::render::{gl_helper, glfw_window};

    // let mut instance = self.create_instance(rom_path);

    let mut glfw = glfw_window::init_glfw();
    let runtime_config = self.runtime_config.clone();
    // glfw window creation
    // --------------------
    let (width, height) = runtime_config.window_size();
    let (window, events) = glfw_window::init_window_and_gl(&glfw, width, height);
    let window = Rc::new(RefCell::new(window));

    unsafe {
      crate::controller::gl_key::WINDOW_INSTANCE = Some(window.clone());
    }

    #[allow(non_snake_case)]
    let (shader, VAO) = unsafe { (gl_helper::compile_shader(), gl_helper::create_vao()) };

    let texture = unsafe { gl_helper::create_texture(shader) };

    // Loop until the user closes the window
    while !window.borrow().should_close() {
      // Swap front and back buffers
      window.borrow_mut().swap_buffers();
      // Poll for and process events
      glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
        if !self.handle_event(&runtime_config, event, &mut instance) {
          window.borrow_mut().set_should_close(true);
          break;
        }
      }
      let mut rgba = instance.take_rgba();
      if rgba.is_some() {
        unsafe {
          gl_helper::set_texture(rgba.take().unwrap());
        }
      }

      unsafe {
        gl_helper::draw_frame(shader, VAO, texture);
      }
      if instance.can_run() {
        instance.update_timer();
        let cost = self.one_frame(&mut instance);
        if FRAME_DURATION > cost {
          thread::sleep(FRAME_DURATION - cost);
        }
      } else {
        thread::sleep(FRAME_DURATION);
      }
    }
    instance.stop();
  }
  fn handle_event(
    &mut self,
    runtime_config: &RuntimeConfig,
    event: glfw::WindowEvent,
    instance: &mut Instance,
  ) -> bool {
    use glfw::{Action, WindowEvent};
    match event {
      WindowEvent::Key(glfw::Key::Escape, _, Action::Press, _) => {
        return false;
      }
      WindowEvent::Focus(focused) => {
        if focused {
          instance.stat.focus();
          instance.cycle_timer = Instant::now();
        } else {
          instance.stat.unfocus();
        }
      }
      WindowEvent::Key(glfw::Key::Z, _, Action::Press, _) => {
        instance.do_save(&runtime_config.save_path)
      }
      WindowEvent::Key(glfw::Key::X, _, Action::Press, _) => {
        match Instance::load(&runtime_config) {
          Ok(instance_load) => {
            *instance = instance_load;
            info!("load success")
          }
          Err(e) => error!("load failed: {}", e),
        }
      }
      WindowEvent::Key(glfw::Key::F2, _, Action::Press, _) => instance.toggle_pause(),
      WindowEvent::Key(glfw::Key::F3, _, Action::Press, _) => {
        if instance.stat.is_pausing() {
          for _ in 0..29781 {
            instance.step();
          }
        }
      }
      WindowEvent::Key(glfw::Key::F4, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Debug);
        log::debug!("log switch into debug mode");
      }
      WindowEvent::Key(glfw::Key::F5, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Info);
        log::debug!("log switch into info mode");
      }
      WindowEvent::Key(glfw::Key::F6, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Warn);
        log::debug!("log switch into warn mode");
      }
      WindowEvent::Key(glfw::Key::F7, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Error);
        log::debug!("log switch into error mode");
      }
      _ => {}
    }
    true
  }
}
