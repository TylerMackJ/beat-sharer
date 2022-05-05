pub mod bsaber;
pub mod db;

#[derive(Debug)]
pub enum APIErr {
    InvalidID,
    IDNotFound,
    ReqwestFailed,
    SongNotFound,
    FileCreationFailed,
}

#[macro_export]
macro_rules! check_id {
    ($id: expr) => {
        if $id.parse::<u8>().is_err() {
            return Err(APIErr::InvalidID);
        }
    };
}
