use crate::util::StringUtils;
use eframe::egui;
use std::path::PathBuf;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

pub struct BeatSharerApp {}

impl epi::App for BeatSharerApp {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |_| {});
    }

    fn setup(
        &mut self,
        _ctx: &egui::Context,
        frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        // Load previous state
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        frame.set_window_size(egui::Vec2::new(600.0, 400.0));
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    fn name(&self) -> &str {
        "Beat Sharer"
    }
}

fn get_codes(path: PathBuf) -> Vec<String> {
    let dir = path.read_dir().unwrap();
    let mut codes: Vec<String> = Vec::new();

    for entry in dir {
        let full_path = entry.unwrap().path();

        let filename = full_path.file_name().unwrap();

        let end = match filename.to_str().unwrap().find(" (") {
            Some(t) => t,
            None => continue,
        };
        let code = filename.to_str().unwrap().to_string().substring(0, end);

        if code.chars().count() <= 5 {
            codes.push(code);
        }
    }

    codes
}
