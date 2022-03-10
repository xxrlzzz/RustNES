# RustNESEmulator

Hello, this is a NES Emulator base on [SimpleNES](https://github.com/amhndu/SimpleNES) implemented by Rust.


## Build

We use [SFML](https://docs.rs/sfml/latest/sfml/) binding on c++ to show game window, so SFML 2.5 and CSFML 2.5 must be installed.

Note due to the performance issue, you should run in release mode to get enough frame rate to fresh screen.

## TODO

1. Audio support # DONE
2. More Rom type support
3. Pause/Resume/Save