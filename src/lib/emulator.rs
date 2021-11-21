use log::info;
use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::cartridge::Cartridge;
use crate::cpu::{Cpu, InterruptType};
use crate::main_bus::*;
use crate::mapper::factory;
use crate::picture_bus::PictureBus;
use crate::ppu::{Ppu, SCANLINE_VISIBLE_DOTS, VISIBLE_SCANLINES};
use crate::virtual_screen::VirtualScreen;

const NES_VIDEO_WIDTH: u32 = SCANLINE_VISIBLE_DOTS as u32;
const NES_VIDEO_HEIGHT: u32 = VISIBLE_SCANLINES as u32;
const DEFAULT_SCREEN_SCALE: f32 = 2.;
const CPU_CYCLE_DURATION: Duration = Duration::from_nanos(559);

pub struct Emulator {
  cpu: Rc<RefCell<Cpu>>,
  ppu: Rc<RefCell<Ppu>>,
  screen_scale: f32,
  emulator_screen: Rc<RefCell<VirtualScreen>>,
  window: RenderWindow,
}

impl Emulator {
  pub fn new() -> Self {
    let emulator_screen = Rc::new(RefCell::new(VirtualScreen::new()));
    let ppu = Rc::new(RefCell::new(Ppu::new(
      PictureBus::new(),
      emulator_screen.clone(),
    )));
    let main_bus = Rc::new(RefCell::new(MainBus::new()));
    let cpu = Rc::new(RefCell::new(Cpu::new(main_bus.clone())));
    main_bus.clone().borrow_mut().set_ppu(ppu.clone());

    let video_mode = VideoMode::new(
      (NES_VIDEO_WIDTH as f32 * DEFAULT_SCREEN_SCALE) as u32,
      (NES_VIDEO_HEIGHT as f32 * DEFAULT_SCREEN_SCALE) as u32,
      32,
    );
    let video_style = Style::TITLEBAR | Style::CLOSE;
    Self {
      cpu: cpu,
      ppu: ppu,
      screen_scale: DEFAULT_SCREEN_SCALE,
      emulator_screen: emulator_screen,
      window: RenderWindow::new(
        video_mode,
        "NES-Simulator",
        video_style,
        &ContextSettings::default(),
      ),
    }
  }

  pub fn step(&mut self) {
    self.ppu.borrow_mut().step();
    if self.ppu.borrow_mut().check_and_reset_interrupt() {
      self.cpu.borrow_mut().interrupt(InterruptType::NMI);
    }
    self.ppu.borrow_mut().step();
    if self.ppu.borrow_mut().check_and_reset_interrupt() {
      self.cpu.borrow_mut().interrupt(InterruptType::NMI);
    }
    self.ppu.borrow_mut().step();
    if self.ppu.borrow_mut().check_and_reset_interrupt() {
      self.cpu.borrow_mut().interrupt(InterruptType::NMI);
    }
    self.cpu.borrow_mut().step();
  }

  pub fn run(&mut self, rom_path: &str) {
    let mut cartridge = Cartridge::new();
    if !cartridge.load_from_file(rom_path) {
      return;
    }
    let mapper = factory::create_mapper(
      cartridge,
      Box::new(|| {
        // TDDO
      }),
    );
    self
      .cpu
      .borrow_mut()
      .main_bus()
      .borrow_mut()
      .set_mapper(mapper.clone());
    self
      .ppu
      .borrow_mut()
      .picture_bus()
      .borrow_mut()
      .set_mapper(mapper.clone());
    self.cpu.borrow_mut().reset();
    self.ppu.borrow_mut().reset();

    self.window.set_vertical_sync_enabled(true);
    self.emulator_screen.borrow_mut().create(
      NES_VIDEO_WIDTH,
      NES_VIDEO_HEIGHT,
      self.screen_scale,
      Color::WHITE,
    );
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
            log::set_max_level(log::LevelFilter::Info);
          }
          Event::KeyReleased { code: Key::F5, .. } => {
            log::set_max_level(log::LevelFilter::Warn);
          }
          Event::KeyReleased { code: Key::F6, .. } => {
            log::set_max_level(log::LevelFilter::Error);
          }
          _ => { /* Do nothing */ }
        }
      }
      if focus && !pause {
        let now = Instant::now();
        info!(
          "{:?} {:?} {:?}",
          now - cycle_timer,
          elapsed_time,
          elapsed_time + (now - cycle_timer)
        );
        elapsed_time += now - cycle_timer;
        cycle_timer = now;

        let mut iter_time = 0;
        while elapsed_time > CPU_CYCLE_DURATION {
          self.step();
          elapsed_time -= CPU_CYCLE_DURATION;
          iter_time += 1;
        }

        self
          .window
          .draw(self.emulator_screen.borrow_mut().deref_mut());
        self.window.display();
        info!(
          "last frame toke {:?} for {} times.",
          Instant::now() - now,
          iter_time
        );
      } else {
        sfml::system::sleep(sfml::system::Time::milliseconds(1000 / 60));
      }
    }
  }

  pub fn set_keys(&mut self, p1: Vec<Key>, p2: Vec<Key>) {
    self
      .cpu
      .borrow_mut()
      .main_bus()
      .borrow_mut()
      .set_controller_keys(p1, p2);
  }
}
