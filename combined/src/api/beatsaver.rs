use crate::api::*;
use std::io::Cursor;
use std::path::PathBuf;
use zip::read::ZipArchive;

impl std::fmt::Display for SongInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({} - {})", self.id, self.name, self.author)
    }
}

const BSABER_ADDR: &str = "https://api.beatsaver.com";

pub(in crate::api) async fn get_song_info(id: String) -> Result<SongInfo, APIErr> {
    let addr = format!("{}/maps/id/{}", BSABER_ADDR, id);
    let response = reqwest::get(addr).await?;
    let contents = response.text().await.map_err(|_| APIErr::InvalidText)?;

    let download_url;
    let name;
    let author;

    // todo a closer look on this, not sure how the responses get formatted so I can't really
    // modify this stuff without oversight or testing

    // this may be able to take advantage of String::split
    // there's a lot of unwrapping on network returns
    {
        let dl_start = contents.find("downloadURL").ok_or(APIErr::SongNotFound)?;
        let dl_end = contents[dl_start..contents.len() - 1].find(',').unwrap() + dl_start;
        download_url = contents[dl_start + 15..dl_end - 1].to_string();

        let name_start = contents.find("songName").unwrap();
        let name_end = contents[name_start..contents.len() - 1].find(',').unwrap() + name_start;
        name = contents[name_start + 12..name_end - 1].to_string();

        let author_start = contents.find("levelAuthorName").unwrap();
        let author_end = contents[author_start..contents.len() - 1]
            .find(',')
            .unwrap()
            + author_start;
        author = contents[author_start + 19..author_end - 7].to_string();
    }

    Ok(SongInfo {
        id,
        name,
        download_url,
        author,
    })
}

async fn download_song(song_info: &SongInfo) -> Result<Vec<u8>, APIErr> {
    let response = reqwest::get(&song_info.download_url).await?;
    // the .bytes call is untested here, not sure if it converts the entire HTTP GET response into
    // bytes or just the data we need here
    let mut bytes_in = &response.bytes().await?.to_vec()[..];
    let mut bytes_out = Vec::new();
    tokio::io::copy(&mut bytes_in, &mut bytes_out).await?;
    Ok(bytes_out)
}

fn unzip_song(song_info: SongInfo, bytes: Vec<u8>, dir: PathBuf) -> Result<(), APIErr> {
    let song_path = dir.clone().join(PathBuf::from(song_info.to_string()));
    std::fs::create_dir(song_path.clone())?;
    let mut zip = ZipArchive::new(Cursor::new(bytes))?;
    zip.extract(song_path.clone())?;
    Ok(())
}

pub(in crate::api) async fn download_and_unzip_song(
    song_info: SongInfo,
    dir: PathBuf,
) -> Result<(), APIErr> {
    let bytes = download_song(&song_info).await?;
    // if unzip fails, look at download_song for a note
    unzip_song(song_info, bytes, dir)?;
    Ok(())
}
