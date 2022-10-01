use crossterm::event::{poll, Event};

use super::AppEvent;

pub fn spawn_keypress_watcher(tx: tokio::sync::mpsc::Sender<AppEvent>) {
    std::thread::spawn(move || loop {
        if poll(std::time::Duration::from_millis(2)).unwrap() {
            if let Event::Key(key) = crossterm::event::read().unwrap() {
                tx.blocking_send(AppEvent::Input(key)).unwrap();
            }
        }
    });
}
