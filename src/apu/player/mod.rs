use std::sync::mpsc;

use anyhow::anyhow;

use crate::NesResult;

#[cfg(feature = "native-audio")]
mod portaudio;

pub trait Player {
  fn init(&mut self) -> NesResult<f64>;

  fn start(&mut self) -> NesResult<()>;

  fn stop(&mut self) -> NesResult<()>;

  fn send_sample(&mut self, sample: f32) -> NesResult<()>;

  fn pull_samples(&mut self, sample_size: usize) -> NesResult<Vec<f32>>;
}

pub struct DummyPlayer {
  sample_rate: f64,
  buf: Vec<f32>,
  sender: Option<mpsc::Sender<f32>>,
  receiver: Option<mpsc::Receiver<f32>>,
}

impl DummyPlayer {
  pub fn new() -> Self {
    let (sender, receiver) = mpsc::channel();
    DummyPlayer {
      sample_rate: 44100.0,
      buf: Vec::new(),
      sender: Some(sender),
      receiver: Some(receiver),
    }
  }
}

impl Player for DummyPlayer {
  fn init(&mut self) -> NesResult<f64> {
    Ok(self.sample_rate)
  }

  fn start(&mut self) -> NesResult<()> {
    Ok(())
  }

  fn stop(&mut self) -> NesResult<()> {
    Ok(())
  }

  fn send_sample(&mut self, sample: f32) -> NesResult<()> {
    if let Some(ref sender) = self.sender {
      sender.send(sample)?;
      Ok(())
    } else {
      Err(anyhow!("sender is not initialized"))
    }
  }

  fn pull_samples(&mut self, sample_size: usize) -> NesResult<Vec<f32>> {
    if let Some(ref receiver) = self.receiver {
      for sample in receiver.try_iter() {
        self.buf.push(sample);
      }
      while self.buf.len() < sample_size {
        self.buf.push(0.0);
      }

      Ok(self.buf.drain(..sample_size).collect())
    } else {
      return Err(anyhow!("receiver is not initialized"));
    }
  }
}

impl Default for Box<dyn Player> {
  fn default() -> Self {
    #[cfg(feature = "wasm")]
    return Box::new(DummyPlayer::new());

    #[cfg(feature = "native-audio")]
    return Box::new(portaudio::PortAudioPlayer::new());
  }
}
