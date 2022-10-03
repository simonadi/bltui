use std::{
    fmt::{Debug, Display, Formatter},
    time::Duration,
};

use tokio::{
    sync::{mpsc::Sender, oneshot},
    time::timeout,
};
use zbus::{dbus_interface, Connection};

use crate::events::{agent::AgentEvent, AppEvent};
use log::debug;
use zbus::fdo::Error;

struct AgentServer {
    tx: Sender<AppEvent>,
}

#[dbus_interface(name = "org.bluez.Agent1")]
impl AgentServer {
    async fn release(&self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::Release { tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn request_pin_code(
        &self,
        _device: zvariant::ObjectPath<'_>,
    ) -> Result<String, zbus::fdo::Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestPincode { tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn display_pin_code(
        &self,
        _device: zvariant::ObjectPath<'_>,
        pincode: String,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::DisplayPincode { pincode, tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn request_passkey(&self, _device: zvariant::ObjectPath<'_>) -> Result<u32, Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestPasskey { tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn display_passkey(
        &self,
        _device: zvariant::ObjectPath<'_>,
        passkey: u32,
        _entered: u16,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::DisplayPasskey { passkey, tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn request_confirmation(
        &self,
        _device: zvariant::ObjectPath<'_>,
        passkey: u32,
    ) -> Result<(), zbus::fdo::Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestConfirmation {
                passkey,
                tx,
            }))
            .await
            .unwrap();
        debug!("Sent request for confirmation");
        let result = timeout(Duration::from_secs(20), rx).await.unwrap().unwrap();
        debug!("Received request for confirmation input");
        result
    }

    async fn request_authorization(&self, _device: zvariant::ObjectPath<'_>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::RequestAuthorization { tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn authorize_service(
        &self,
        _device: zvariant::ObjectPath<'_>,
        uuid: String,
    ) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::AuthorizeService { uuid, tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
    }

    async fn cancel(&self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        self.tx
            .send(AppEvent::Agent(AgentEvent::Cancel { tx }))
            .await
            .unwrap();
        timeout(Duration::from_secs(20), rx).await.unwrap().unwrap()
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
    capability: AgentCapability,
    connection: Connection,
}

impl Agent<'static> {
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
            .at("/bltui/agent", AgentServer { tx })
            .await
            .unwrap();

        debug!("Started the agent server")
    }
}
