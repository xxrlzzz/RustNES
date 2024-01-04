#[cfg(target_os = "android")]
pub mod android;

#[cfg(target_arch = "wasm32")]
pub mod wasm;

#[cfg(feature = "wasm-miniapp")]
pub mod miniapp;

#[cfg(all(feature = "wasm", not(feature = "wasm-miniapp")))]
pub mod web;
