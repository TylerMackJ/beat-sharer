use std::num::ParseIntError;
use reqwest::Error;

pub mod bsaber;
pub mod db;

#[derive(Debug)]
pub enum APIErr {
    InvalidIndex,
    IndexNotFound,
    ReqwestFailed,
    InvalidText,
}

impl From<reqwest::Error> for APIErr {
    fn from(_: Error) -> Self {
        APIErr::ReqwestFailed
    }
}

impl From<ParseIntError> for APIErr {
    fn from(_: ParseIntError) -> Self {
        APIErr::InvalidIndex
    }
}