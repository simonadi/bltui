use bluetooth::devices::Devices;
use events::AppEvent;
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use ui::widgets::popup::YesNoPopup;

pub mod bluetooth;
pub mod events;
pub mod logging;
pub mod settings;
pub mod ui;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error in BT stuff")]
    BluetoothError(#[from] btleplug::Error),
    #[error("Invalid input : {}", .0)]
    InvalidInput(String),
    #[error("Failed parsing the config file at {:?}", .0)]
    InvalidConfigFile(PathBuf),
    #[error("IO Error")]
    IOError(#[from] std::io::Error),
}

pub struct App {
    pub devices: Devices,
    pub popup: Option<YesNoPopup>,
    tx: Sender<AppEvent>,
    rx: Receiver<AppEvent>,
}

#[allow(clippy::new_without_default)]
impl App {
    pub fn new() -> App {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        App {
            devices: Devices::new(),
            popup: None,
            tx,
            rx,
        }
    }

    pub fn tx(&self) -> Sender<AppEvent> {
        self.tx.clone()
    }

    pub async fn events(&mut self) -> Option<AppEvent> {
        self.rx.recv().await
    }
}
