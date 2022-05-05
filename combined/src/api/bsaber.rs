use crate::api::*;
use crate::check_id;

pub struct SongInfo {
    id: String,
    name: String,
    author: String,
    download_url: String,
}

const BSABER_ADDR: &str = "https://api.beatsaver.com";

pub fn get_song_info(id: String) -> Result<SongInfo, APIErr> {
    check_id!(id);
    let addr = format!("{}/maps/id/{}", BSABER_ADDR, id);
    let contents = match reqwest::blocking::get(addr) {
        Ok(con) => match con.text() {
            Ok(con) => con,
            Err(_) => return Err(APIErr::ReqwestFailed),
        },
        Err(_) => return Err(APIErr::ReqwestFailed),
    };

    let download_url;
    let name;
    let author;

    if let Some(dl_start) = contents.find("downloadURL") {
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
    } else {
        return Err(APIErr::SongNotFound);
    }

    Ok(SongInfo {
        id,
        name,
        download_url,
        author,
    })
}
