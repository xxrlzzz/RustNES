use clap::Parser;
use rust_emu_common::logger;

#[cfg(any(feature = "use_gl", feature = "use_sdl2"))]
use rust_emu_common::{controller, emulator};
use rust_nes::instance::init_rom_from_path;

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

#[cfg(any(feature = "use_gl", feature = "use_sdl2"))]
fn main() {
  match logger::init() {
    Err(_) => return,
    Ok(_) => {}
  };
  let args = Args::parse();
  let (p1_key, p2_key) = controller::key_binding_parser::parse_key_binding(&args.key_binding_path);
  let mut emulator = emulator::Emulator::new(args.scale, args.save_path, p1_key, p2_key);
  let instance = init_rom_from_path(&args.rom_path, &emulator.runtime_config).expect("Failed to load rom.");
  emulator.run(instance);
}

#[cfg(not(any(feature = "use_gl", feature = "use_sdl2")))]
fn main() {
  println!("Please use feature `use_gl` or `use_sdl2` to run this program.");
}

#[cfg(test)]
mod tests {
  use std::time::Instant;

  type MatrixType = std::collections::HashMap<&'static str, u128>;
  const CNT: u32 = 1788908;

  #[test]
  fn hashmap_test() {
    let mut matrix = MatrixType::default();
    let category_list = vec!["1", "2", "3"];
    let start = Instant::now();
    for i in 0..CNT {
      matrix
        .entry(category_list[i as usize % 3])
        .and_modify(|e| *e += 1)
        .or_insert(0);
    }
    println!("total cost :{:?}", Instant::now() - start);
  }

  #[test]
  fn match_test() {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let n = rng.gen_range(0x00..=0xFF);
    let mut sum = 0;
    let mut start = Instant::now();
    let time = 10000;
    for _ in 0..time {
      for i in 0..n {
        sum += match i {
          0x00..=0x1F => 1,
          0x20..=0x2F => 2,
          0x30..=0x4F => 4,
          0x50..=0x8F => 5,
          0x90..=0xFF => 3,
          _ => 0,
        }
      }
    }
    println!("total cost :{:?}, sum {}", Instant::now() - start, sum);
    start = Instant::now();
    for _ in 0..time {
      for i in 0..n {
        sum += if i <= 0x1F {
          1
        } else if i <= 0x2F {
          2
        } else if i <= 0x4F {
          4
        } else if i <= 0x8F {
          5
        } else {
          3
        };
      }
    }
    println!(
      "total cost :{:?}, sum {}",
      (Instant::now() - start) * 100,
      sum
    );
    match 9 {
      1..=5 => println!("1..=5"),
      6..=9 => println!("6..=10"),
      _ => println!("other"),
    }
  }
}
