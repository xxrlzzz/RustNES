
## multiplatform

Current we use target_os, target_arch, custom features to support multiplatform

1. web: wasm
2. miniapp: wasm, wasm-miniapp
3. desktop: use_gl or use_sdl2
4. android: use_sdl2

### 

android toolchain: `aarch64-linux-android` and `armv7-linux-androideabi`

add ndk config for your `.cargo/config`

```
[target.aarch64-linux-android]
linker = /path2your_ndk_folder/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android30-clang
ar = ...
[target.aarch64-linux-android]
linker = ...
ar = ...
```

we also need `libsdl2` for android compile, you can download the source code, or copy so from playground.

more android stuff: https://gendignoux.com/blog/2022/10/24/rust-library-android.html

### web

wasm toolchain: `wasm32-unknown-unknown`

## debug support for vscode

1. install `CodeLLDB` extension
2. add configuration to `launch.json`
```
{
    "name": "Debug Rust",
    "type": "lldb",
    "request": "launch",
    "cargo": {
        // cargo args
        "args": [
            "build", 
            "--manifest-path", 
            "${workspaceFolder}/Cargo.toml",
            "--features",
            "use_sdl2",
        ]
    },
    // program args
    "args": ["--rom-path=${workspaceFolder}/assets/kunio-kun-no-nekketsu-soccer-league.nes"]
}
```