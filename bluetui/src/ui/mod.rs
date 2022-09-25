use crate::app::AppState;
use crossterm::terminal::enable_raw_mode;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::ListItem,
    Terminal,
};

pub mod popup;
pub mod widgets;

use self::{
    popup::{QuestionPopup, QuestionPopupItem, QuestionPopupState},
    widgets::{
        device_details::get_device_details,
        devices::devices_list,
        logger::get_logger_widget,
        statics::{blue_box, main_commands, popup_commands, title},
    },
};

pub fn initialize_terminal() -> Terminal<CrosstermBackend<std::io::Stdout>> {
    let stdout = std::io::stdout();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal
}

pub async fn draw_frame<'a, B: Backend>(
    terminal: &mut Terminal<B>,
    state: &std::sync::Arc<tokio::sync::Mutex<AppState<'_>>>,
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
            rect.render_stateful_widget(
                devices_list(&state.devices),
                main_chunks[0],
                &mut state.devices().list_state,
            );
            rect.render_widget(get_logger_widget(), right_chunks[1]);
            rect.render_widget(get_device_details(selected_device), right_chunks[0]);

            

            if let Some(popup) = &state.popup {
                rect.render_widget(popup_commands(), chunks[2]);
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),
                        Constraint::Length(6),
                        Constraint::Percentage(45),
                    ])
                    .split(size);

                let popup_chunk = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                        Constraint::Percentage(20),
                    ])
                    .split(vertical_chunks[1])[1];

                rect.render_widget(popup.clone(), popup_chunk);
            } else {
                rect.render_widget(main_commands(scanning), chunks[2]);
            }
        })
        .unwrap();
}
