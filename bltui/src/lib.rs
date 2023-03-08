use bluetooth::{
    agent::{Agent, AgentCapability, AgentEvent},
    controller::{AdapterEvent, BluetoothController},
};
use devices::{Device, Devices, MacAddress};
use events::{spawn_adapter_watcher, spawn_keypress_watcher, spawn_ticker, AppEvent, UserAction};
use std::collections::HashSet;

use log::{error, info, trace, warn};
use settings::{AppSettings, LogSettings};
use std::{path::PathBuf, time::Duration};
use tokio::sync::mpsc::{Receiver, Sender};
use ui::AppRenderer;

pub mod bluetooth;
pub mod devices;
pub mod events;
pub mod settings;
pub mod ui;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref TICK_RATE: Duration = Duration::from_millis(16);
    static ref KEY_POLL_RATE: Duration = Duration::from_millis(8);
}

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
    #[error("Request cancelled")]
    RequestCancelled,
}

/// TODO : can't be doing that because the popup input box needs the actual keycode.

pub fn initialize_logging(log_settings: &LogSettings) -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log_settings.level).unwrap();
    tui_logger::set_default_level(log_settings.level);

    let mut log_file_path = log_settings.folder.clone();

    let datetime = time::OffsetDateTime::now_utc();
    log_file_path.push(
        datetime
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    );
    log_file_path.set_extension("log");

    if log_settings.log_to_file {
        std::fs::create_dir_all(&log_settings.folder)?;
        tui_logger::set_log_file(log_file_path.to_str().unwrap())?;
        info!("Logging to file {}", log_file_path.to_str().unwrap());
    }
    Ok(())
}

// struct Popup<R> {
//     tx: tokio::sync::oneshot::Sender<R>,
//     popup_widget: Widget,
// }

pub struct AppState {
    pub devices: Devices,
    pub selected_device: Option<MacAddress>,
    // pub popup: Option<dyn Popup>,
    pub scanning: bool,
    pub connecting: HashSet<MacAddress>,
    pub disconnecting: HashSet<MacAddress>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            devices: Devices::new(),
            selected_device: None,
            // popup: None,
            scanning: false,
            connecting: HashSet::new(),
            disconnecting: HashSet::new(),
        }
    }

    pub fn selected_device(&self) -> Option<Device> {
        self.selected_device
            .and_then(|addr| self.devices.get_by_mac_address(&addr))
    }
}

struct EventsChannel {
    tx: Sender<AppEvent>,
    rx: Receiver<AppEvent>,
}

impl EventsChannel {
    fn new() -> EventsChannel {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        EventsChannel { tx, rx }
    }

    fn tx(&self) -> Sender<AppEvent> {
        self.tx.clone()
    }
}

pub struct App<'a> {
    state: AppState,
    events_channel: EventsChannel,
    renderer: AppRenderer,
    settings: AppSettings,
    agent: Agent<'a>,
    bt_controller: BluetoothController,
}

#[allow(clippy::new_without_default)]
impl App<'_> {
    pub async fn init() -> App<'static> {
        let settings = AppSettings::parse();

        initialize_logging(&settings.log_settings).unwrap();

        let events_channel = EventsChannel::new();

        let bt_controller = if let Some(adapter) = &settings.adapter {
            BluetoothController::from_adapter(adapter)
                .await
                .expect("The requested adapter was not found")
        } else {
            BluetoothController::from_first_adapter().await
        };

        let agent = Agent::initialize_dbus_connection(
            "/bltui/agent".into(),
            AgentCapability::KeyboardDisplay,
        )
        .await;

        agent.start_server(events_channel.tx.clone()).await;
        agent.request_name("bltui.agent").await;
        agent.register().await;
        agent.request_default().await;

        spawn_ticker(*TICK_RATE, events_channel.tx.clone());

        spawn_keypress_watcher(events_channel.tx.clone());

        spawn_adapter_watcher(
            bt_controller.events().await.unwrap(),
            events_channel.tx.clone(),
        )
        .await;

        App {
            state: AppState::new(),
            events_channel,
            settings,
            agent,
            bt_controller,
            renderer: AppRenderer::initialize_terminal(),
        }
    }

    pub async fn run(&mut self) {
        while let Some(event) = self.events_channel.rx.recv().await {
            trace!("Received event : {:?}", event);
            // match &event {
            //     AppEvent::Tick => {}
            //     _ => {
            //         trace!("Received event : {:?}", event)
            //     }
            // }

            match event {
                AppEvent::Input(action) => {
                    self.handle_user_action(&action).await;
                }
                AppEvent::Tick => {
                    self.render();
                }
                AppEvent::Adapter(event) => {
                    self.handle_adapter_event(event).await;
                }
                AppEvent::Agent(event) => {
                    self.handle_agent_event(&event).await;
                }
                AppEvent::Quit => {
                    self.shutdown().await;
                    break;
                }
            }
        }
    }

    fn render(&mut self) {
        self.renderer
            .render(&self.state, &self.settings.display_settings)
    }

    async fn shutdown(&mut self) {
        self.renderer.clear();
        self.agent.unregister().await;
    }

    async fn handle_user_action(&mut self, action: &UserAction) {
        match action {
            UserAction::TriggerScan => {
                self.state.scanning = self.bt_controller.trigger_scan().await.unwrap();
                info!("done switching scan state to {}", self.state.scanning);
            }
            // Connection is slow, so it is started in a separate task
            // to avoid blocking the UI
            UserAction::Connect => {
                if let Some(device) = self.state.selected_device() {
                    let controller = self.bt_controller.clone();
                    let tx = self.events_channel.tx();
                    self.state.connecting.insert(device.address);
                    info!("Connecting to {}", device);
                    tokio::spawn(async move {
                        match controller.connect(&device).await {
                            Ok(_) => info!("Connected to {}", device),
                            Err(_) => {
                                error!("Failed to connect to {}", device);
                                tx.send(AppEvent::Adapter(AdapterEvent::FailedToConnect(device)))
                                    .await
                                    .unwrap();
                            }
                        }
                    });
                }
            }
            // Disonnection is slow, so it is started in a separate task
            // to avoid blocking the UI
            UserAction::Disconnect => {
                if let Some(device) = self.state.selected_device() {
                    let controller = self.bt_controller.clone();
                    let tx = self.events_channel.tx();
                    self.state.disconnecting.insert(device.address);
                    info!("Disconnecting from {}", device);
                    tokio::spawn(async move {
                        match controller.disconnect(&device).await {
                            Ok(_) => info!("Disconnected from {}", device),
                            Err(e) => {
                                error!("Failed to disconnect from {}: {}", &device, e);
                                tx.send(AppEvent::Adapter(AdapterEvent::FailedToDisconnect(
                                    device,
                                )))
                                .await
                                .unwrap();
                            }
                        }
                    });
                }
            }
            UserAction::MoveUp => {
                if let Some(device) = &self.state.selected_device {
                    self.state.selected_device = self.renderer.get_previous_device(device);
                } else {
                    self.state.selected_device = self.renderer.get_last_device();
                }
            }
            UserAction::MoveDown => {
                if let Some(device) = &self.state.selected_device {
                    self.state.selected_device = self.renderer.get_next_device(device);
                } else {
                    self.state.selected_device = self.renderer.get_first_device();
                }
            }
            UserAction::Quit => {
                self.events_channel.tx.send(AppEvent::Quit).await.unwrap();
            }
        }
    }

    async fn handle_agent_event(&mut self, event: &AgentEvent) {
        match event {
            AgentEvent::Release { tx } => {
                error!("Agent was released, exiting...");
                self.events_channel.tx.send(AppEvent::Quit).await.unwrap();
            }
            AgentEvent::RequestPincode { tx, address } => {
                // self.state.popup = Some(Box::new(InputPopup::new(
                //     "Pincode".into(),
                //     "message".into(),
                //     false,
                // )));
                todo!()
            }
            AgentEvent::DisplayPincode {
                pincode,
                tx,
                address,
            } => {
                // self.state.popup = Some(Box::new(ConfirmationPopup::new(
                //     "Pincode".into(),
                //     format!("Pincode: {}", pincode),
                // )));
                todo!()
            }
            AgentEvent::RequestPasskey { tx, address } => {
                todo!()
            }
            AgentEvent::DisplayPasskey {
                passkey,
                tx,
                address,
            } => {
                todo!()
            }
            AgentEvent::RequestConfirmation {
                passkey,
                tx,
                address,
            } => {
                todo!()
            }
            AgentEvent::RequestAuthorization { tx, address } => {
                todo!()
            }
            AgentEvent::AuthorizeService { uuid, tx, address } => {
                todo!()
            }
            AgentEvent::Cancel { tx } => {
                warn!("Request cancelled");
                todo!()
                // self.state.popup = None;
            }
        }
    }

    async fn handle_adapter_event(&mut self, event: AdapterEvent) {
        match event {
            AdapterEvent::Discovered(device) => {
                self.state.devices.update(device);
            }
            AdapterEvent::Updated(device) => {
                self.state.devices.update(device);
            }
            AdapterEvent::Connecting(device) => {
                self.state.connecting.insert(device.address);
            }
            AdapterEvent::Connected(device) => {
                self.state.connecting.remove(&device.address);
                self.state.devices.update(device);
            }
            AdapterEvent::FailedToConnect(device) => {
                self.state.connecting.remove(&device.address);
            }
            AdapterEvent::Disconnecting(device) => {
                self.state.disconnecting.insert(device.address);
            }
            AdapterEvent::Disconnected(device) => {
                self.state.disconnecting.remove(&device.address);
                self.state.devices.update(device);
            }
            AdapterEvent::FailedToDisconnect(device) => {
                self.state.disconnecting.remove(&device.address);
            }
        }
    }
}
