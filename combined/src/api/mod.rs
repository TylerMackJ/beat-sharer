use std::io;
use reqwest::Error;
use std::num::ParseIntError;

pub mod bsaber;
pub mod db;

#[derive(Debug)]
pub enum APIErr {
    InvalidIndex,
    IndexNotFound,
    ReqwestFailed,
    SongNotFound,
    FileCreationFailed,
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

impl From<io::Error> for APIErr {
    fn from(_: io::Error) -> Self {
        APIErr::FileCreationFailed
    }
}