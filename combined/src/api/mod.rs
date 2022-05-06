use std::{io, thread};
use std::num::ParseIntError;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use tokio::sync::oneshot;
use zip::result::ZipError;

pub mod beatsaver;
pub mod db;

// todo remove reqwest::Client initializations everywhere and create a lazy static one here

const SEND_UNWRAP_FAILURE_MESSAGE: &str = "failed to send resulting value, was the receiver dropped?";

lazy_static! {
    static ref ASYNC_RUNTIME: tokio::runtime::Runtime =
        tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .build()
            .unwrap();
}

pub fn get_list(index: u8) -> oneshot::Receiver<Result<Vec<String>, APIErr>> {
    let (sender, receiver) = oneshot::channel();
    async fn f(sender: oneshot::Sender<Result<Vec<String>, APIErr>>, index: u8) {
        let result = db::get_list(index).await;
        sender.send(result).expect(SEND_UNWRAP_FAILURE_MESSAGE);
    }
    ASYNC_RUNTIME.spawn(f(sender, index));
    receiver
}

pub fn put_list(index: u8, list: String) -> oneshot::Receiver<Result<(), APIErr>> {
    let (sender, receiver) = oneshot::channel();
    async fn f(sender: oneshot::Sender<Result<(), APIErr>>, index: u8, list: String) {
        let result = db::put_list(index, list).await;
        sender.send(result).expect(SEND_UNWRAP_FAILURE_MESSAGE);
    }
    ASYNC_RUNTIME.spawn(f(sender, index, list));
    receiver
}

pub fn get_and_inc_index() -> oneshot::Receiver<Result<u8, APIErr>> {
    let (sender, receiver) = oneshot::channel();
    async fn f(sender: oneshot::Sender<Result<u8, APIErr>>) {
        let result = db::get_and_inc_index().await;
        sender.send(result).expect(SEND_UNWRAP_FAILURE_MESSAGE);
    }
    ASYNC_RUNTIME.spawn(f(sender));
    receiver
}

#[derive(Clone)]
pub struct DownloadObserver {
    info: Arc<SharedInfo>
}

impl DownloadObserver {
    pub fn get_downloaded(&self) -> usize {
        self.info.downloaded.load(Ordering::Acquire)
    }

    pub fn set_max_threads(&self, n: usize) {
        self.info.max_concurrent_downloads.store(n, Ordering::Release);
    }
}

pub struct DownloadUpdater {
    info: Arc<SharedInfo>
}

impl DownloadUpdater {
    pub fn increment_downloaded(&self) {
        self.info.downloaded.fetch_add(1, Ordering::Release);
    }

    pub fn get_max_threads(&self) -> usize {
        self.info.max_concurrent_downloads.load(Ordering::Acquire)
    }
}

#[derive(Default)]
struct SharedInfo {
    downloaded: AtomicUsize,
    // ongoing_downloads: Mutex<Vec<?>>
    // failed_downloads: Mutex<Vec<?>>,
    max_concurrent_downloads: AtomicUsize
}

impl SharedInfo {
    fn create() -> (DownloadUpdater, DownloadObserver) {
        let shared = Arc::new(SharedInfo::default());
        (
            DownloadUpdater { info: shared.clone() },
            DownloadObserver { info: shared }
        )
    }
}

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
