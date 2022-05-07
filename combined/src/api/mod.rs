use futures::stream::FuturesUnordered;
use futures::StreamExt;
use lazy_static::lazy_static;
use std::io;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};
use tokio::sync::oneshot;
use zip::result::ZipError;

mod beatsaver;
mod db;

const SEND_UNWRAP_FAILURE_MESSAGE: &str =
    "failed to send resulting value, was the receiver dropped?";
const POISONED_MUTEX_MESSAGE: &str =
    "failed to unlock mutex due to another thread panicking while holding it";

lazy_static! {
    static ref ASYNC_RUNTIME: tokio::runtime::Runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
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

pub fn download(
    id_list: Vec<String>,
    dir: PathBuf,
    max_concurrent_downloads: NonZeroUsize,
) -> DownloadObserver {
    let (updater, observer) = SharedInfo::create(max_concurrent_downloads);
    ASYNC_RUNTIME.spawn(download_list_async(id_list, dir, updater));
    observer
}

async fn download_list_async(mut id_list: Vec<String>, dir: PathBuf, updater: DownloadUpdater) {
    let mut handles = FuturesUnordered::new();
    while let Some(id) = id_list.pop() {
        let handle = tokio::spawn(download_async(id.clone(), dir.clone()));
        updater.add_ongoing_download(id);
        handles.push(handle);

        while handles.len() > updater.get_max_concurrent_downloads().get() {
            if let Some(result) = handles.next().await {
                match result {
                    Ok((id, Ok(_))) => {
                        updater.increment_downloaded().await;
                        updater.remove_ongoing_download(id);
                    }
                    Ok((id, Err(err))) => updater.add_failure(id, err).await,
                    Err(err) => panic!("error joining with download task: {}", err),
                }
            }
        }
    }
}

async fn download_async(id: String, dir: PathBuf) -> (String, Result<(), APIErr>) {
    (id.clone(), download_async_inner(id, dir).await)
}

async fn download_async_inner(id: String, dir: PathBuf) -> Result<(), APIErr> {
    let song_info = beatsaver::get_song_info(id).await?;
    beatsaver::download_and_unzip_song(song_info, dir).await?;
    Ok(())
}

#[derive(Clone)]
pub struct DownloadObserver {
    info: Arc<SharedInfo>,
}

impl DownloadObserver {
    pub fn get_downloaded(&self) -> usize {
        self.info.downloaded.load(Ordering::Acquire)
    }

    pub fn ongoing_downloads(&self) -> Vec<String> {
        self.info
            .ongoing_downloads
            .lock()
            .expect(POISONED_MUTEX_MESSAGE)
            .clone()
    }

    pub fn failed_downloads(&self) -> Vec<(String, APIErr)> {
        self.info
            .failed_downloads
            .lock()
            .expect(POISONED_MUTEX_MESSAGE)
            .clone()
    }

    pub fn set_max_concurrent_downloads(&self, n: NonZeroUsize) {
        self.info
            .max_concurrent_downloads
            .store(n.get(), Ordering::Release);
    }
}

#[derive(Clone)]
pub struct DownloadUpdater {
    info: Arc<SharedInfo>,
}

impl DownloadUpdater {
    pub async fn increment_downloaded(&self) {
        self.info.downloaded.fetch_add(1, Ordering::Release);
    }

    pub fn add_ongoing_download(&self, id: String) {
        self.info
            .ongoing_downloads
            .lock()
            .expect(POISONED_MUTEX_MESSAGE)
            .push(id);
    }

    pub fn remove_ongoing_download(&self, id: String) {
        let mut ongoing_downloads = self
            .info
            .ongoing_downloads
            .lock()
            .expect(POISONED_MUTEX_MESSAGE);
        if let Some(index) = ongoing_downloads
            .iter()
            .position(|ongoing_id| ongoing_id == &id)
        {
            ongoing_downloads.swap_remove(index);
        }
    }

    pub async fn add_failure(&self, id: String, err: APIErr) {
        self.info
            .failed_downloads
            .lock()
            .expect(POISONED_MUTEX_MESSAGE)
            .push((id, err));
    }

    pub fn get_max_concurrent_downloads(&self) -> NonZeroUsize {
        let max_threads = self.info.max_concurrent_downloads.load(Ordering::Acquire);
        // the setters for max_concurrent_downloads only allow setting it to a NonZeroUsize, so unwrap is ok here
        NonZeroUsize::new(max_threads).unwrap()
    }

    pub fn downloading(&self) -> bool {
        todo!()
    }
}

struct SharedInfo {
    downloaded: AtomicUsize,
    ongoing_downloads: Mutex<Vec<String>>,
    failed_downloads: Mutex<Vec<(String, APIErr)>>,
    max_concurrent_downloads: AtomicUsize,
}

impl SharedInfo {
    fn new(max_concurrent_downloads: NonZeroUsize) -> Self {
        Self {
            downloaded: Default::default(),
            ongoing_downloads: Default::default(),
            failed_downloads: Default::default(),
            max_concurrent_downloads: AtomicUsize::new(max_concurrent_downloads.get()),
        }
    }

    fn create(max_concurrent_downloads: NonZeroUsize) -> (DownloadUpdater, DownloadObserver) {
        let shared = Arc::new(SharedInfo::new(max_concurrent_downloads));
        (
            DownloadUpdater {
                info: shared.clone(),
            },
            DownloadObserver { info: shared },
        )
    }
}

#[derive(Clone, Debug)]
pub struct SongInfo {
    id: String,
    name: String,
    author: String,
    download_url: String,
}

#[derive(Debug, Clone)]
pub enum APIErr {
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
    io::Error, APIErr::FileCreationFailed,
    ZipError, APIErr::UnzipFailed
}
