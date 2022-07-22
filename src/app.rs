use btleplug::api::CentralEvent;
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::disable_raw_mode;
use futures::stream::StreamExt;
use log::{debug, error, info, trace, warn};

use crate::bluetooth::BluetoothController;
use crate::devices::{Device, Devices};
use crate::ui::{draw_ui, initialize_terminal};

pub struct App {
    state: std::sync::Arc<tokio::sync::Mutex<AppState>>,
    bt_controller: BluetoothController,
}

pub struct AppState {
    devices: Devices,
    pub hide_unnamed: bool,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            devices: Devices::new(),
            hide_unnamed: false,
        }
    }

    pub fn devices(&self) -> Devices {
        self.devices.clone()
    }
}

impl App {
    pub async fn new() -> App {
        info!("Initializing the app");

        let app_state = std::sync::Arc::new(tokio::sync::Mutex::new(AppState::new()));
        let bt_controller = BluetoothController::from_first_adapter().await;

        App {
            state: app_state,
            bt_controller,
        }
    }

    pub async fn start(&mut self) {
        let mut bt_events = self.bt_controller.events().await;
        let app_state_bt = std::sync::Arc::clone(&self.state);
        let ev_bt_controller = self.bt_controller.clone();
        tokio::spawn(async move {
            while let Some(event) = bt_events.next().await {
                trace!("Received a new event : {:?}", event);
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let device =
                            Device::from_periph(&ev_bt_controller.get_peripheral(&id).await).await;
                        debug!("New device : {} ({})", device.name, device.address);

                        let mut state = app_state_bt.lock().await;
                        state.devices.insert_or_replace(device);
                    }
                    CentralEvent::DeviceUpdated(id) => {
                        let device =
                            Device::from_periph(&ev_bt_controller.get_peripheral(&id).await).await;
                        let mut state = app_state_bt.lock().await;
                        state.devices.insert_or_replace(device);
                    }
                    CentralEvent::DeviceConnected(id) => {
                        let device =
                            Device::from_periph(&ev_bt_controller.get_peripheral(&id).await).await;
                        info!("Connected to {} ({})", device.name, device.address);
                        let mut state = app_state_bt.lock().await;
                        state.devices.insert_or_replace(device);
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        let device =
                            Device::from_periph(&ev_bt_controller.get_peripheral(&id).await).await;
                        info!("Disconnected from {} ({})", device.name, device.address);
                        let mut state = app_state_bt.lock().await;
                        state.devices.insert_or_replace(device);
                    }
                    _ => {}
                }
            }
        });

        let mut terminal = initialize_terminal();
        let tick_rate = std::time::Duration::from_millis(7);

        let app_state_ui = std::sync::Arc::clone(&self.state);
        let mut frame_times = Vec::new();

        loop {
            let starting_time = std::time::Instant::now();

            if crossterm::event::poll(tick_rate).unwrap() {
                if let Event::Key(key) = crossterm::event::read().unwrap() {
                    match key.code {
                        KeyCode::Char('q') => {
                            // Quit
                            break;
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
                                self.bt_controller
                                    .disconnect(&device.periph_id)
                                    .await
                                    .unwrap();
                            }
                        }
                        KeyCode::Char('c') => {
                            // Connect to the device
                            let device_opt = {
                                let state = app_state_ui.lock().await;
                                state.devices.get_selected_device().await
                            };
                            if let Some(device) = device_opt {
                                info!("Trying to connect to {} ({})", device.name, device.address);
                                self.bt_controller.connect(&device.periph_id).await.unwrap();
                            }
                        }
                        KeyCode::Char('h') => {
                            // Hide unnamed devices
                            let mut state = app_state_ui.lock().await;
                            state.hide_unnamed = !state.hide_unnamed;
                        }
                        KeyCode::Char('s') => {
                            // Trigger scan
                            self.bt_controller.trigger_scan().await.unwrap();
                        }
                        KeyCode::Down => {
                            let mut state = app_state_ui.lock().await;
                            state.devices.move_selector_down();
                        }
                        KeyCode::Up => {
                            let mut state = app_state_ui.lock().await;
                            state.devices.move_selector_up();
                        }
                        KeyCode::Enter => {}
                        _ => {}
                    }
                }
            }

            draw_ui(&mut terminal, &app_state_ui, self.bt_controller.scanning).await;
            frame_times.push(starting_time.elapsed());

            if frame_times.len() % 100 == 1 {
                debug!(
                    "Avg frame time : {:?}",
                    frame_times
                        .clone()
                        .into_iter()
                        .reduce(|a, b| a + b)
                        .unwrap()
                        / frame_times.len() as u32
                );
            }
        }
        info!("Quitting");
        disable_raw_mode().unwrap();
        terminal.clear().unwrap();
    }
}
