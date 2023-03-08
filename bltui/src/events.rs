// use std::{
//     pin::Pin,
//     task::{Context, Poll},
// };

use crossterm::event::{poll, Event, KeyCode};
// use futures::Stream;
use futures::StreamExt;

use crate::bluetooth::{agent::AgentEvent, controller::AdapterEvent};

#[derive(Debug, PartialEq)]
pub enum UserAction {
    TriggerScan,
    Connect,
    Disconnect,
    MoveUp,
    MoveDown,
    Quit,
}

#[derive(Debug)]
pub enum AppEvent {
    Input(UserAction),
    Tick,
    Adapter(AdapterEvent),
    Agent(AgentEvent),
    Quit,
}

// struct EventStream {
//     controller: BluetoothController,
// }

// impl EventStream {
//     pub fn new(controller: BluetoothController) -> Self {
//         Self { controller }
//     }
// }

// impl Stream for EventStream {
//     type Item = AppEvent;

//     fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
//         todo!()
//     }
// }

/// Need to send keycode and do translation depending on the context
pub fn spawn_keypress_watcher(tx: tokio::sync::mpsc::Sender<AppEvent>) {
    std::thread::spawn(move || loop {
        if poll(std::time::Duration::from_millis(2)).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                if let Some(action) = match key.code {
                    KeyCode::Char('q') => Some(UserAction::Quit),
                    KeyCode::Char('s') => Some(UserAction::TriggerScan),
                    KeyCode::Char('c') => Some(UserAction::Connect),
                    KeyCode::Char('d') => Some(UserAction::Disconnect),
                    KeyCode::Up | KeyCode::Char('k') => Some(UserAction::MoveUp),
                    KeyCode::Down | KeyCode::Char('j') => Some(UserAction::MoveDown),
                    _ => None,
                } {
                    tx.blocking_send(AppEvent::Input(action)).unwrap();
                }
            }
        }
    });
}

pub async fn spawn_adapter_watcher(
    mut events: std::pin::Pin<Box<dyn futures::Stream<Item = AdapterEvent> + std::marker::Send>>,
    tx: tokio::sync::mpsc::Sender<AppEvent>,
) {
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            tx.send(AppEvent::Adapter(event)).await.unwrap();
        }
    });
}

pub fn spawn_ticker(tick_rate: std::time::Duration, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    let mut ticker = tokio::time::interval(tick_rate);

    tokio::spawn(async move {
        loop {
            ticker.tick().await;
            tx.send(AppEvent::Tick).await.unwrap();
        }
    });
}
