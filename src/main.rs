use std::time::Duration;

use bltui::{
    bluetooth::{
        agent::{Agent, AgentCapability},
        controller::BluetoothController,
    },
    events::{
        adapter::spawn_adapter_watcher, agent::AgentEvent, keys::spawn_keypress_watcher,
        tick::spawn_ticker, AppEvent,
    },
    logging::initialize_logging,
    settings::AppSettings,
    ui::{draw_frame, initialize_terminal, widgets::popup::YesNoPopup},
    App,
};
use btleplug::api::CentralEvent;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use log::{debug, error, info, trace, warn};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref TICK_RATE: Duration = Duration::from_millis(16);
    static ref KEY_POLL_RATE: Duration = Duration::from_millis(8);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = AppSettings::parse();

    initialize_logging(settings.log_settings)?;

    let mut app = App::new();

    let mut bt_controller = if let Some(adapter) = settings.adapter {
        BluetoothController::from_adapter(&adapter)
            .await
            .expect("The requested adapter was not found")
    } else {
        BluetoothController::from_first_adapter().await
    };

    let agent =
        Agent::initialize_dbus_connection("/bltui/agent".into(), AgentCapability::KeyboardDisplay)
            .await;
    agent.start_server(app.tx()).await;
    agent.request_name("bltui.agent").await;
    agent.register().await;
    agent.request_default().await;

    spawn_ticker(*TICK_RATE, app.tx());

    spawn_keypress_watcher(app.tx());

    spawn_adapter_watcher(bt_controller.events().await?, app.tx()).await;

    let mut terminal = initialize_terminal()?;

    while let Some(event) = app.events().await {
        match event {
            AppEvent::Agent(ev) => {
                debug!("Received Agent event : {:?}", ev);
                match ev {
                    AgentEvent::RequestConfirmation { passkey, tx } => {
                        app.popup = Some(YesNoPopup::new(
                            format!("Confirm pairing with passkey {}", passkey),
                            tx,
                        ));
                    }
                    AgentEvent::AuthorizeService { uuid, tx } => {
                        app.popup = Some(YesNoPopup::new(
                            format!("Confirm service authorization ({})", uuid),
                            tx,
                        ));
                    }
                    AgentEvent::DisplayPasskey { passkey, tx } => {
                        app.popup = Some(YesNoPopup::new(format!("Passkey : {}", passkey), tx));
                    }
                    AgentEvent::DisplayPincode { pincode, tx } => {
                        app.popup = Some(YesNoPopup::new(format!("Pincode : {}", pincode), tx));
                    }
                    AgentEvent::Release { tx } => {
                        error!("Agent release was requested, shutting down");
                        std::thread::sleep(Duration::from_secs(5));
                        tx.send(Ok(())).unwrap();
                        break;
                    }
                    AgentEvent::Cancel { tx } => {
                        warn!("Pairing cancelled");
                        app.popup = None;
                        tx.send(Ok(())).unwrap();
                    }
                    AgentEvent::RequestAuthorization { tx } => {
                        app.popup = Some(YesNoPopup::new(
                            "Accept pairing authorization ?".to_string(),
                            tx,
                        ));
                    }
                    AgentEvent::RequestPasskey { tx } => {
                        tx.send(Ok(0_u32)).unwrap();
                    }
                    AgentEvent::RequestPincode { tx } => {
                        tx.send(Ok(String::from("wontwork"))).unwrap();
                    }
                }
            }
            AppEvent::Adapter(ev) => {
                trace!("Received adapter event : {:?}", ev);
                match ev {
                    CentralEvent::DeviceDiscovered(periph_id) => {
                        debug!("Device discovered");
                        let device = bt_controller.get_device(&periph_id).await;
                        if device.name != "Unknown" || settings.show_unknown {
                            app.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceConnected(periph_id) => {
                        info!("Connected to");
                        let device = bt_controller.get_device(&periph_id).await;
                        if device.name != "Unknown" || settings.show_unknown {
                            app.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceDisconnected(periph_id) => {
                        info!("Disconnected from ");
                        let device = bt_controller.get_device(&periph_id).await;
                        if device.name != "Unknown" || settings.show_unknown {
                            app.devices.insert_or_replace(device);
                        }
                    }
                    CentralEvent::DeviceUpdated(periph_id) => {
                        let device = bt_controller.get_device(&periph_id).await;
                        if device.name != "Unknown" || settings.show_unknown {
                            app.devices.insert_or_replace(device);
                        }
                    }
                    _ => {}
                }
            }
            AppEvent::Input(key) => {
                debug!("Received input key : {:?}", key);
                if let Some(popup) = &mut app.popup {
                    match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            popup.move_selector_down();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            popup.move_selector_up();
                        }
                        KeyCode::Enter => {
                            popup.confirm();
                            app.popup = None;
                        }
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            app.devices.move_selector_down();
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            app.devices.move_selector_up();
                        }
                        KeyCode::Char('c') => {
                            let controller = bt_controller.clone();
                            let device_opt = &app.devices.get_selected_device().await;
                            if let Some(device) = device_opt {
                                let periph_id = device.periph_id.clone();
                                tokio::spawn(async move {
                                    controller.connect(&periph_id).await.unwrap();
                                });
                            }
                        }
                        KeyCode::Char('d') => {
                            let controller = bt_controller.clone();
                            let device_opt = &app.devices.get_selected_device().await;
                            if let Some(device) = device_opt {
                                let periph_id = device.periph_id.clone();
                                tokio::spawn(async move {
                                    controller.disconnect(&periph_id).await.unwrap();
                                });
                            }
                        }
                        KeyCode::Char('s') => {
                            bt_controller.trigger_scan().await?;
                        }
                        KeyCode::Char('q') => {
                            break;
                        }
                        _ => {}
                    }
                }
                // TODO : If popup, use popup key handler, otherwise use the main one
            }
            AppEvent::Tick => {
                trace!("Frame tick");
                draw_frame(&mut terminal, &mut app, bt_controller.scanning).await;
                // if popup.is_some() {
                // draw_popup();
                // }
            }
        }
    }

    agent.unregister().await;

    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    Ok(())
}
