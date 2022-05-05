use std::io;
use std::num::ParseIntError;
use zip::result::ZipError;

pub mod beatsaver;
pub mod db;

#[derive(Debug)]
pub enum APIErr {
    InvalidIndex,
    IndexNotFound,
    ReqwestFailed,
    SongNotFound,
    FileCreationFailed,
    InvalidText,
    UnzipFailed,
}

macro_rules! impl_from_error_to_api_err {
    ($($from: ty, $err: expr),+) => {
        $(
            impl From<$from> for APIErr {
                fn from(_: $from) -> Self {
                    $err
                }
            }
        )+
    };
}

impl_from_error_to_api_err! {
    reqwest::Error, APIErr::ReqwestFailed,
    ParseIntError, APIErr::InvalidIndex,
    io::Error, APIErr::FileCreationFailed,
    ZipError, APIErr::UnzipFailed
}
