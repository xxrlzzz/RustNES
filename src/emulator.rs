use log::{debug, error, info};
use serde_json::json;
use sfml::graphics::{Color, RenderTarget, RenderWindow};
use sfml::window::{ContextSettings, Event, Key, Style, VideoMode};
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

use crate::apu::{Apu, CPU_FREQUENCY};
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

struct Instance {
  pub(crate) apu: Rc<RefCell<Apu>>,
  pub(crate) cpu: Cpu,
  pub(crate) ppu: Rc<RefCell<Ppu>>,
}

type MatrixType = HashMap<&'static str, u128>;

impl Instance {
  pub(crate) fn new(apu: Rc<RefCell<Apu>>, cpu: Cpu, ppu: Rc<RefCell<Ppu>>) -> Self {
    Self { apu, cpu, ppu }
  }
}
pub struct Emulator {
  emulator_screen: VirtualScreen,
  window: RenderWindow,
  save_path: String,

  message_bus: Rc<RefCell<MessageBus>>,
  ctl1: Vec<Key>,
  ctl2: Vec<Key>,
}

impl Emulator {
  pub fn new(screen_scale: f32, save_path: String, ctl1: Vec<Key>, ctl2: Vec<Key>) -> Self {
    let message_bus = Rc::new(RefCell::new(MessageBus::new()));
    let video_mode = VideoMode::new(
      (NES_VIDEO_WIDTH as f32 * screen_scale) as u32,
      (NES_VIDEO_HEIGHT as f32 * screen_scale) as u32,
      32,
    );

    let mut emulator_screen = VirtualScreen::new();

    emulator_screen.create(
      NES_VIDEO_WIDTH,
      NES_VIDEO_HEIGHT,
      screen_scale,
      Color::WHITE,
    );

    Self {
      emulator_screen,
      window: RenderWindow::new(
        video_mode,
        "NES-Simulator",
        Style::TITLEBAR | Style::CLOSE,
        &ContextSettings::default(),
      ),
      save_path,
      message_bus,
      ctl1,
      ctl2,
    }
  }

  fn load(&self) -> Result<Instance, std::io::Error> {
    let mut file = std::fs::File::open(&self.save_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let json_obj: serde_json::Value = serde_json::from_str(buffer.as_str()).unwrap();
    let mut cpu: Cpu = json_obj
      .get("cpu")
      .map(|cpu_value| serde_json::from_str(cpu_value.as_str().unwrap()).unwrap())
      .unwrap();
    let apu = json_obj
      .get("apu")
      .map(|apu| {
        let mut apu: Apu = serde_json::from_str(apu.as_str().unwrap()).unwrap();
        apu.start();
        Rc::new(RefCell::new(apu))
      })
      .unwrap();
    let ppu = json_obj
      .get("ppu")
      .map(|ppu| {
        let mut ppu: Ppu = serde_json::from_str(ppu.as_str().unwrap()).unwrap();
        ppu.set_message_bus(self.message_bus.clone());
        Rc::new(RefCell::new(ppu))
      })
      .unwrap();
    json_obj.get("main_bus").map(|main_bus| {
      let mut main_bus = MainBus::load(main_bus, ppu.clone(), apu.clone());
      main_bus.set_controller_keys(self.ctl1.clone(), self.ctl2.clone());
      cpu.set_main_bus(main_bus);
    });
    Ok(Instance::new(apu, cpu, ppu))
  }

  fn consume_message(&mut self, cpu: &mut Cpu) {
    let mut message_bus = self.message_bus.borrow_mut();
    loop {
      let message = message_bus.peek();
      match message {
        None => break,
        Some(message) => {
          match message {
            Message::CpuInterrupt => {
              cpu.interrupt(InterruptType::NMI);
            }
            Message::PpuRender(frame) => {
              self.emulator_screen.set_picture(&frame);
            }
          };
          message_bus.pop();
        }
      }
    }
  }

  fn step(&mut self, instance: &mut Instance, matrix: &mut MatrixType) {
    let mut now = Instant::now();
    let mut ppu = instance.ppu.borrow_mut();
    ppu.step();
    ppu.step();
    ppu.step();
    self.consume_message(&mut instance.cpu);

    now = sample_profile(now, "ppu", matrix);

    instance.cpu.step();
    now = sample_profile(now, "cpu", matrix);

    instance.apu.borrow_mut().step();
    sample_profile(now, "apu", matrix);
  }

  fn init_rom(&self, rom_path: &str) -> Option<Instance> {
    let ppu = Rc::new(RefCell::new(Ppu::new(
      PictureBus::new(),
      self.message_bus.clone(),
    )));
    let apu = Rc::new(RefCell::new(Apu::new()));
    let mut main_bus = MainBus::new(apu.clone(), ppu.clone());
    main_bus.set_controller_keys(self.ctl1.clone(), self.ctl2.clone());

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

  fn one_frame(
    &mut self,
    instance: &mut Instance,
    start_timer: Instant,
    elapsed_time: &mut Duration,
  ) {
    let mut iter_time = 0;
    let mut matrix = MatrixType::default();
    while *elapsed_time > CPU_CYCLE_DURATION && iter_time < CPU_FREQUENCY {
      self.step(instance, &mut matrix);
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
      matrix["ppu"] / 1000,
      matrix["cpu"] / 1000,
      matrix["apu"] / 1000,
      (end - start).as_millis(),
    );
  }

  pub fn run(&mut self, rom_path: &str) {
    let mut instance = match self.init_rom(rom_path) {
      None => return,
      Some(instance) => instance,
    };
    self.window.set_vertical_sync_enabled(true);

    instance.apu.borrow_mut().start();
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

          Event::KeyPressed { code: Key::Z, .. } => match save(&instance, &self.save_path) {
            Ok(_) => info!("save success"),
            Err(e) => error!("save failed: {}", e),
          },
          Event::KeyPressed { code: Key::X, .. } => match self.load() {
            Ok(instance_load) => {
              instance = instance_load;
              info!("load success")
            }
            Err(e) => error!("load failed: {}", e),
          },
          Event::KeyPressed { code: Key::F2, .. } => {
            pause = !pause;
            if !pause {
              cycle_timer = Instant::now();
            }
          }
          Event::KeyReleased { code: Key::F3, .. } => {
            if pause {
              for _ in 0..29781 {
                let mut matrix = MatrixType::default();
                self.step(&mut instance, &mut matrix);
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
        self.one_frame(&mut instance, now, &mut elapsed_time);
      } else {
        sfml::system::sleep(sfml::system::Time::milliseconds(1000 / 60));
      }
    }
  }
}

fn sample_profile(start: Instant, category: &'static str, matrix: &mut MatrixType) -> Instant {
  let now = Instant::now();
  let duration = (now - start).as_micros();
  matrix
    .entry(category)
    .and_modify(|e| *e += duration)
    .or_insert(duration);
  now
}

fn save(instance: &Instance, file: &String) -> Result<(), std::io::Error> {
  let mut file = std::fs::File::create(file)?;
  let json = json!({
    "apu": serde_json::to_string(&*instance.apu.borrow()).unwrap(),
    "cpu": serde_json::to_string(&instance.cpu).unwrap(),
    "ppu": serde_json::to_string(&*instance.ppu.borrow()).unwrap(),
    "main_bus" : instance.cpu.main_bus().save(),
  });
  file.write_all(json.to_string().as_bytes())
}
