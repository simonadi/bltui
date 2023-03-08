use std::{
    fmt::{Debug, Display, Formatter},
    time::Duration,
};

use tokio::{
    sync::{mpsc::Sender, oneshot},
    time::timeout,
};
use zbus::{dbus_interface, Connection};

use crate::{devices::MacAddress, AppEvent};
use log::debug;
use zbus::DBusError;

static TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, DBusError, PartialEq, Eq)]
#[dbus_error(prefix = "org.bluez.Error", impl_display = true)]
pub enum BluezError {
    Rejected(String),
    Canceled(String),
}

pub type Responder<T> = oneshot::Sender<Result<T, BluezError>>;

#[derive(Debug)]
pub enum AgentEvent {
    Release {
        tx: Responder<()>,
    },
    RequestPincode {
        tx: Responder<String>,
        address: MacAddress,
    },
    DisplayPincode {
        pincode: String,
        tx: Responder<()>,
        address: MacAddress,
    },
    RequestPasskey {
        tx: Responder<u32>,
        address: MacAddress,
    },
    DisplayPasskey {
        passkey: u32,
        tx: Responder<()>,
        address: MacAddress,
    },
    RequestConfirmation {
        passkey: u32,
        tx: Responder<()>,
        address: MacAddress,
    },
    RequestAuthorization {
        tx: Responder<()>,
        address: MacAddress,
    },
    AuthorizeService {
        uuid: String,
        tx: Responder<()>,
        address: MacAddress,
    },
    Cancel {
        tx: Responder<()>,
    },
}

struct AgentServer {
    tx: Sender<AppEvent>,
}

#[dbus_interface(name = "org.bluez.Agent1")]
impl AgentServer {
    async fn release(&self) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::Release { tx }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn request_pin_code(
        &self,
        device: zvariant::ObjectPath<'_>,
    ) -> Result<String, BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestPincode {
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn display_pin_code(
        &self,
        device: zvariant::ObjectPath<'_>,
        pincode: String,
    ) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::DisplayPincode {
                pincode,
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn request_passkey(&self, device: zvariant::ObjectPath<'_>) -> Result<u32, BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestPasskey {
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn display_passkey(
        &self,
        device: zvariant::ObjectPath<'_>,
        passkey: u32,
        _entered: u16,
    ) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::DisplayPasskey {
                passkey,
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn request_confirmation(
        &self,
        device: zvariant::ObjectPath<'_>,
        passkey: u32,
    ) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestConfirmation {
                passkey,
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        debug!("Sent request for confirmation");
        let result = timeout(TIMEOUT, rx).await.unwrap().unwrap();
        debug!("Received request for confirmation input");
        result
    }

    async fn request_authorization(
        &self,
        device: zvariant::ObjectPath<'_>,
    ) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestAuthorization {
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn authorize_service(
        &self,
        device: zvariant::ObjectPath<'_>,
        uuid: String,
    ) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::AuthorizeService {
                uuid,
                tx,
                address: MacAddress::from_dbus_path(device),
            }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }

    async fn cancel(&self) -> Result<(), BluezError> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::Cancel { tx }))
            .await
            .unwrap();
        timeout(TIMEOUT, rx).await.unwrap().unwrap()
    }
}

#[derive(Debug)]
pub enum AgentCapability {
    DisplayOnly,
    DisplayYesNo,
    KeyboardDisplay,
    KeyboardOnly,
    NoInputNoOutput,
}

impl Display for AgentCapability {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        Debug::fmt(self, f)
    }
}

pub struct Agent<'a> {
    path: zvariant::ObjectPath<'a>,
    // name: String,
    // tx: Sender<AppEvent>,
    capability: AgentCapability,
    connection: Connection,
}

impl Agent<'_> {
    pub async fn initialize_dbus_connection(
        path: String,
        capability: AgentCapability,
    ) -> Agent<'static> {
        let connection = Connection::system().await.unwrap();

        Agent {
            path: zvariant::ObjectPath::try_from(path).unwrap(),
            capability,
            connection,
        }
    }

    pub async fn request_name(&self, name: &str) {
        self.connection.request_name(name).await.unwrap();
    }

    // pub async fn register_as_default(&self) {
    //     self.start_server().await;
    //     self.register().await;
    //     self.request_default().await;
    // }

    pub async fn register(&self) {
        self.connection
            .call_method(
                Some("org.bluez"),
                "/org/bluez",
                Some("org.bluez.AgentManager1"),
                "RegisterAgent",
                &(self.path.clone(), self.capability.to_string()),
            )
            .await
            .unwrap();

        debug!("Registered the agent");
    }

    pub async fn request_default(&self) {
        self.connection
            .call_method(
                Some("org.bluez"),
                "/org/bluez",
                Some("org.bluez.AgentManager1"),
                "RequestDefaultAgent",
                &(self.path.clone(),),
            )
            .await
            .unwrap();

        debug!("Requested default agent");
    }

    pub async fn unregister(&self) {
        self.connection
            .call_method(
                Some("org.bluez"),
                "/org/bluez",
                Some("org.bluez.AgentManager1"),
                "UnregisterAgent",
                &(self.path.clone(),),
            )
            .await
            .unwrap();

        debug!("Unregistered the agent");
    }

    pub async fn start_server(&self, tx: Sender<AppEvent>) {
        self.connection
            .object_server()
            .at(self.path.clone(), AgentServer { tx: tx.clone() })
            .await
            .unwrap();

        debug!("Started the agent server")
    }
}
