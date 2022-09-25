use btleplug::api::CentralEvent;
use crossterm::{
    event::{Event, KeyCode},
    terminal::disable_raw_mode,
};
use futures::stream::StreamExt;
use log::{debug, error, info, trace};
use tui::widgets::{List, ListItem};

use crate::{
    app,
    bluetooth::{
        agent::{Agent, AgentCapability},
        controller::BluetoothController,
        devices::Devices,
    },
    ui::{draw_frame, initialize_terminal, popup::QuestionPopup, widgets::statics::blue_box},
};
use dirs::home_dir;

pub struct App<'a> {
    state: std::sync::Arc<tokio::sync::Mutex<AppState<'a>>>,
    bt_controller: BluetoothController,
    settings: AppSettings,
}

struct AppSettings {
    log_level: log::LevelFilter,
    show_unknown: bool,
    log_to_file: bool,
}

impl AppSettings {
    pub fn new(log_level: log::LevelFilter, show_unknown: bool, log_to_file: bool) -> AppSettings {
        AppSettings {
            log_level,
            show_unknown,
            log_to_file,
        }
    }
}

#[derive(Clone)]
pub struct AppState<'a> {
    pub devices: Devices,
    pub popup: Option<QuestionPopup<'a>>,
}

impl<'a> AppState<'a> {
    pub fn new() -> AppState<'a> {
        AppState {
            devices: Devices::new(),
            popup: None,
            // popup: Option<QuestionPopupState>,
        }
    }

    pub fn devices(&self) -> Devices {
        self.devices.clone()
    }
}

impl<'a> Default for AppState<'a> {
    fn default() -> AppState<'a> {
        Self::new()
    }
}

impl App<'static> {
    pub async fn new(
        logging_level: log::LevelFilter,
        show_unknown: bool,
        log_to_file: bool,
    ) -> App<'static> {
        info!("Initializing the app");

        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new()));
        let bt_controller = BluetoothController::from_first_adapter().await;

        App {
            state: app_state,
            bt_controller,
            settings: AppSettings::new(logging_level, show_unknown, log_to_file),
        }
    }

    async fn spawn_bt_event_handler(&self) {
        let bt_controller = self.bt_controller.clone();
        let state = std::sync::Arc::clone(&self.state);
        let mut bt_events = bt_controller.events().await;
        let show_unknown = self.settings.show_unknown;

        tokio::spawn(async move {
            while let Some(event) = bt_events.next().await {
                trace!("Receveived a new event : {:?}", event);

                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let device = bt_controller.get_device(&id).await;
                        debug!("New device : {} ({})", device.name, device.address);

                        if device.name != "Unknown" || show_unknown {
                            let mut state = state.lock().await;
                            state.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceUpdated(id) => {
                        let device = bt_controller.get_device(&id).await;
                        if device.name != "Unknown" || show_unknown {
                            let mut state = state.lock().await;
                            state.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        let device = bt_controller.get_device(&id).await;
                        info!("Connected to {} ({})", device.name, device.address);
                        if device.name != "Unknown" || show_unknown {
                            let mut state = state.lock().await;
                            state.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        let device = bt_controller.get_device(&id).await;
                        info!("Disconnected from {} ({})", device.name, device.address);
                        if device.name != "Unknown" || show_unknown {
                            let mut state = state.lock().await;
                            state.devices.insert_or_replace(device);
                        }
                    }
                    // CentralEvent::CustomEvent(message) => {
                    //     warn!("Received a custom event : {}", message);
                    // }
                    _ => {}
                }
            }
        });
    }

    pub async fn start(&mut self) {
        tui_logger::init_logger(self.settings.log_level).unwrap();
        tui_logger::set_default_level(self.settings.log_level);

        if self.settings.log_to_file {
            let timestamp = chrono::Utc::now();
            let mut logs_dir = home_dir().unwrap();
            logs_dir.push(".bluetui");
            logs_dir.push("logs");
            std::fs::create_dir_all(&logs_dir).expect("Could not create log directory");
            tui_logger::set_log_file(&format!("{}/{}.log", logs_dir.to_str().unwrap(), timestamp))
                .unwrap();
        }

        let agent = Agent::new("/bluetui/agent", AgentCapability::DisplayOnly);
        agent.start().await;
        agent.register_and_request_default_agent().await;

        self.spawn_bt_event_handler().await;

        let mut terminal = initialize_terminal();
        let tick_rate = std::time::Duration::from_millis(7);

        let app_state_ui = std::sync::Arc::clone(&self.state);

        loop {
            if crossterm::event::poll(tick_rate).unwrap() {
                if let Event::Key(key) = crossterm::event::read().unwrap() {
                    let some_popup = {
                        let state = app_state_ui.lock().await;
                        state.popup.is_some()
                    };

                    if some_popup {
                        match key.code {
                            KeyCode::Down => {
                                let mut state = app_state_ui.lock().await;
                                let popup = state.popup.as_mut().unwrap();
                                popup.move_selector_down();
                            }
                            KeyCode::Up => {
                                let mut state = app_state_ui.lock().await;
                                let popup = state.popup.as_mut().unwrap();
                                popup.move_selector_up();
                            }
                            KeyCode::Enter => {
                                let mut state = app_state_ui.lock().await;
                                state.popup = None;
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => {
                                // Quit
                                break;
                            }
                            KeyCode::Char('1') => {
                                info!("11111111111111");
                            }
                            KeyCode::Char('d') => {
                                // Disconnect from the device
                                let device_opt = {
                                    let state = app_state_ui.lock().await;
                                    state.devices.get_selected_device().await
                                };
                                if let Some(device) = device_opt {
                                    info!(
                                        "Trying to disconnect from {} ({})",
                                        device.name, device.address
                                    );
                                    if self
                                        .bt_controller
                                        .disconnect(&device.periph_id)
                                        .await
                                        .is_err()
                                    {
                                        error!("Failed to disconnect");
                                    }
                                }
                            }
                            KeyCode::Char('c') => {
                                // Connect to the device
                                let device_opt = {
                                    let state = app_state_ui.lock().await;
                                    state.devices.get_selected_device().await
                                };
                                if let Some(device) = device_opt {
                                    info!(
                                        "Trying to connect to {} ({})",
                                        device.name, device.address
                                    );
                                    let bt_controller_temp = self.bt_controller.clone();
                                    tokio::spawn(async move {
                                        if bt_controller_temp
                                            .connect(&device.periph_id)
                                            .await
                                            .is_err()
                                        {
                                            error!("Failed to connect");
                                        }
                                    });
                                }
                            }
                            KeyCode::Char('s') => {
                                // Trigger scan
                                if self.bt_controller.trigger_scan().await.is_err() {
                                    error!("Failed to switch scan");
                                }
                            }
                            KeyCode::Down => {
                                let mut state = app_state_ui.lock().await;
                                state.devices.move_selector_down();
                            }
                            KeyCode::Up => {
                                let mut state = app_state_ui.lock().await;
                                state.devices.move_selector_up();
                            }
                            KeyCode::Right => {
                                let mut state = app_state_ui.lock().await;
                                state.popup = Some(
                                    QuestionPopup::new(
                                        "Confirm pairing ?".to_string(),
                                        vec![ListItem::new("Yes"), ListItem::new("No")],
                                    )
                                    .block(blue_box(None)),
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }

            draw_frame(&mut terminal, &app_state_ui, self.bt_controller.scanning).await;
        }
        debug!("Quitting");
        disable_raw_mode().unwrap();
        terminal.clear().unwrap();
    }
}
