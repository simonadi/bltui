use tokio::sync::oneshot::Sender;
use zbus::fdo::Error;

#[derive(Debug)]
pub enum AgentEvent {
    Release {
        tx: Sender<Result<(), Error>>,
    },
    RequestPincode {
        tx: Sender<Result<String, Error>>,
    },
    DisplayPincode {
        pincode: String,
        tx: Sender<Result<(), Error>>,
    },
    RequestPasskey {
        tx: Sender<Result<u32, Error>>,
    },
    DisplayPasskey {
        passkey: u32,
        tx: Sender<Result<(), Error>>,
    },
    RequestConfirmation {
        passkey: u32,
        tx: Sender<Result<(), Error>>,
    },
    RequestAuthorization {
        tx: Sender<Result<(), Error>>,
    },
    AuthorizeService {
        uuid: String,
        tx: Sender<Result<(), Error>>,
    },
    Cancel {
        tx: Sender<Result<(), Error>>,
    },
}
