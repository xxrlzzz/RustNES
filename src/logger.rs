use log::{Level, Metadata, SetLoggerError};

struct SimpleLogger {
  level: Level,
}

impl log::Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level
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
  // static LOGGER: SimpleLogger = SimpleLogger {
  //   // file_path: String::from("log.txt"),
  //   // file: Option::None,
  //   level: Level::Info,
  // };
  // log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
  let logger = SimpleLogger { level: Level::Info };
  log::set_boxed_logger(Box::new(logger))
}

//TODO:
// pub fn update_level() {
// }
//
