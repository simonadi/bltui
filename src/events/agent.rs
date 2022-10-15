use crate::bluetooth::agent::BluezError;
use tokio::sync::oneshot::Sender;

type Responder<T> = Sender<Result<T, BluezError>>;

#[derive(Debug)]
pub enum AgentEvent {
    Release { tx: Responder<()> },
    RequestPincode { tx: Responder<String> },
    DisplayPincode { pincode: String, tx: Responder<()> },
    RequestPasskey { tx: Responder<u32> },
    DisplayPasskey { passkey: u32, tx: Responder<()> },
    RequestConfirmation { passkey: u32, tx: Responder<()> },
    RequestAuthorization { tx: Responder<()> },
    AuthorizeService { uuid: String, tx: Responder<()> },
    Cancel { tx: Responder<()> },
}
