use btleplug::api::CentralEvent;
use crossterm::event::KeyEvent;

pub mod adapter;
pub mod agent;
pub mod keys;
pub mod tick;

use agent::AgentEvent;

#[derive(Debug)]
pub enum AppEvent {
    Input(KeyEvent),
    Tick,
    Adapter(CentralEvent),
    Agent(AgentEvent),
}
