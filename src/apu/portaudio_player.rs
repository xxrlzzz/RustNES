use log::warn;
use portaudio::{
  stream_flags, Error, NonBlocking, Output, OutputStreamCallbackArgs, PortAudio, Stream,
  StreamCallbackResult::Continue,
};

use std::sync::mpsc;

#[derive(Default)]
pub struct PortAudioPlayer {
  stream: Option<Stream<NonBlocking, Output<f32>>>,
  sender: Option<mpsc::Sender<f32>>,
}

const FRAMES_PER_BUFFER: u32 = 64;

impl PortAudioPlayer {
  pub fn new() -> Self {
    Self {
      stream: None,
      sender: None,
    }
  }

  pub(crate) fn send_sample(&mut self, sample: f32) {
    match self.sender.as_ref().unwrap().send(sample) {
      Ok(_) => {}
      Err(_) => {
        warn!("failed to add sample to sound buffer");
      }
    }
  }

  pub fn init(&mut self) -> Result<f64, Error> {
    let pa = PortAudio::new()?;
    let (tx, rx) = mpsc::channel();
    self.sender = Some(tx);

    let output_device = pa.default_output_device()?;
    let output_info = pa.device_info(output_device)?;
    let mut output_setting = pa.default_output_stream_settings(
      output_info.max_output_channels,
      output_info.default_sample_rate,
      FRAMES_PER_BUFFER,
    )?;
    // we won't output out of range samples so don't bother clipping them.
    output_setting.flags = stream_flags::CLIP_OFF;
    // let channels = output_info.max_output_channels as usize;

    // This routine will be called by the PortAudio engine when audio is needed. It may called at
    // interrupt level on some machines so don't do anything that could mess up the system like
    // dynamic resource allocation or IO.
    let callback = move |OutputStreamCallbackArgs { buffer, frames, .. }| {
      let mut i = 0;
      for _ in 0..frames {
        // copy one channel.
        buffer[i] = rx.try_recv().unwrap_or(0.0);
        buffer[i + 1] = buffer[i];
        i += 2;
      }
      Continue
    };
    let stream = pa.open_non_blocking_stream(output_setting, callback)?;
    self.stream = Some(stream);
    Ok(output_info.default_sample_rate)
  }

  pub fn start(&mut self) -> Result<(), Error> {
    if let Some(ref mut stream) = self.stream {
      stream.start()?
    } else {
      self.init()?;
      self.start()?;
    }
    Ok(())
  }

  pub fn stop(&mut self) -> Result<(), Error> {
    if let Some(ref mut stream) = self.stream {
      stream.stop()?;
      stream.close()?;
    }
    Ok(())
  }
}
impl Drop for PortAudioPlayer {
  fn drop(&mut self) {
    match self.stop() {
      Ok(_) => {}
      Err(e) => {
        warn!("failed to stop portaudio player: {}", e);
      }
    }
  }
}

mod test {

  #[test]
  pub(crate) fn test_portaudio_player() {
    use super::*;
    use core::time::Duration;
    use std::f64::consts::PI;
    use std::thread::sleep;
    let mut player = PortAudioPlayer::new();
    player.init().unwrap();
    player.start().unwrap();
    let sample_length = 200;
    for i in 0..sample_length {
      player.send_sample((i as f64 / sample_length as f64 * PI * 2.0).sin() as f32);
    }
    sleep(Duration::from_millis(100));
    player.stop().unwrap();
  }
}
