# RustNESEmulator

Hello, this is a NES Emulator base on [SimpleNES](https://github.com/amhndu/SimpleNES) implemented by Rust.


## Build

We support 
- [SFML](https://docs.rs/sfml/latest/sfml/) binding on c++ or
- [glfw]  
to show game window. Switch by set feature flag.

and also use [portaudio](https://docs.rs/portaudio/0.7.0/portaudio/) to play music. 

Those dependencies are required to build this project.

Note due to the performance issue, you should run in release mode to get enough frame rate to fresh screen.

## TODO

- [x] Audio support
- [ ] More Rom type support
- [ ] Test more nes games and upload result.
- [x] Pause/Resume/Save 
