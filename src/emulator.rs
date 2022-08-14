use core::time;
use image::{ImageBuffer, Rgba};
use log::{error, info};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Texture};
use sdl2::surface::Surface;
use std::collections::HashSet;
use std::rc::Rc;

use std::cell::RefCell;
use std::thread;
use std::time::{Duration, Instant};

use crate::apu::CPU_FREQUENCY;
use crate::bus::message_bus::{Message, MessageBus};
use crate::common::{sample_profile, MatrixType};
use crate::controller::key_binding_parser::KeyType;
use crate::cpu::{Cpu, InterruptType};
use crate::instance::Instance;
use crate::ppu::{SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES};
use crate::CONFIG;

const NES_VIDEO_WIDTH: u32 = SCANLINE_VISIBLE_DOTS as u32;
const NES_VIDEO_HEIGHT: u32 = VISIBLE_SCANLINES as u32;
const CPU_CYCLE_DURATION: Duration = Duration::from_nanos(559);

pub const APP_NAME: &str = "NES-Simulator";

const FRAME_DURATION: Duration = time::Duration::from_millis(16);

#[derive(Clone)]
pub struct RuntimeConfig {
  pub save_path: String,
  pub screen_scale: f32,

  pub ctl1: Vec<KeyType>,
  pub ctl2: Vec<KeyType>,
}

impl RuntimeConfig {
  pub fn window_size(&self) -> (u32, u32) {
    let width = (NES_VIDEO_WIDTH as f32 * self.screen_scale) as u32;
    let height = (NES_VIDEO_HEIGHT as f32 * self.screen_scale) as u32;
    (width, height)
  }
}

pub struct Emulator {
  rgba: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
  message_bus: Rc<RefCell<MessageBus>>,
  runtime_config: RuntimeConfig,
}

impl Emulator {
  pub fn new(screen_scale: f32, save_path: String, ctl1: Vec<KeyType>, ctl2: Vec<KeyType>) -> Self {
    let message_bus = Rc::new(RefCell::new(MessageBus::new()));

    Self {
      runtime_config: RuntimeConfig {
        save_path,
        screen_scale,
        ctl1,
        ctl2,
      },

      rgba: None,
      message_bus,
    }
  }

  fn consume_message(&mut self, cpu: &mut Cpu) {
    for message in self.message_bus.take().into_iter() {
      match message {
        Message::CpuInterrupt => {
          cpu.interrupt(InterruptType::NMI);
        }
        Message::PpuRender(frame) => {
          self.rgba = Some(frame);
        }
      };
    }
  }

  fn one_frame(&mut self, instance: &mut Instance) -> Duration {
    let mut iter_time = 0;
    let mut matrix = MatrixType::default();
    instance.update_timer();
    for i in 0..CPU_FREQUENCY {
      instance.step(&mut matrix);
      iter_time = i;

      let mut now = Instant::now();
      self.consume_message(&mut instance.cpu);
      sample_profile(&mut now, "msg", &mut matrix);
      let cost = Instant::now() - instance.cycle_timer;
      if instance.elapsed_time < CPU_CYCLE_DURATION || cost > FRAME_DURATION {
        break;
      }
      instance.elapsed_time -= CPU_CYCLE_DURATION;
    }
    let cost = Instant::now() - instance.cycle_timer;
    if CONFIG.profile {
      info!(
        "last frame toke {:?} for {} times. 
          ppu total cost:{}, cpu total cost:{}, 
          apu total cost:{}, msg total cost:{}",
        cost,
        iter_time,
        matrix["ppu"] / 1000,
        matrix["cpu"] / 1000,
        matrix["apu"] / 1000,
        matrix["msg"] / 1000,
      );
    }

    cost
  }

  #[cfg(feature = "use_sdl2")]
  pub fn run(&mut self, mut instance: Instance) {
    let (width, height) = self.runtime_config.window_size();
    let runtime_config = self.runtime_config.clone();

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
      .window(APP_NAME, width, height)
      .position_centered()
      .allow_highdpi()
      .build()
      .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_blend_mode(BlendMode::None);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    let texture_creator = canvas.texture_creator();

    let surface =
      Surface::new(NES_VIDEO_WIDTH, NES_VIDEO_HEIGHT, PixelFormatEnum::ABGR8888).unwrap();
    let mut texture = texture_creator
      .create_texture_from_surface(surface)
      .unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    info!("start running");
    'running: loop {
      for event in event_pump.poll_iter() {
        if !self.handle_event(&runtime_config, event, &mut instance) {
          break 'running;
        }
      }
      let keyboard_status = event_pump.keyboard_state();
      let keycodes: HashSet<sdl2::keyboard::Keycode> = keyboard_status
        .pressed_scancodes()
        .flat_map(sdl2::keyboard::Keycode::from_scancode)
        .collect();
      unsafe {
        crate::controller::KEYBOARD_STATE = Some(keycodes);
      }

      // The rest of the game loop goes here...
      if self.rgba.is_some() {
        set_sdl2_texture(&mut texture, self.rgba.take().unwrap());

        // info!("update game screen");
        let _ = canvas.copy(&texture, None, None);
        canvas.present();
      }

      if instance.can_run() {
        let cost = self.one_frame(&mut instance);
        if FRAME_DURATION > cost {
          thread::sleep(FRAME_DURATION - cost);
        }
      } else {
        thread::sleep(FRAME_DURATION);
      }
    }
  }

  pub fn create_instance(&self, rom_path: &str) -> Instance {
    Instance::init_rom_from_path(rom_path, &self.runtime_config, self.message_bus.clone())
      .expect("Failed to load rom.")
  }

  pub fn create_instance_from_data(&self, rom_data: &[u8]) -> Instance {
    Instance::init_rom_from_data(rom_data, &self.runtime_config, self.message_bus.clone())
      .expect("Failed to load rom.")
  }

  #[cfg(feature = "use_gl")]
  pub fn run(&mut self, rom_path: &str) {
    let mut instance = self.create_instance(rom_path);

    let mut glfw = glfw_window::init_glfw();
    let runtime_config = self.runtime_config.clone();
    // glfw window creation
    // --------------------
    let (width, height) = runtime_config.window_size();
    let (window, events) = glfw_window::init_window_and_gl(&glfw, width, height);
    let window = Rc::new(RefCell::new(window));

    unsafe {
      WINDOW_INSTANCE = Some(window.clone());
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

      if self.rgba.is_some() {
        unsafe {
          gl_helper::set_texture(self.rgba.take().unwrap());
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
  }

  #[cfg(feature = "use_sdl2")]
  fn handle_event(
    &mut self,
    runtime_config: &RuntimeConfig,
    event: sdl2::event::Event,
    instance: &mut Instance,
  ) -> bool {
    use sdl2::event::{Event, WindowEvent};
    use sdl2::keyboard::Keycode;
    match event {
      Event::Quit { .. }
      | Event::KeyDown {
        keycode: Some(sdl2::keyboard::Keycode::Escape),
        ..
      } => return false,
      Event::Window {
        win_event: WindowEvent::FocusGained,
        ..
      } => {
        info!("window gain focus");
        instance.stat.focus();
      }
      Event::Window {
        win_event: WindowEvent::FocusLost,
        ..
      } => {
        info!("window lost focus");
        instance.stat.unfocus();
      }
      Event::KeyDown {
        keycode: Some(key), ..
      } => match key {
        Keycode::Z => instance.do_save(&runtime_config.save_path),
        Keycode::X => match Instance::load(&runtime_config, self.message_bus.clone()) {
          Ok(instance_load) => {
            *instance = instance_load;
            info!("load success")
          }
          Err(e) => error!("load failed: {}", e),
        },
        Keycode::F2 => instance.toggle_pause(),
        Keycode::F3 => {
          if instance.stat.is_pausing() {
            let mut matrix = MatrixType::default();
            for _ in 0..29781 {
              instance.step(&mut matrix);
              self.consume_message(&mut instance.cpu);
            }
          }
        }
        Keycode::F4 => {
          log::set_max_level(log::LevelFilter::Debug);
          log::debug!("log switch into debug mode");
        }
        Keycode::F5 => {
          log::set_max_level(log::LevelFilter::Info);
          log::debug!("log switch into info mode");
        }
        Keycode::F6 => {
          log::set_max_level(log::LevelFilter::Warn);
          log::debug!("log switch into warn mode");
        }
        Keycode::F7 => {
          log::set_max_level(log::LevelFilter::Error);
          log::debug!("log switch into error mode");
        }
        _ => {}
      },
      _ => {}
    };
    return true;
  }

  // handle glfw events
  #[cfg(feature = "use_gl")]
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
        match Instance::load(&runtime_config, self.message_bus.clone()) {
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
          let mut matrix = MatrixType::default();
          for _ in 0..29781 {
            instance.step(&mut matrix);
            self.consume_message(&mut instance.cpu);
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

fn set_sdl2_texture(texture: &mut Texture, rgba: ImageBuffer<Rgba<u8>, Vec<u8>>) {
  let width = rgba.width();
  let data = rgba.into_vec();
  texture
    .update(None, data.as_slice(), (width * 4) as _)
    .unwrap();
}
