pub mod bsaber;
pub mod db;

#[derive(Debug)]
pub enum APIErr {
    InvalidID,
    IDNotFound,
    ReqwestFailed,
}
