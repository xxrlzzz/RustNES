use log::debug;
use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::apu::Apu;
use crate::bus::main_bus::MainBus;
use crate::bus::message_bus::{Message, MessageBus};
use crate::bus::picture_bus::PictureBus;
use crate::cartridge::Cartridge;
use crate::cpu::{Cpu, InterruptType};
use crate::mapper::factory;
use crate::ppu::{Ppu, SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES};
use crate::virtual_screen::VirtualScreen;

const NES_VIDEO_WIDTH: u32 = SCANLINE_VISIBLE_DOTS as u32;
const NES_VIDEO_HEIGHT: u32 = VISIBLE_SCANLINES as u32;
const CPU_CYCLE_DURATION: Duration = Duration::from_nanos(559);

pub struct Emulator {
  cpu: Cpu,
  apu: Rc<RefCell<Apu>>,
  ppu: Rc<RefCell<Ppu>>,
  screen_scale: f32,
  emulator_screen: VirtualScreen,
  window: RenderWindow,

  matrix: HashMap<&'static str, u128>,

  message_bus: Rc<RefCell<MessageBus>>,
}

impl Emulator {
  pub fn new(scale: f32) -> Self {
    let message_bus = Rc::new(RefCell::new(MessageBus::new()));
    let ppu = Rc::new(RefCell::new(Ppu::new(
      PictureBus::new(),
      message_bus.clone(),
    )));
    let apu = Rc::new(RefCell::new(Apu::new()));
    let main_bus = Rc::new(RefCell::new(MainBus::new(ppu.clone(), apu.clone())));
    let video_mode = VideoMode::new(
      (NES_VIDEO_WIDTH as f32 * scale) as u32,
      (NES_VIDEO_HEIGHT as f32 * scale) as u32,
      32,
    );
    Self {
      cpu: Cpu::new(main_bus),
      apu,
      ppu,
      screen_scale: scale,
      emulator_screen: VirtualScreen::new(),
      window: RenderWindow::new(
        video_mode,
        "NES-Simulator",
        Style::TITLEBAR | Style::CLOSE,
        &ContextSettings::default(),
      ),
      matrix: HashMap::default(),
      message_bus: message_bus,
    }
  }

  fn consume_message(&mut self) {
    loop {
      let message = self.message_bus.borrow_mut().peek();
      match message {
        None => break,
        Some(message) => {
          match message {
            Message::CpuInterrupt => {
              self.cpu.interrupt(InterruptType::NMI);
            }
            Message::PpuRender(frame) => {
              self.emulator_screen.set_picture(&frame);
            }
          };
          self.message_bus.borrow_mut().pop();
        }
      }
    }
  }

  fn sample_profile(&mut self, start: Instant, end: Instant, category: &'static str) {
    let duration = (end - start).as_micros();
    self
      .matrix
      .entry(category)
      .and_modify(|e| *e += duration)
      .or_insert(duration);
  }

  pub fn step(&mut self) {
    let mut now = Instant::now();
    {
      let mut ppu = self.ppu.borrow_mut();
      ppu.step();
      ppu.step();
      ppu.step();
    }
    self.consume_message();

    let mut nxt = Instant::now();
    self.sample_profile(now, nxt, "ppu");

    now = nxt;
    self.cpu.step();
    nxt = Instant::now();
    self.sample_profile(now, nxt, "cpu");

    now = nxt;
    self.apu.borrow_mut().step();
    nxt = Instant::now();
    self.sample_profile(now, nxt, "apu");
  }

  fn init_rom(&mut self, rom_path: &str) {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_file(rom_path) {
      return;
    }
    let mapper = factory::create_mapper(cartridge);
    self.cpu.main_bus().borrow_mut().set_mapper(mapper.clone());
    self.cpu.reset();
    self.ppu.borrow_mut().set_mapper_for_bus(mapper.clone());
    self.ppu.borrow_mut().reset();
  }

  fn init_screen(&mut self) {
    self.window.set_vertical_sync_enabled(true);
    self.emulator_screen.create(
      NES_VIDEO_WIDTH,
      NES_VIDEO_HEIGHT,
      self.screen_scale,
      Color::WHITE,
    );
  }

  fn one_frame(&mut self, start_timer: Instant, elapsed_time: &mut Duration) {
    let mut iter_time = 0;
    while *elapsed_time > CPU_CYCLE_DURATION && iter_time < 1788908 {
      self.step();
      *elapsed_time -= CPU_CYCLE_DURATION;
      iter_time += 1;
    }
    let start = Instant::now();
    self.window.draw(&self.emulator_screen);
    self.window.display();
    let end = Instant::now();
    debug!(
      "last frame toke {:?} for {} times. 
        ppu total cost:{}, cpu total cost:{}, 
        apu total cost:{}, render cost: {}",
      Instant::now() - start_timer,
      iter_time,
      self.matrix["ppu"] / 1000,
      self.matrix["cpu"] / 1000,
      self.matrix["apu"] / 1000,
      (end - start).as_millis(),
    );
    self.matrix.clear();
  }

  pub fn run(&mut self, rom_path: &str) {
    self.init_rom(rom_path);
    self.init_screen();
    self.apu.borrow_mut().start();
    let mut focus = true;
    let mut pause = false;
    let mut cycle_timer = Instant::now();
    let mut elapsed_time = cycle_timer - cycle_timer;
    while self.window.is_open() {
      while let Some(event) = self.window.poll_event() {
        match event {
          Event::Closed
          | Event::KeyPressed {
            code: Key::ESCAPE, ..
          } => {
            self.window.close();
            return;
          }
          Event::GainedFocus => {
            focus = true;
            cycle_timer = Instant::now();
          }
          Event::LostFocus => {
            focus = false;
          }
          Event::KeyPressed { code: Key::F2, .. } => {
            pause = !pause;
            if !pause {
              cycle_timer = Instant::now();
            }
          }
          Event::KeyReleased { code: Key::F3, .. } => {
            if pause {
              for _ in 0..29781 {
                self.step();
              }
            }
          }
          Event::KeyReleased { code: Key::F4, .. } => {
            log::set_max_level(log::LevelFilter::Debug);
            log::debug!("log switch into debug mode");
          }
          Event::KeyReleased { code: Key::F5, .. } => {
            log::set_max_level(log::LevelFilter::Info);
            log::info!("log switch into info mode");
          }
          Event::KeyReleased { code: Key::F6, .. } => {
            log::set_max_level(log::LevelFilter::Warn);
            log::warn!("log switch into warn mode");
          }
          Event::KeyReleased { code: Key::F7, .. } => {
            log::set_max_level(log::LevelFilter::Error);
            log::error!("log switch into error mode");
          }
          _ => { /* Do nothing */ }
        }
      }
      if focus && !pause {
        let now = Instant::now();
        elapsed_time += now - cycle_timer;
        cycle_timer = now;
        self.one_frame(now, &mut elapsed_time);
      } else {
        sfml::system::sleep(sfml::system::Time::milliseconds(1000 / 60));
      }
    }
  }

  pub fn set_keys(&mut self, p1: Vec<Key>, p2: Vec<Key>) {
    self.cpu.main_bus().borrow_mut().set_controller_keys(p1, p2);
  }
}
