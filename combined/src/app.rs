use eframe::{egui, epi};
use std::{
    path::{
        PathBuf,
        Path,
    },
    env,
    thread,
    fs::{
        self,
        File
    },
    io::copy,
    sync::{
        Arc,
        Mutex,
    },
};
use rand::Rng;
use tinyfiledialogs::select_folder_dialog;
use zip::read::ZipArchive;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))] // if we add new fields, give them default values when deserializing old state

pub struct BeatSharerApp {
    custom_level_path: PathBuf,
    thread_count: usize,

    #[cfg_attr(feature = "persistence", serde(skip))]
    upload_code: u8,
    #[cfg_attr(feature = "persistence", serde(skip))]
    download_code: u8,
    #[cfg_attr(feature = "persistence", serde(skip))]
    download_code_buf: String,
    #[cfg_attr(feature = "persistence", serde(skip))]
    download_error: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    download_warning: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    uploaded: bool,
    #[cfg_attr(feature = "persistence", serde(skip))]
    codes: Vec<String>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    mutex: Arc<Mutex<Vec<String>>>,
    #[cfg_attr(feature = "persistence", serde(skip))]
    downloading: bool,

}

impl BeatSharerApp {
    fn download(&mut self) {
        self.downloading = true;
        // Download
        let mut contents = reqwest::blocking::get(format!("https://beat-sharer-default-rtdb.firebaseio.com/{}.json?auth={}", self.download_code, dotenv!("secret"))).unwrap().text().unwrap();
        self.download_warning = false;
        if contents == "null" {
            self.download_error = true;
        } else {
            contents = contents.substring(1, contents.chars().count() - 2);
            let mut songs_needed = Vec::new();
            for song in contents.split(',') {
                if !self.codes.contains(&String::from(song)) {
                    songs_needed.push(String::from(song));
                }
            }
            if self.thread_count > songs_needed.len() {
                self.thread_count = songs_needed.len();
            }
            let bot = songs_needed.len() as f32 / self.thread_count as f32;
            for thread in 0..self.thread_count {
                let start = (thread as f32 * bot) as usize;
                let mut end = ((thread + 1) as f32 * bot) as usize;

                if thread == self.thread_count - 1 {
                    end = songs_needed.len();
                }

                let cl_path = self.custom_level_path.clone();
                let mutex = Arc::clone(&self.mutex);

                let list = songs_needed[start..end].to_vec();

                let _download_thread = thread::spawn(move || {
                    download_songs(list, cl_path, mutex);
                });
            }
        }
    }
}

impl Default for BeatSharerApp {
    fn default() -> Self {
        let mut rng = rand::thread_rng();
        Self {
            custom_level_path: env::current_dir().unwrap(),
            thread_count: std::thread::available_parallelism().unwrap().get() / 4,
            upload_code: rng.gen(),
            download_code: 0,
            download_code_buf: String::from(""),
            download_error: false,
            download_warning: false,
            uploaded: false,
            codes: Vec::new(),
            mutex: Arc::new(Mutex::new(Vec::new())),
            downloading: false,
        }
    }
}

impl epi::App for BeatSharerApp {
    fn name(&self) -> &str {
        "Beat Sharer"
    }

    fn setup(&mut self, _ctx: &egui::Context, frame: &epi::Frame, _storage: Option<&dyn epi::Storage>) {
        // Load previous state
        #[cfg(feature = "persistence")]
        if let Some(storage) = _storage {
            *self = epi::get_value(storage, epi::APP_KEY).unwrap_or_default()
        }

        frame.set_window_size(egui::Vec2::new(600.0, 400.0));

        self.codes = get_codes(self.custom_level_path.clone());
    }

    /// Called by the frame work to save state before shutdown.
    #[cfg(feature = "persistence")]
    fn save(&mut self, storage: &mut dyn epi::Storage) {
        epi::set_value(storage, epi::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            // Select folder menu
            ui.vertical(|ui| {
                ui.heading("Selected Folder");
                ui.label(format!("{} ({} Songs found)", self.custom_level_path.to_str().unwrap(), self.codes.len()));
                if ui.add(egui::Button::new("Change Folder")).clicked() {
                    if let Some(result) = select_folder_dialog("Select CustomLevels Folder", ".") {
                        self.custom_level_path = Path::new(&result).to_path_buf();
                        self.codes = get_codes(self.custom_level_path.clone());
                    }
                }
            });

            ui.separator();

            // Upload song menu
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_min_size(egui::Vec2::new(200.0, 60.0));
                    ui.set_max_size(egui::Vec2::new(200.0, 60.0));

                    ui.heading("Upload");
                    // Uploaded
                    if self.uploaded {
                        ui.label(format!("Uploaded {} songs to ID: {}", self.codes.len(), self.upload_code));
                    // No songs
                    } else if self.codes.is_empty() {
                        ui.label("Found no songs to upload");
                    // Click to upload
                    } else if ui.add(egui::Button::new(format!("Upload {} songs", self.codes.len()))).clicked() {
                        // Upload
                        let client = reqwest::blocking::Client::new();
                        let mut upload_string = String::new();
                        for c in &self.codes {
                            upload_string.push_str(format!("{},", c).as_str());
                        }
                        let _res = client.put(format!("https://beat-sharer-default-rtdb.firebaseio.com/{}.json?auth={}", self.upload_code, dotenv!("secret"))).json(&upload_string).send().unwrap();
                        self.uploaded = true;
                    }
                });

                ui.separator();

                // Download song menu
                ui.vertical(|ui| {
                    ui.set_min_size(egui::Vec2::new(400.0, 60.0));
                    ui.set_max_size(egui::Vec2::new(400.0, 60.0));

                    ui.heading("Download");
                    ui.horizontal(|ui| {
                        // Confirm CustomLevels folder
                        if self.download_warning {
                            ui.label("Are you sure your CustomLevels folder is selected?");
                            if ui.add(egui::Button::new("Yes")).clicked() {
                                self.download();
                            } else if ui.add(egui::Button::new("No")).clicked() {
                                self.download_warning = false;
                            }
                        // ID input and download button
                        } else {
                            // ID Entry
                            if self.download_error {
                                ui.add(egui::TextEdit::singleline(&mut self.download_code_buf).hint_text("Bad ID").desired_width(75.0));
                            } else {
                                ui.add(egui::TextEdit::singleline(&mut self.download_code_buf).hint_text("Enter ID").desired_width(75.0));
                            }
                            // Download song button
                            if ui.add(egui::Button::new("Download Songs")).clicked() {
                                // Grab ID
                                match self.download_code_buf.parse::<u8>() {
                                    Ok(code) => {
                                        self.download_code = code;
                                        self.download_error = false;

                                        // Run check if folder is not named properly
                                        if self.custom_level_path.file_name().unwrap().to_str().unwrap() == "CustomLevels" {
                                            self.download()
                                        } else {
                                            self.download_warning = true;
                                        }
                                    },
                                    Err(_) => {
                                        self.download_error = true;
                                        self.download_code_buf = String::from("");
                                    },
                                }
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("Download Limit: ");
                        ui.add(egui::Slider::new(&mut self.thread_count, 1..=std::thread::available_parallelism().unwrap().get() / 2).text(""));
                    });
                });
            });
        });

        // Downloads popup
        if self.downloading {
            let guard = self.mutex.lock().unwrap();
            let downloading_songs = (*guard).clone();
            drop(guard);
            egui::TopBottomPanel::bottom("Downloads").show(ctx, |ui| {
                ui.heading("Downloading");
                if downloading_songs.is_empty() {
                    ui.label("Done");
                } else {
                    for song in downloading_songs {
                        ui.label(song);
                    }
                }
            });
        }
    }
}

trait StringUtils {
    fn substring(&self, start: usize, len: usize) -> Self;
}

impl StringUtils for String {
    fn substring(&self, start: usize, len: usize) -> Self {
        self.chars().skip(start).take(len - start).collect()
    }
}


fn get_codes(path: PathBuf) -> Vec<String> {
    let dir = path.read_dir().unwrap();
    let mut codes: Vec<String> = Vec::new();

    for entry in dir {

        let full_path = entry.unwrap().path();

        let filename = full_path.file_name().unwrap();

        let end = match filename.to_str().unwrap().find(" (")
        {
            Some(t) => {t},
            None => {continue},
        };
        
        let code = filename.to_str().unwrap().to_string().substring(0, end);

        if code.chars().count() <= 5
        {
            codes.push(code);
        }
    }

    codes
}

fn download_songs(download_list: Vec<String>, download_path: PathBuf, mutex: Arc<Mutex<Vec<String>>>) {
    let mut failed_songs = Vec::new();

    for song in download_list {
        // get https://api.beatsaver.com/maps/id/{song}
        if let Ok(contents) = reqwest::blocking::get(format!("https://api.beatsaver.com/maps/id/{}", song)).unwrap().text() {
            // download versions[0].downloadURL
            if let Some(dl_start) = contents.find("downloadURL") {
                let dl_end = contents[dl_start..contents.len() - 1].find(',').unwrap() + dl_start;
                let dl_link = &contents[dl_start + 15..dl_end - 1];

                let name_start = contents.find("songName").unwrap();
                let name_end = contents[name_start..contents.len() - 1].find(',').unwrap() + name_start;
                let name = &contents[name_start + 12..name_end - 1];

                let author_start = contents.find("levelAuthorName").unwrap();
                let author_end = contents[author_start..contents.len() - 1].find(',').unwrap() + author_start;
                let author = &contents[author_start + 19..author_end - 7];

                let full_name = format!("{} ({} - {})", song, name, author);

                let mut guard = mutex.lock().unwrap();
                (*guard).push(full_name.clone());
                drop(guard);

                let mut dl_song = reqwest::blocking::get(dl_link).unwrap();

                let mut file = File::create(download_path.clone().join(format!("{}.zip", full_name.clone()))).unwrap();
                copy(&mut dl_song, &mut file).unwrap();
                drop(file);

                //unzip
                let file = File::open(download_path.clone().join(format!("{}.zip", full_name.clone()))).unwrap();

                fs::create_dir(download_path.clone().join(full_name.clone())).unwrap();
                let mut zip = ZipArchive::new(file).unwrap();
                zip.extract(download_path.clone().join(full_name.clone())).unwrap();
                drop(zip);

                fs::remove_file(download_path.clone().join(format!("{}.zip", full_name.clone()))).unwrap();

                let mut guard = mutex.lock().unwrap();
                let list = (*guard).clone();
                for (i, s) in list.iter().enumerate() {
                    if s == &full_name {
                        (*guard).remove(i);
                        break;
                    }
                }
                drop(guard);

            } else {
                failed_songs.push(song);
            }
        } else {
            failed_songs.push(song);
        }
    }
}