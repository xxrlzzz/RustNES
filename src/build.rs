use std::{env, path::PathBuf};

fn main() {
  let library_name = "SDL2";
  let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
  // Link SDL2 for android.
  match env::var("TARGET") {
    Ok(target) => {
      let relative_path = if target.contains("armv7") {
        "native-libs/armv7"
      } else if target.contains("i686") {
        "native-libs/x86"
      } else if target.contains("aarch64") {
        "native-libs/aarch64"
      } else if target.contains("wasm") {
        return;
      } else {
        ""
      };
      println!("cargo:rustc-link-lib=dylib={}", library_name);
      let _ = dunce::canonicalize(root.join(relative_path)).map(|library_dir| {
        println!(
          "cargo:rustc-link-search=native={}",
          env::join_paths(&[library_dir]).unwrap().to_str().unwrap()
        );
      });
    }
    Err(_e) => {}
  }
}
