use crate::api::*;
use crate::util::StringUtils;

const DB_ADDR: &str = "https://beat-sharer-default-rtdb.firebaseio.com";

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();
}

pub(in crate::api) async fn get_list(index: u8) -> Result<Vec<String>, APIErr> {
    let auth = dotenv!("secret");
    let addr = format!("{}/{}.json?auth={}", DB_ADDR, index, auth);
    let response = reqwest::get(addr).await?;
    // can't use implicit `?` From here since that would just map to APIErr::ReqwestFailed
    let mut contents = response.text().await.map_err(|_| APIErr::InvalidText)?;

    if contents == "null" {
        return Err(APIErr::IndexNotFound);
    }

    contents = contents.substring(1, contents.chars().count() - 2);
    let mut list = Vec::new();
    for code in contents.split(',') {
        list.push(String::from(code));
    }

    Ok(list)
}

pub(in crate::api) async fn put_list(index: u8, list: String) -> Result<(), APIErr> {
    CLIENT
        .put(format!(
            "{}/{}.json?auth={}",
            DB_ADDR,
            index,
            dotenv!("secret")
        ))
        .json(&list)
        .send()
        .await?;
    Ok(())
}

async fn get_index() -> Result<u8, APIErr> {
    let auth = dotenv!("secret");
    let addr = format!("{}/index.json?auth={}", DB_ADDR, auth);
    let response = reqwest::get(addr).await?;
    // can't use From here since that would just map to APIErr::ReqwestFailed
    let mut contents = response.text().await.map_err(|_| APIErr::InvalidText)?;
    contents = contents.substring(1, contents.len() - 1);
    Ok(contents.parse::<u8>().unwrap())
}

async fn put_index(index: u8) -> Result<(), APIErr> {
    CLIENT
        .put(format!("{}/index.json?auth={}", DB_ADDR, dotenv!("secret")))
        .json(&index.to_string())
        .send()
        .await?;
    Ok(())
}

pub(in crate::api) async fn get_and_inc_index() -> Result<u8, APIErr> {
    let index = get_index().await?;
    put_index(index.wrapping_add(1)).await?;
    Ok(index)
}
