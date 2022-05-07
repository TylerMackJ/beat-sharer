#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

pub mod api;
mod app;
pub mod util;

pub use app::BeatSharerApp;

#[macro_use]
extern crate dotenv_codegen;

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    dotenv::dotenv().ok();

    let native_options = eframe::NativeOptions::default();
}
