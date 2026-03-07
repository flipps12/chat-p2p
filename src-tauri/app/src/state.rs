use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub p2p_sender: mpsc::Sender<String>,
    // pub file_manager: Arc<Mutex<FileManager>>,
}