use core::time;
#[cfg(feature = "use_gl")]
use glfw::{Action, Context};
#[cfg(feature = "use_gl")]
use image::{ImageBuffer, Rgba};
use log::{debug, error, info};
use serde_json::json;
use std::fs::OpenOptions;

#[cfg(feature = "use_sfml")]
use sfml::graphics::{Color, RenderTarget, RenderWindow};

#[cfg(feature = "use_sfml")]
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

use crate::apu::{Apu, CPU_FREQUENCY};
use crate::bus::main_bus::MainBus;
use crate::bus::message_bus::{Message, MessageBus};
use crate::bus::picture_bus::PictureBus;
use crate::cartridge::Cartridge;
use crate::controller::key_binding_parser::KeyType;
use crate::cpu::{Cpu, InterruptType};
use crate::mapper::factory;
use crate::ppu::{Ppu, SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES};

#[cfg(feature = "use_gl")]
use crate::render::gl_helper;
#[cfg(feature = "use_gl")]
use crate::render::glfw_window;
#[cfg(feature = "use_sfml")]
use crate::render::virtual_screen::VirtualScreen;

const NES_VIDEO_WIDTH: u32 = SCANLINE_VISIBLE_DOTS as u32;
const NES_VIDEO_HEIGHT: u32 = VISIBLE_SCANLINES as u32;
const CPU_CYCLE_DURATION: Duration = Duration::from_nanos(559);

pub const APP_NAME: &str = "NES-Simulator";

const FRAME_DURATION: Duration = time::Duration::from_millis(16);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RunningStatus {
  Running = 0,
  Pause = 1,
  LostFocus = 2,
  PauseAndLostFocus = 3,
}

impl From<u8> for RunningStatus {
  fn from(v: u8) -> Self {
    match v {
      0 => Self::Running,
      1 => Self::Pause,
      2 => Self::LostFocus,
      3 => Self::PauseAndLostFocus,
      _ => panic!(""),
    }
  }
}
impl RunningStatus {
  pub(crate) fn unpause(&mut self) {
    *self = Self::from(*self as u8 & Self::LostFocus as u8)
  }

  pub(crate) fn pause(&mut self) {
    *self = Self::from(*self as u8 | Self::Pause as u8)
  }

  pub(crate) fn is_pausing(&self) -> bool {
    (*self as u8 & Self::Pause as u8) != 0
  }

  pub(crate) fn unfocus(&mut self) {
    *self = Self::from(*self as u8 & Self::Pause as u8);
  }

  pub(crate) fn focus(&mut self) {
    *self = Self::from(*self as u8 | Self::LostFocus as u8);
  }

  pub(crate) fn is_focusing(&self) -> bool {
    (*self as u8 & Self::LostFocus as u8) == 0
  }
}

struct Instance {
  pub(crate) apu: Rc<RefCell<Apu>>,
  pub(crate) cpu: Cpu,
  pub(crate) ppu: Rc<RefCell<Ppu>>,
  pub(crate) stat: RunningStatus,
  pub(crate) cycle_timer: Instant,
  pub(crate) elapsed_time: Duration,
}

type MatrixType = HashMap<&'static str, u128>;

impl Instance {
  pub(crate) fn new(apu: Rc<RefCell<Apu>>, cpu: Cpu, ppu: Rc<RefCell<Ppu>>) -> Self {
    Self {
      apu,
      cpu,
      ppu,
      stat: RunningStatus::Running,
      cycle_timer: Instant::now(),
      elapsed_time: Duration::new(0, 0),
    }
  }

  pub(crate) fn toggle_pause(&mut self) {
    if self.stat.is_focusing() {
      self.cycle_timer = Instant::now();
      self.stat.unpause();
    } else {
      self.stat.pause();
    }
  }

  pub(crate) fn can_run(&self) -> bool {
    self.stat.is_focusing() && !self.stat.is_pausing()
  }

  pub(crate) fn update_timer(&mut self) {
    let now = Instant::now();
    self.elapsed_time += now - self.cycle_timer;
    self.cycle_timer = now;
  }

  pub(crate) fn do_save(&self, file: &String) {
    match save(self, file) {
      Ok(_) => info!("save success"),
      Err(e) => error!("save failed: {}", e),
    }
  }
}
pub struct Emulator {
  #[cfg(feature = "use_sfml")]
  emulator_screen: VirtualScreen,
  #[cfg(feature = "use_sfml")]
  window: RenderWindow,

  save_path: String,
  screen_scale: f32,
  #[cfg(feature = "use_gl")]
  rgba: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
  message_bus: Rc<RefCell<MessageBus>>,
  ctl1: Vec<KeyType>,
  ctl2: Vec<KeyType>,
}

impl Emulator {
  pub fn new(screen_scale: f32, save_path: String, ctl1: Vec<KeyType>, ctl2: Vec<KeyType>) -> Self {
    let message_bus = Rc::new(RefCell::new(MessageBus::new()));

    #[cfg(feature = "use_sfml")]
    let (video_mode, emulator_screen) = {
      let width = (NES_VIDEO_WIDTH as f32 * screen_scale) as u32;
      let height = (NES_VIDEO_HEIGHT as f32 * screen_scale) as u32;
      let video_mode = VideoMode::new(width, height, 32);

      let mut emulator_screen = VirtualScreen::new();

      emulator_screen.create(
        NES_VIDEO_WIDTH,
        NES_VIDEO_HEIGHT,
        screen_scale,
        Color::WHITE,
      );
      (video_mode, emulator_screen)
    };

    Self {
      #[cfg(feature = "use_sfml")]
      emulator_screen,
      #[cfg(feature = "use_sfml")]
      window: RenderWindow::new(
        video_mode,
        APP_NAME,
        Style::TITLEBAR | Style::CLOSE,
        &ContextSettings::default(),
      ),

      screen_scale,

      #[cfg(feature = "use_gl")]
      rgba: None,
      save_path,
      message_bus,
      ctl1,
      ctl2,
    }
  }

  fn load(
    &self,
    #[cfg(feature = "use_gl")] window: Rc<RefCell<glfw::Window>>,
  ) -> Result<Instance, std::io::Error> {
    let file = OpenOptions::new().read(true).open(&self.save_path)?;

    let json_obj: serde_json::Value = serde_json::from_reader(file)?;
    fn str_mapper(json_value: &serde_json::Value) -> &str {
      json_value.as_str().unwrap()
    }
    let mut cpu: Cpu = json_obj
      .get("cpu")
      .map(str_mapper)
      .map(|cpu_str| serde_json::from_str(cpu_str).unwrap())
      .unwrap();
    let apu = json_obj
      .get("apu")
      .map(str_mapper)
      .map(|apu_str| {
        let mut apu: Apu = serde_json::from_str(apu_str).unwrap();
        apu.start();
        Rc::new(RefCell::new(apu))
      })
      .unwrap();
    let ppu = json_obj
      .get("ppu")
      .map(str_mapper)
      .map(|ppu_str| {
        let mut ppu: Ppu = serde_json::from_str(ppu_str).unwrap();
        ppu.set_message_bus(self.message_bus.clone());
        Rc::new(RefCell::new(ppu))
      })
      .unwrap();
    json_obj.get("main_bus").map(|main_bus| {
      let mut main_bus = MainBus::load(main_bus, ppu.clone(), apu.clone());
      main_bus.set_controller_keys(self.ctl1.clone(), self.ctl2.clone());
      #[cfg(feature = "use_gl")]
      main_bus.set_window(window);
      cpu.set_main_bus(main_bus);
    });
    Ok(Instance::new(apu, cpu, ppu))
  }

  fn consume_message(&mut self, cpu: &mut Cpu) {
    for message in self.message_bus.take().into_iter() {
      match message {
        Message::CpuInterrupt => {
          cpu.interrupt(InterruptType::NMI);
        }
        Message::PpuRender(frame) => {
          #[cfg(feature = "use_sfml")]
          self.emulator_screen.set_picture(frame.clone());
          #[cfg(feature = "use_gl")]
          {
            self.rgba = Some(frame);
          }
        }
      };
    }
  }

  fn step(&mut self, instance: &mut Instance, matrix: &mut MatrixType) {
    let mut now = Instant::now();
    {
      let mut ppu = instance.ppu.borrow_mut();
      ppu.step();
      ppu.step();
      ppu.step();
    }
    self.consume_message(&mut instance.cpu);

    sample_profile(&mut now, "ppu", matrix);

    instance.cpu.step();
    sample_profile(&mut now, "cpu", matrix);

    instance.apu.borrow_mut().step();
    sample_profile(&mut now, "apu", matrix);
  }

  fn init_rom(
    &self,
    rom_path: &str,
    #[cfg(feature = "use_gl")] window: Rc<RefCell<glfw::Window>>,
  ) -> Option<Instance> {
    let ppu = Rc::new(RefCell::new(Ppu::new(
      PictureBus::new(),
      self.message_bus.clone(),
    )));
    let apu = Rc::new(RefCell::new(Apu::new()));
    let mut main_bus = MainBus::new(apu.clone(), ppu.clone());
    main_bus.set_controller_keys(self.ctl1.clone(), self.ctl2.clone());
    #[cfg(feature = "use_gl")]
    main_bus.set_window(window);

    let mut cpu = Cpu::new(main_bus);
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_file(rom_path) {
      return None;
    }
    let mapper = factory::create_mapper(cartridge);
    cpu.main_bus_mut().set_mapper(mapper.clone());
    cpu.reset();
    ppu.borrow_mut().set_mapper_for_bus(mapper);
    // ppu.borrow_mut().reset();

    Some(Instance::new(apu, cpu, ppu))
  }

  fn one_frame(&mut self, instance: &mut Instance) -> Duration {
    let mut iter_time = 0;
    let mut matrix = MatrixType::default();
    while instance.elapsed_time > CPU_CYCLE_DURATION && iter_time < CPU_FREQUENCY {
      self.step(instance, &mut matrix);
      instance.elapsed_time -= CPU_CYCLE_DURATION;
      iter_time += 1;
    }
    let cost = Instant::now() - instance.cycle_timer;
    debug!(
      "last frame toke {:?} for {} times. 
        ppu total cost:{}, cpu total cost:{}, 
        apu total cost:{}",
      cost,
      iter_time,
      matrix["ppu"] / 1000,
      matrix["cpu"] / 1000,
      matrix["apu"] / 1000,
    );
    cost
  }

  #[cfg(feature = "use_gl")]
  pub fn run(&mut self, rom_path: &str) {
    let mut glfw = glfw_window::init_glfw();

    // glfw window creation
    // --------------------
    let width = NES_VIDEO_WIDTH as f32 * self.screen_scale;
    let height = NES_VIDEO_HEIGHT as f32 * self.screen_scale;
    let (window, events) = glfw_window::init_window_and_gl(&glfw, width as u32, height as u32);
    let window = Rc::new(RefCell::new(window));
    let mut instance = self
      .init_rom(rom_path, window.clone())
      .expect("Failed to load rom.");

    instance.apu.borrow_mut().start();
    #[allow(non_snake_case)]
    let (shader, VAO) = unsafe { (gl_helper::compile_shader(), gl_helper::create_vao()) };

    let texture = unsafe {
      let texture = gl_helper::create_texture();

      gl::BindTexture(gl::TEXTURE_2D, 0);
      gl::Uniform1i(
        gl::GetUniformLocation(shader, "texture".as_ptr() as *const i8),
        0,
      );
      texture
    };
    // Loop until the user closes the window

    while !window.borrow().should_close() {
      // Swap front and back buffers
      window.borrow_mut().swap_buffers();
      // Poll for and process events
      glfw.poll_events();
      for (_, event) in glfw::flush_messages(&events) {
        if !self.handle_event(event, window.clone(), &mut instance) {
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
        thread::sleep(time::Duration::from_millis(16));
      }
    }
  }

  #[cfg(feature = "use_gl")]
  fn handle_event(
    &mut self,
    event: glfw::WindowEvent,
    window: Rc<RefCell<glfw::Window>>,
    instance: &mut Instance,
  ) -> bool {
    match event {
      glfw::WindowEvent::Key(glfw::Key::Escape, _, Action::Press, _) => {
        window.borrow_mut().set_should_close(true);
        return false;
      }
      glfw::WindowEvent::Focus(focused) => {
        if focused {
          instance.stat.focus();
          instance.cycle_timer = Instant::now();
        } else {
          instance.stat.unfocus();
        }
      }
      glfw::WindowEvent::Key(glfw::Key::Z, _, Action::Press, _) => {
        instance.do_save(&self.save_path)
      }
      glfw::WindowEvent::Key(glfw::Key::X, _, Action::Press, _) => match self.load(window) {
        Ok(instance_load) => {
          *instance = instance_load;
          info!("load success")
        }
        Err(e) => error!("load failed: {}", e),
      },
      glfw::WindowEvent::Key(glfw::Key::F2, _, Action::Press, _) => instance.toggle_pause(),
      glfw::WindowEvent::Key(glfw::Key::F3, _, Action::Press, _) => {
        if instance.stat.is_pausing() {
          for _ in 0..29781 {
            let mut matrix = MatrixType::default();
            self.step(instance, &mut matrix);
          }
        }
      }
      glfw::WindowEvent::Key(glfw::Key::F4, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Debug);
        log::debug!("log switch into debug mode");
      }
      glfw::WindowEvent::Key(glfw::Key::F5, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Info);
        log::debug!("log switch into info mode");
      }
      glfw::WindowEvent::Key(glfw::Key::F6, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Warn);
        log::debug!("log switch into warn mode");
      }
      glfw::WindowEvent::Key(glfw::Key::F7, _, Action::Press, _) => {
        log::set_max_level(log::LevelFilter::Error);
        log::debug!("log switch into error mode");
      }
      _ => {}
    }
    true
  }

  #[cfg(feature = "use_sfml")]
  fn handle_event(&mut self, event: sfml::window::Event, instance: &mut Instance) -> bool {
    match event {
      Event::Closed
      | Event::KeyPressed {
        code: Key::ESCAPE, ..
      } => {
        self.window.close();
        return false;
      }
      Event::GainedFocus => {
        instance.stat.focus();
        instance.cycle_timer = Instant::now();
      }
      Event::LostFocus => instance.stat.unfocus(),
      Event::KeyPressed { code: Key::Z, .. } => instance.do_save(&self.save_path),
      Event::KeyPressed { code: Key::X, .. } => match self.load() {
        Ok(instance_load) => {
          *instance = instance_load;
          info!("load success")
        }
        Err(e) => error!("load failed: {}", e),
      },
      Event::KeyPressed { code: Key::F2, .. } => instance.toggle_pause(),
      Event::KeyPressed { code: Key::F3, .. } => {
        if instance.stat.is_pausing() {
          for _ in 0..29781 {
            let mut matrix = MatrixType::default();
            self.step(instance, &mut matrix);
          }
        }
      }
      Event::KeyPressed { code: Key::F4, .. } => {
        log::set_max_level(log::LevelFilter::Debug);
        log::debug!("log switch into debug mode");
      }
      Event::KeyPressed { code: Key::F5, .. } => {
        log::set_max_level(log::LevelFilter::Info);
        log::debug!("log switch into info mode");
      }
      Event::KeyPressed { code: Key::F6, .. } => {
        log::set_max_level(log::LevelFilter::Warn);
        log::debug!("log switch into warn mode");
      }
      Event::KeyPressed { code: Key::F7, .. } => {
        log::set_max_level(log::LevelFilter::Error);
        log::debug!("log switch into error mode");
      }
      _ => { /* Do nothing */ }
    }
    true
  }
  #[cfg(feature = "use_sfml")]
  pub fn run(&mut self, rom_path: &str) {
    let mut instance = self.init_rom(rom_path).expect("Failed to load rom.");
    self.window.set_vertical_sync_enabled(true);

    instance.apu.borrow_mut().start();
    while self.window.is_open() {
      while let Some(event) = self.window.poll_event() {
        if !self.handle_event(event, &mut instance) {
          break;
        }
      }
      self.window.draw(&self.emulator_screen);
      self.window.display();
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
}

fn sample_profile(start: &mut Instant, category: &'static str, matrix: &mut MatrixType) {
  let now = Instant::now();
  let duration = (now - *start).as_micros();
  matrix
    .entry(category)
    .and_modify(|e| *e += duration)
    .or_insert(duration);
  *start = now;
}

fn save(instance: &Instance, file: &String) -> Result<(), std::io::Error> {
  let path = std::path::Path::new(file);
  {
    let dir = path.parent().expect("invalid path");
    if !dir.exists() {
      std::fs::create_dir(dir)?;
    }
  }
  let mut file = std::fs::File::create(path)?;
  let json = json!({
    "apu": serde_json::to_string(&*instance.apu.borrow()).unwrap(),
    "cpu": serde_json::to_string(&instance.cpu).unwrap(),
    "ppu": serde_json::to_string(&*instance.ppu.borrow()).unwrap(),
    "main_bus" : instance.cpu.main_bus().save(),
  });
  file.write_all(json.to_string().as_bytes())
}
