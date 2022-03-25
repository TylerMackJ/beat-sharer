#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

mod app;
pub use app::BeatSharerApp;

#[macro_use]
extern crate dotenv_codegen;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    dotenv::dotenv().ok();

    let app = BeatSharerApp::default();
    //let native_options = eframe::NativeOptions::default();
    let native_options = eframe::NativeOptions {
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(Box::new(app), native_options)
}
