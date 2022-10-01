use std::time::Duration;

use bluetui::{
    bluetooth::{
        agent::{Agent, AgentCapability},
        controller::BluetoothController,
    },
    events::{
        adapter::spawn_adapter_watcher, agent::AgentEvent, keys::spawn_keypress_watcher,
        tick::spawn_ticker, AppEvent,
    },
    logging::{init_file_logging, init_tui_logger},
    ui::{draw_frame, initialize_terminal, widgets::popup::YesNoPopup},
    App,
};
use btleplug::api::CentralEvent;
use clap::Parser;
use crossterm::{
    event::KeyCode,
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
};
use log::{debug, trace};

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref TICK_RATE: Duration = Duration::from_millis(16);
    static ref KEY_POLL_RATE: Duration = Duration::from_millis(8);
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct AppSettings {
    /// Activate debug/trace messages
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Display devices with an unknown name
    #[arg(short = 'u', long, action)]
    show_unknown: bool,

    /// Log to file (/$homedir/.bluetui/logs)
    #[arg(short, long, action)]
    log_to_file: bool,
}

fn translate_log_level(count: u8) -> log::LevelFilter {
    match count {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let mut stdout = stdout();
    let settings = AppSettings::parse();

    let log_level = translate_log_level(settings.debug);

    init_tui_logger(log_level);
    if settings.log_to_file {
        init_file_logging()?;
    }

    let mut app = App::new();

    let mut bt_controller = BluetoothController::from_first_adapter().await;

    let agent = Agent::initialize_dbus_connection(
        "/bluetui/agent".into(),
        AgentCapability::KeyboardDisplay,
    )
    .await;
    agent.start_server(app.tx()).await;
    agent.request_name("bluetui.agent").await;
    agent.register().await;
    agent.request_default().await;

    spawn_ticker(*TICK_RATE, app.tx());

    spawn_keypress_watcher(app.tx());

    spawn_adapter_watcher(bt_controller.events().await, app.tx()).await;

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
                    _ => {}
                }
            }
            AppEvent::Adapter(ev) => {
                trace!("Received adapter event : {:?}", ev);
                match ev {
                    CentralEvent::DeviceDiscovered(periph_id)
                    | CentralEvent::DeviceUpdated(periph_id)
                    | CentralEvent::DeviceConnected(periph_id)
                    | CentralEvent::DeviceDisconnected(periph_id) => {
                        let device = bt_controller.get_device(&periph_id).await;
                        app.devices.insert_or_replace(device);
                    }
                    _ => {}
                }
            }
            AppEvent::Input(key) => {
                debug!("Received input key : {:?}", key);
                if let Some(popup) = &mut app.popup {
                    match key.code {
                        KeyCode::Down => {
                            popup.move_selector_down();
                        }
                        KeyCode::Up => {
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
                        KeyCode::Down => {
                            app.devices.move_selector_down();
                        }
                        KeyCode::Up => {
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