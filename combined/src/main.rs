#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] //Hide console window in release builds on Windows, this blocks stdout.

pub mod api;
mod app;
pub mod util;

pub use app::BeatSharerApp;

use std::path::PathBuf;

#[macro_use]
extern crate dotenv_codegen;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    dotenv::dotenv().ok();

    //let song_info = api::beatsaver::get_song_info("10065".to_string()).unwrap();
    //api::beatsaver::download_and_unzip_song(song_info, PathBuf::from(".")).unwrap();

    /*
    let mut bytes = include_bytes!("icon.png").to_vec();
    let mut im = image::io::Reader::new(std::io::Cursor::new(&mut bytes));
    im.set_format(image::ImageFormat::Png);

    let app = BeatSharerApp::default();
    let native_options = eframe::NativeOptions {
        icon_data: Some(eframe::epi::IconData {
            rgba: im.decode().unwrap().to_rgba8().into_raw(),
            width: 400,
            height: 400,
        }),
        resizable: false,
        ..Default::default()
    };

    eframe::run_native(Box::new(app), native_options)
    */
}
