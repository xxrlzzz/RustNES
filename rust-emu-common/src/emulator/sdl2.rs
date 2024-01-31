use std::collections::HashSet;
use std::thread;

use crate::emulator::{APP_NAME, FRAME_DURATION, NES_VIDEO_HEIGHT, NES_VIDEO_WIDTH};
use crate::instance::Instance;

use super::{Emulator, RuntimeConfig};

use image::{ImageBuffer, Rgba};
use log::{error, info};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Canvas, Texture};
use sdl2::surface::Surface;
use sdl2::video::Window;
use sdl2::Sdl;

impl Emulator {
  fn update_keys(&self, event_pump: &sdl2::EventPump) {
    let keyboard_status = event_pump.keyboard_state();
    let keycodes: HashSet<sdl2::keyboard::Keycode> = keyboard_status
      .pressed_scancodes()
      .flat_map(sdl2::keyboard::Keycode::from_scancode)
      .collect();
    unsafe {
      crate::controller::sdl2_key::KEYBOARD_STATE = Some(keycodes);
    }
  }

  fn update_display(
    &self,
    instance: &mut Instance,
    texture: &mut Texture,
    canvas: &mut Canvas<Window>,
  ) {
    let mut rgba = instance.take_rgba();
    if rgba.is_some() {
      set_sdl2_texture(texture, rgba.take().unwrap());

      let _ = canvas.copy(&texture, None, None);
      canvas.present();
    }
  }

  fn create_window(&self) -> Result<(Sdl, Canvas<Window>), String> {
    let (width, height) = self.runtime_config.window_size();
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
      .window(APP_NAME, width, height)
      .position_centered()
      .allow_highdpi()
      .build()
      .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;
    canvas.set_blend_mode(BlendMode::None);
    canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
    canvas.clear();

    Ok((sdl_context, canvas))
  }

  pub fn run(&mut self, mut instance: Instance) {
    let runtime_config = self.runtime_config.clone();

    let (sdl_context, mut canvas) = self.create_window().unwrap();

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
      self.update_keys(&event_pump);
      self.update_display(&mut instance, &mut texture, &mut canvas);

      if instance.can_run() {
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
        Keycode::X => match Instance::load(&runtime_config) {
          Ok(instance_load) => {
            *instance = instance_load;
            info!("load success")
          }
          Err(e) => error!("load failed: {}", e),
        },
        Keycode::F2 => instance.toggle_pause(),
        Keycode::F3 => {
          if instance.stat.is_pausing() {
            for _ in 0..29781 {
              instance.step();
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
}

fn set_sdl2_texture(texture: &mut Texture, rgba: ImageBuffer<Rgba<u8>, Vec<u8>>) {
  let width = rgba.width();
  let data = rgba.into_vec();
  texture
    .update(None, data.as_slice(), (width * 4) as _)
    .unwrap();
}
