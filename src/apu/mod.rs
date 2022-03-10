pub mod portaudio_player;
pub mod sound_filter;
pub mod sound_wave;

use log::{info, warn};

use crate::common::{
  bit_eq,
  types::{Address, Byte},
};

use self::{
  portaudio_player::PortAudioPlayer,
  sound_filter::{Filter, SoundFilter, SoundFilterChain},
  sound_wave::{Noise, Pulse, Triangle, DMC},
};

lazy_static! {
  static ref PULSE_TABLE: [f32; 32] = {
    let mut table = [0.0; 32];
    for i in 0..32 {
      table[i] = 95.52 / (8128.0 / (i as f32) + 100.0);
    }
    table
  };
  static ref TND_TABLE: [f32; 203] = {
    let mut table = [0.0; 203];
    for i in 0..203 {
      table[i] = 163.67 / (24329.0 / (i as f32) + 100.0);
    }
    table
  };
}
/**
 * Reference https://www.nesdev.org/apu_ref.txt
 */
pub struct Apu {
  cycle: u64,
  frame_period: Byte,
  frame_value: Byte,
  frame_irq: bool,
  player: PortAudioPlayer,
  sample_rate: f64,

  pulse1: Pulse,
  pulse2: Pulse,
  triangle: Triangle,
  noise: Noise,
  dmc: DMC,

  filter_chain: SoundFilterChain,
}

const CPU_FREQUENCY: u32 = 1788908;

const FRAME_COUNTER_RATE: f64 = CPU_FREQUENCY as f64 / 240.0;
impl Apu {
  pub fn new() -> Self {
    let mut player = PortAudioPlayer::new();
    let sample_rate = player.init().unwrap() as f32;
    Self {
      cycle: 0,
      frame_period: 0,
      frame_value: 0,
      frame_irq: false,
      player,
      sample_rate: CPU_FREQUENCY as f64 / sample_rate as f64,
      pulse1: Pulse::new(1),
      pulse2: Pulse::new(2),
      triangle: Triangle::new(),
      noise: Noise::new(),
      dmc: DMC::new(),
      filter_chain: vec![
        SoundFilter::new_high_pass_filter(sample_rate, 90.),
        SoundFilter::new_high_pass_filter(sample_rate, 440.),
        SoundFilter::new_low_pass_filter(sample_rate, 14000.),
      ],
    }
  }
  pub fn start(&mut self) {
    match self.player.start() {
      Ok(_) => {}
      Err(e) => {
        warn!("failed to start portaudio player: {}", e);
      }
    }
  }
  pub fn stop(&mut self) {
    match self.player.stop() {
      Ok(_) => {}
      Err(e) => {
        warn!("failed to stop portaudio player: {}", e);
      }
    }
  }

  pub fn step(&mut self) {
    let cycle1 = self.cycle as f64;
    self.cycle += 1;
    let cycle2 = self.cycle as f64;
    self.step_timer();
    let f1 = (cycle1 / FRAME_COUNTER_RATE) as u32;
    let f2 = (cycle2 / FRAME_COUNTER_RATE) as u32;
    if f1 != f2 {
      self.step_frame_counter();
    }
    let s1 = (cycle1 / self.sample_rate) as u32;
    let s2 = (cycle2 / self.sample_rate) as u32;
    if s1 != s2 {
      self.send_sample();
    }
  }

  fn send_sample(&mut self) {
    let pulse1 = self.pulse1.output();
    let pulse2 = self.pulse2.output();
    let triangle = self.triangle.output();
    let noise = self.noise.output();
    let dmc = self.dmc.output();
    let sample = PULSE_TABLE[(pulse1 + pulse2) as usize]
      + TND_TABLE[(3 * triangle + 2 * noise + dmc) as usize];
    let after_sample = self.filter_chain.step(sample);
    if sample != 0. {
      // info!("sample: {:?} {:?}", after_sample, sample);
    }
    self.player.send_sample(after_sample);
  }

  // mode 0:    mode 1:       function
  // ---------  -----------  -----------------------------
  //  - - - f    - - - - -    IRQ (if bit 6 is clear)
  //  - l - l    l - l - -    Length counter and sweep
  //  e e e e    e e e e -    Envelope and linear counter
  pub fn step_frame_counter(&mut self) {
    if self.frame_period == 0 {
      return;
    }
    self.frame_value = (self.frame_value + 1) % self.frame_period;
    if self.frame_period == 4 {
      match self.frame_value {
        0 | 2 => self.step_envelope(),
        1 => {
          self.step_envelope();
          self.step_sweep();
          self.step_length();
          self.fire_irq();
        }
        3 => {
          self.step_envelope();
          self.step_sweep();
          self.step_length();
        }
        _ => (),
      }
    } else if self.frame_period == 5 {
      match self.frame_value {
        0 | 2 => self.step_envelope(),
        1 | 3 => {
          self.step_sweep();
          self.step_length();
        }
        _ => (),
      }
    }
  }

  fn step_timer(&mut self) {
    if self.cycle % 2 == 0 {
      self.pulse1.step_timer();
      self.pulse2.step_timer();
      self.noise.step_timer();
      self.dmc.step_timer();
    }
    self.triangle.step_timer()
  }

  pub fn step_envelope(&mut self) {
    self.pulse1.step_envelope();
    self.pulse2.step_envelope();
    self.triangle.step_counter();
    self.noise.step_envelope();
  }

  pub fn step_sweep(&mut self) {
    self.pulse1.step_sweep();
    self.pulse2.step_sweep();
  }

  pub fn step_length(&mut self) {
    self.pulse1.step_length();
    self.pulse2.step_length();
    self.triangle.step_length();
    self.noise.step_length();
  }

  pub fn fire_irq(&self) {}

  pub fn read_status(&self) -> Byte {
    info!("read status");
    let mut result = 0;
    if self.pulse1.length_value() > 0 {
      result |= 1;
    }
    if self.pulse2.length_value() > 0 {
      result |= 2;
    }
    if self.triangle.length_value > 0 {
      result |= 4;
    }
    if self.noise.length_value() > 0 {
      result |= 8;
    }
    if self.dmc.current_length > 0 {
      result |= 16;
    }
    result
  }

  pub fn write_register(&mut self, address: Address, value: Byte) {
    match address {
      0x4000 => self.pulse1.write_control(value),
      0x4001 => self.pulse1.write_sweep(value),
      0x4002 => self.pulse1.write_timer_low(value),
      0x4003 => self.pulse1.write_timer_high(value),
      0x4004 => self.pulse2.write_control(value),
      0x4005 => self.pulse2.write_sweep(value),
      0x4006 => self.pulse2.write_timer_low(value),
      0x4007 => self.pulse2.write_timer_high(value),
      0x4008 => self.triangle.write_control(value),
      0x4009 => (),
      0x400A => self.triangle.write_timer_low(value),
      0x400B => self.triangle.write_timer_high(value),
      0x400C => self.noise.write_control(value),
      0x400D => (),
      0x400E => self.noise.write_period(value),
      0x400F => self.noise.write_length(value),
      0x4010 => self.dmc.write_control(value),
      0x4011 => self.dmc.write_value(value),
      0x4012 => self.dmc.write_address(value as Address),
      0x4013 => self.dmc.write_length(value as Address),
      0x4015 => self.write_control(value),
      0x4017 => self.write_frame_counter(value),
      _ => warn!("unhandled apu register write at address: 0x {}", address),
    }
  }

  pub fn write_control(&mut self, value: Byte) {
    self.pulse1.set_enabled(bit_eq(value, 1));
    self.pulse2.set_enabled(bit_eq(value, 2));
    self.triangle.set_enabled(bit_eq(value, 4));
    self.noise.set_enabled(bit_eq(value, 8));
    self.dmc.set_enabled(bit_eq(value, 16));
  }

  //  mi-- ----       mode, IRQ disable
  pub fn write_frame_counter(&mut self, value: Byte) {
    self.frame_period = 4 + if bit_eq(value, 0x80) { 1 } else { 0 };
    self.frame_irq = !bit_eq(value, 0x40);
    if self.frame_period == 5 {
      self.step_envelope();
      self.step_sweep();
      self.step_length();
    }
  }
}
