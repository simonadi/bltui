use bluetooth::devices::Devices;
use events::AppEvent;
use tokio::sync::mpsc::{Receiver, Sender};
use ui::widgets::popup::YesNoPopup;

pub mod bluetooth;
pub mod events;
pub mod logging;
pub mod ui;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error in BT stuff")]
    BluetoothError(#[from] btleplug::Error),
    #[error("Invalid input : {}", .0)]
    InvalidInput(String),
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
