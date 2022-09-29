use std::{path::PathBuf, pin::Pin, thread, time::Duration};

use bluetui::{
    app::AppNew,
    bluetooth::{
        agent::{Agent, AgentCapability},
        controller::BluetoothController,
    },
    events::{AgentEvent, AppEvent},
    ui::{draw_frame, initialize_terminal, widgets::popup::YesNoPopup},
};
use btleplug::api::CentralEvent;
use clap::Parser;
use crossterm::{
    event::{Event, KeyCode},
    terminal::disable_raw_mode,
};
use dirs::home_dir;
use futures::{Stream, StreamExt};
use log::{debug, trace};
use tokio::time;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref TICK_RATE: Duration = Duration::from_millis(7);
    static ref KEY_POLL_RATE: Duration = Duration::from_millis(2);
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

fn get_logs_dir() -> PathBuf {
    let mut logs_dir = home_dir().unwrap();
    logs_dir.push(".bluetui");
    logs_dir.push("logs");
    logs_dir
}

fn get_log_file_path(logs_dir: PathBuf) -> PathBuf {
    let mut log_file = logs_dir.clone();
    log_file.push(chrono::Utc::now().to_rfc3339());
    log_file.set_extension("log");
    log_file
}

fn spawn_ticker(tick_rate: Duration, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    let mut ticker = time::interval(tick_rate);

    tokio::spawn(async move {
        loop {
            ticker.tick().await;
            tx.send(AppEvent::Tick).await.unwrap();
        }
    });
}

fn spawn_keypress_watcher(tx: tokio::sync::mpsc::Sender<AppEvent>) {
    thread::spawn(move || loop {
        if crossterm::event::poll(*KEY_POLL_RATE).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                tx.blocking_send(AppEvent::Input(key)).unwrap();
            }
        }
    });
}

async fn spawn_adapter_watcher(
    mut events: Pin<Box<dyn Stream<Item = CentralEvent> + std::marker::Send>>,
    tx: tokio::sync::mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            tx.send(AppEvent::Adapter(event)).await.unwrap();
        }
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = AppSettings::parse();

    let log_level = translate_log_level(settings.debug);

    tui_logger::init_logger(log_level).unwrap();
    tui_logger::set_default_level(log_level);

    if settings.log_to_file {
        let logs_dir = get_logs_dir();
        std::fs::create_dir_all(&logs_dir).expect("Could not create log directory");
        let log_file = get_log_file_path(logs_dir);
        println!("log  file : {:?}", log_file);
        tui_logger::set_log_file(&log_file.to_str().unwrap()).unwrap();
    }

    let mut terminal = initialize_terminal();

    let (app_tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(100);

    let mut app = AppNew::new();

    let mut bt_controller = BluetoothController::from_first_adapter().await;

    let agent =
        Agent::initialize_dbus_connection("/bluetui/agent", AgentCapability::KeyboardDisplay).await;
    agent.start_server(app_tx.clone()).await;
    agent.request_name("bluetui.agent").await;
    agent.register_agent().await;
    agent.request_default_agent().await;

    spawn_ticker(*TICK_RATE, app_tx.clone());

    spawn_keypress_watcher(app_tx.clone());

    spawn_adapter_watcher(bt_controller.events().await, app_tx.clone()).await;
    // TODO : Spawn task that transmits central events

    while let Some(event) = rx.recv().await {
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
                            // app.popup.as_ref().unwrap().move_selector_down();
                        }
                        KeyCode::Up => {
                            popup.move_selector_up();
                            // app.popup.unwrap().move_selector_up();
                        }
                        KeyCode::Enter => {
                            // let tx = popup.get_tx();
                            // app.popup.unwrap().confirm();
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
                            let device = &app.devices.get_selected_device().await.unwrap();
                            let periph_id = device.periph_id.clone();
                            tokio::spawn(async move {
                                controller.connect(&periph_id).await.unwrap();
                            });
                        }
                        KeyCode::Char('d') => {
                            let controller = bt_controller.clone();
                            let device = &app.devices.get_selected_device().await.unwrap();
                            let periph_id = device.periph_id.clone();
                            tokio::spawn(async move {
                                controller.disconnect(&periph_id).await.unwrap();
                            });
                        }
                        KeyCode::Char('s') => {
                            bt_controller.trigger_scan().await.unwrap();
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
                // println!("tick");
                draw_frame(&mut terminal, &mut app, bt_controller.scanning).await;
                // if popup.is_some() {
                // draw_popup();
                // }
            }
        }
    }

    disable_raw_mode().unwrap();
    terminal.clear().unwrap();

    Ok(())
}
