use clap::Parser;

use rust_nes::{controller, emulator, logger};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
struct Args {
  #[clap(short, long, default_value = "assets/mario.nes")]
  rom_path: String,
  // relative path to where you run cmd.
  #[clap(short, long, default_value = "assets/keybindings.ini")]
  key_binding_path: String,

  #[clap(short, long, default_value = "2.0")]
  scale: f32,

  #[clap(long, default_value = "save/saved.json")]
  save_path: String,
}

// #[tokio::main(flavor = "multi_thread", worker_threads = 10)]
fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let args = Args::parse();
  let (p1_key, p2_key) = controller::key_binding_parser::parse_key_binding(&args.key_binding_path);
  let mut emulator = emulator::Emulator::new(args.scale, args.save_path, p1_key, p2_key);
  let instance = emulator.create_instance(&args.rom_path);
  emulator.run(instance);
}

#[cfg(test)]
mod tests {
  use std::time::Instant;

  type MatrixType = std::collections::HashMap<&'static str, u128>;
  const CPU_FREQUENCY: u32 = 1788908;

  #[test]
  fn hashmap_test() {
    let mut matrix = MatrixType::default();
    let category_list = vec!["1", "2", "3"];
    let start = Instant::now();
    for i in 0..CPU_FREQUENCY {
      matrix
        .entry(category_list[i as usize % 3])
        .and_modify(|e| *e += 1)
        .or_insert(0);
    }
    println!("total cost :{:?}", Instant::now() - start);
  }

  #[test]
  fn thread_test() {
    // let mut handles = vec![];
    for i in 0..10 {
      std::thread::spawn(move || {
        println!("thread {} start", i);
        std::thread::sleep(std::time::Duration::from_secs(1));
        println!("thread {} end", i);
      });
      // handles.push(handle);
    }
    // for handle in handles {
    //   handle.join().unwrap();
    // }
    std::thread::sleep(std::time::Duration::from_secs(2));
  }
}
