use crate::app::AppState;
use crossterm::terminal::enable_raw_mode;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Terminal,
};

mod widgets;
pub mod popup;

use self::widgets::{
    device_details::get_device_details,
    devices::devices_list,
    logger::get_logger_widget,
    statics::{commands, title},
};

pub fn initialize_terminal() -> Terminal<CrosstermBackend<std::io::Stdout>> {
    let stdout = std::io::stdout();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal
}

pub async fn draw_frame<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &std::sync::Arc<tokio::sync::Mutex<AppState>>,
    scanning: bool,
) {
    let state = state.lock().await;
    let selected_device = state.devices().get_selected_device().await;

    terminal
        .draw(|rect| {
            let size = rect.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                // .constraints([Constraint::Percentage(7), Constraint::Percentage(90)])
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(5),
                    Constraint::Length(3),
                ])
                .split(size);

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
                .split(chunks[1]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(main_chunks[1]);

            rect.render_widget(title(), chunks[0]);
            rect.render_widget(commands(scanning), chunks[2]);
            rect.render_stateful_widget(
                devices_list(&state.devices),
                main_chunks[0],
                &mut state.devices().list_state,
            );
            rect.render_widget(get_logger_widget(), right_chunks[1]);
            rect.render_widget(get_device_details(selected_device), right_chunks[0]);
        })
        .unwrap();
}
