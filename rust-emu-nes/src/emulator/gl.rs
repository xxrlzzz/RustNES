use std::thread;

use log::{error, info};
use rust_emu_common::instance::Instance;

use super::{Emulator, RuntimeConfig};

use crate::emulator::FRAME_DURATION;
use crate::instance::NESInstance;

use rust_emu_common::instant::Instant;

impl Emulator {
  pub fn run(&mut self, mut instance: Box<dyn Instance>) {
    use std::{cell::RefCell, rc::Rc};

    use glfw::Context;

    use crate::render::{gl, glfw_window};

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
    let (shader, VAO) = unsafe { (gl::compile_shader(), gl::create_vao()) };

    let texture = unsafe { gl::create_texture(shader) };

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
      if let Some(rgba) = instance.take_rgba() {
        unsafe {
          gl::set_texture(rgba);
          gl::draw_frame(shader, VAO, texture);
        }
      }

      if instance.can_run() {
        // instance.update_timer();
        self.update_timer();
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
    instance: &mut Box<dyn Instance>,
  ) -> bool {
    use glfw::{Action, WindowEvent};
    match event {
      WindowEvent::Key(glfw::Key::Escape, _, Action::Press, _) => {
        return false;
      }
      WindowEvent::Focus(focused) => {
        if focused {
          instance.focus();
          self.cycle_timer = Instant::now();
        } else {
          instance.unfocus();
        }
      }
      WindowEvent::Key(glfw::Key::Z, _, Action::Press, _) => {
        instance.do_save(&runtime_config.save_path)
      }
      WindowEvent::Key(glfw::Key::X, _, Action::Press, _) => {
        match NESInstance::load(&runtime_config) {
          Ok(instance_load) => {
            *instance = Box::new(instance_load);
            info!("load success")
          }
          Err(e) => error!("load failed: {}", e),
        }
      }
      WindowEvent::Key(glfw::Key::F2, _, Action::Press, _) => {
        instance.toggle_pause();
        self.cycle_timer = Instant::now();
      }
      WindowEvent::Key(glfw::Key::F3, _, Action::Press, _) => {
        if instance.is_pausing() {
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
