use log::{Metadata, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= log::max_level()
  }
  fn log(&self, rec: &log::Record) {
    if !self.enabled(rec.metadata()) {
      return;
    }
    let log_str = format!(
      "[{}] {}:{} {}",
      rec.level(),
      rec.file().unwrap_or("unknown file"),
      rec.line().unwrap_or(0),
      rec.args()
    );
    println!("{}", log_str)
  }
  fn flush(&self) {}
}

pub fn init() -> Result<(), SetLoggerError> {
  log::set_max_level(log::LevelFilter::Info);
  let logger = SimpleLogger;
  log::set_boxed_logger(Box::new(logger))
}
