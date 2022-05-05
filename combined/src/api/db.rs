use crate::api::*;
use crate::util::StringUtils;

macro_rules! check_id {
    ($id: expr) => {
        if $id.parse::<u8>().is_err() {
            return Err(APIErr::InvalidID);
        }
    };
}

const DB_ADDR: &str = "https://beat-sharer-default-rtdb.firebaseio.com";

pub fn get_list(id: String) -> Result<Vec<String>, APIErr> {
    check_id!(id);

    let auth = dotenv!("secret");
    let addr = format!("{}/{}.json?auth={}", DB_ADDR, id, auth);

    let mut contents = match reqwest::blocking::get(addr) {
        Ok(con) => con.text().unwrap(),
        Err(_) => return Err(APIErr::ReqwestFailed),
    };

    if contents == "null" {
        return Err(APIErr::IDNotFound);
    }

    contents = contents.substring(1, contents.chars().count() - 2);
    let mut list = Vec::new();
    for code in contents.split(',') {
        list.push(String::from(code));
    }

    Ok(list)
}

pub fn put_list(id: String, list: String) -> Result<(), APIErr> {
    check_id!(id);

    let client = reqwest::blocking::Client::new();
    return match client
        .put(format!(
            "{}/{}.json?auth={}",
            DB_ADDR,
            id,
            dotenv!("secret")
        ))
        .json(&list)
        .send()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(APIErr::ReqwestFailed),
    };
}

fn get_index() -> Result<u8, APIErr> {
    let auth = dotenv!("secret");
    let addr = format!("{}/index.json?auth={}", DB_ADDR, auth);
    let contents = match reqwest::blocking::get(addr) {
        Ok(con) => con.text().unwrap(),
        Err(_) => return Err(APIErr::ReqwestFailed),
    };

    Ok(contents.parse::<u8>().unwrap())
}

fn put_index(index: u8) -> Result<(), APIErr> {
    let client = reqwest::blocking::Client::new();
    let res = client
        .put(format!("{}/index.json?auth={}", DB_ADDR, dotenv!("secret")))
        .json(&index.to_string())
        .send();

    if res.is_err() {
        return Err(APIErr::ReqwestFailed);
    }

    Ok(())
}

pub fn get_and_inc_index() -> Result<u8, APIErr> {
    let index = match get_index() {
        Ok(i) => i,
        Err(e) => return Err(e),
    };
    match put_index(index + 1) {
        Ok(()) => Ok(index),
        Err(e) => Err(e),
    }
}
