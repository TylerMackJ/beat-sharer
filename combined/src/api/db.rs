use crate::api::*;
use crate::util::StringUtils;

const DB_ADDR: &str = "https://beat-sharer-default-rtdb.firebaseio.com";

pub fn get_list(index: String) -> Result<Vec<String>, APIErr> {
    check_index(&index)?;

    let auth = dotenv!("secret");
    let addr = format!("{}/{}.json?auth={}", DB_ADDR, index, auth);
    let response = reqwest::blocking::get(addr)?;
    // can't use From here since that would just map to APIErr::ReqwestFailed
    let mut contents = response.text().map_err(|_| APIErr::InvalidText)?;

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

pub fn put_list(index: String, list: String) -> Result<(), APIErr> {
    check_index(&index)?;

    let client = reqwest::blocking::Client::new();
    client
        .put(format!(
            "{}/{}.json?auth={}",
            DB_ADDR,
            index,
            dotenv!("secret")
        ))
        .json(&list)
        .send()?;
    Ok(())
}

/// returns Ok if index is parsable to a u8, else API::InvalidIndex
#[inline(always)]
fn check_index(index: &String) -> Result<(), APIErr> {
    index.parse::<u8>()?;
    Ok(())
}

fn get_index() -> Result<u8, APIErr> {
    let auth = dotenv!("secret");
    let addr = format!("{}/index.json?auth={}", DB_ADDR, auth);
    let response = reqwest::blocking::get(addr)?;
    // can't use From here since that would just map to APIErr::ReqwestFailed
    let contents = response.text().map_err(|_| APIErr::InvalidText)?;

    Ok(contents.parse::<u8>().unwrap())
}

fn put_index(index: u8) -> Result<(), APIErr> {
    let client = reqwest::blocking::Client::new();
    client
        .put(format!("{}/index.json?auth={}", DB_ADDR, dotenv!("secret")))
        .json(&index.to_string())
        .send()?;
    Ok(())
}

pub fn get_and_inc_index() -> Result<u8, APIErr> {
    let index = get_index()?;
    put_index(index.wrapping_add(1))?;
    Ok(index)
}
