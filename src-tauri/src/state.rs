use tokio::sync::mpsc;

#[derive(Clone)]
pub struct AppState {
    pub p2p_sender: mpsc::Sender<String>,
}