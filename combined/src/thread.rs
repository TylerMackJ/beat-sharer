use crate::api::*
use std::path::PathBuf;

pub fn download_thread(id: String, path: PathBuf) -> Result<(), APIErr> {
    let song_info = api::beatsaver::get_song_info(id)?;
    api::beatsaver::download_and_unzip_song(song_info, path)?;
    Ok(())
}

pub fn spawn_threads(id_list: Vec<String>, path: PathBuf) -> Arc<AtomicUsize> {
    for id in id_list {
        download_thread(id, path);
    }

    todo!()
}