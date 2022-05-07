use crate::api;
use crate::util::StringUtils;
use std::path::{Path, PathBuf};

enum UploadStatus {
    NotStarted,
    GettingIndex(tokio::sync::oneshot::Receiver<Result<u8, api::APIErr>>),
    Uploading(tokio::sync::oneshot::Receiver<Result<(), api::APIErr>>),
    Completed,
}

enum DownloadStatus {
    NotStarted,
    GettingList(tokio::sync::oneshot::Receiver<Result<Vec<String>, api::APIErr>>),
    Downloading(api::DownloadObserver),
    Completed,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct BeatSharerApp {
    custom_level_path: PathBuf,

    #[serde(skip)]
    codes: Vec<String>,
    #[serde(skip)]
    upload_status: UploadStatus,
    #[serde(skip)]
    upload_code: u8,
    #[serde(skip)]
    download_index_buf: String,
    #[serde(skip)]
    download_status: DownloadStatus,
}

impl Default for BeatSharerApp {
    fn default() -> Self {
        Self {
            custom_level_path: std::env::current_dir().unwrap(),
            codes: get_codes(std::env::current_dir().unwrap()),
            upload_status: UploadStatus::NotStarted,
            upload_code: 0,
            download_index_buf: String::from(""),
            download_status: DownloadStatus::NotStarted,
        }
    }
}

impl BeatSharerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut temp: BeatSharerApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            temp.codes = get_codes(temp.custom_level_path.clone());
            return temp;
        }

        Default::default()
    }
}

impl eframe::App for BeatSharerApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle Getting Index
        if let UploadStatus::GettingIndex(r) = &mut self.upload_status {
            // todo errors
            if let Ok(upload_code) = r.try_recv() {
                self.upload_code = upload_code.unwrap();

                let mut upload_string = String::new();
                for c in &self.codes {
                    upload_string.push_str(format!("{},", c).as_str());
                }

                self.upload_status =
                    UploadStatus::Uploading(api::put_list(self.upload_code, upload_string));
            }
        } else if let UploadStatus::Uploading(r) = &mut self.upload_status {
            // todo errors
            if r.try_recv().is_ok() {
                self.upload_status = UploadStatus::Completed;
            }
        }

        if let DownloadStatus::GettingList(r) = &mut self.download_status {
            // todo errors
            if let Ok(list) = r.try_recv() {
                self.download_status = DownloadStatus::Downloading(api::download(
                    list.unwrap(),
                    self.custom_level_path.clone(),
                    std::num::NonZeroUsize::new(
                        std::thread::available_parallelism()
                            .unwrap_or(std::num::NonZeroUsize::new(1).unwrap())
                            .get()
                            * 2,
                    )
                    .unwrap(),
                ))
            }
        }

        if let DownloadStatus::Downloading(download_observer) = &mut self.download_status {
            // todo progress bar
            if !download_observer.downloading() {
                self.download_status = DownloadStatus::Completed
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.heading("Selected Folder");
                ui.label(format!(
                    "{} ({} Songs found)",
                    self.custom_level_path.to_str().unwrap(),
                    self.codes.len(),
                ));
                if ui.add(egui::Button::new("Change Folder")).clicked() {
                    if let Some(result) =
                        tinyfiledialogs::select_folder_dialog("Select CustomLevels Folder", ".")
                    {
                        self.custom_level_path = Path::new(&result).to_path_buf();
                        self.codes = get_codes(self.custom_level_path.clone());
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_size(egui::Vec2::new(200.0, 60.0));
                    ui.set_max_size(egui::Vec2::new(200.0, 60.0));

                    ui.heading("Upload");
                    // Uploaded
                    if let UploadStatus::Completed = self.upload_status {
                        ui.label(format!(
                            "Uploaded {} songs to ID: {}",
                            self.codes.len(),
                            self.upload_code
                        ));
                    // Getting Index
                    } else if let UploadStatus::GettingIndex(_) = self.upload_status {
                        ui.label("Getting Unique ID...");
                    // Uploading
                    } else if let UploadStatus::Uploading(_) = self.upload_status {
                        ui.label("Uploading...");
                    // No songs
                    } else if self.codes.is_empty() {
                        ui.label("Found no songs to upload");
                    // Click to upload
                    } else if ui
                        .add(egui::Button::new(format!(
                            "Upload {} songs",
                            self.codes.len()
                        )))
                        .clicked()
                    {
                        // Upload
                        self.upload_status = UploadStatus::GettingIndex(api::get_and_inc_index());
                    }
                });

                ui.separator();

                ui.vertical(|ui| {
                    ui.set_min_size(egui::Vec2::new(400.0, 60.0));
                    ui.set_max_size(egui::Vec2::new(400.0, 60.0));

                    ui.heading("Download");
                    ui.horizontal(|ui| {
                        // todo allow to download with no other songs
                        if self.codes.is_empty() {
                            ui.label("Are you sure your CustomLevels folder is selected?");
                        } else {
                            // todo bad id
                            // todo dont allow multiple downloads
                            ui.add(
                                egui::TextEdit::singleline(&mut self.download_index_buf)
                                    .hint_text("Enter ID")
                                    .desired_width(75.0),
                            );
                            if ui.add(egui::Button::new("Download Songs")).clicked() {
                                if let Ok(index) = self.download_index_buf.parse::<u8>() {
                                    self.download_status =
                                        DownloadStatus::GettingList(api::get_list(index))
                                }
                            }
                        }
                    });
                });
            });
        });
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
