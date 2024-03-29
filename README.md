# RustNESEmulator

Hello, this is a NES Emulator base on [SimpleNES](https://github.com/amhndu/SimpleNES) implemented by Rust.


## Build

We support 
- [glfw](https://docs.rs/glfw/0.48.0/glfw/)  
- [sdl2](https://docs.rs/sdl2/0.35.2/sdl2/) (for android)
- [webgl] (for web)
to show game window. Switch by set feature flag.

and also use [portaudio](https://docs.rs/portaudio/0.7.0/portaudio/) to play game music. 

No more extra dependencies are required to build this project.

Note due to the performance issue, you should run in release mode to get enough frame rate to fresh screen.
## Multi-platform Supporting

- Desktop(Mac)
- Android
- Web(Wasm + WebGL)
- Miniapp(same with web, but render on js rather than rust js binding)


## Screenshots

- web
![web](./img/web.png)

- desktop 
![desktop](./img/desktop.png)

- integration with streaming and co-play
![stream](./img/stream_integration.png)

- miniapp  
![miniapp](./img/miniapp.jpg)

## TODO

- [x] Audio support
- [x] Pause/Resume/Save 
- [x] Android support v1
- [x] Android keybinding.
- [ ] More Rom type support
- [ ] Test more nes games and upload result.
- [ ] Dynamic load rom.
- [ ] Fix sound effect issue. 
- [x] Web support(audio, controller)
