use std::io::Stdout;

use crossterm::{execute, terminal::EnterAlternateScreen};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    Terminal,
};

pub mod widgets;

use crate::App;

use self::widgets::{
    device_details::get_device_details,
    devices::devices_list,
    logger::get_logger_widget,
    statics::{main_commands, popup_commands, title},
};

pub fn initialize_terminal() -> Terminal<CrosstermBackend<&Stdout>>{
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen)?;
    crossterm::terminal::enable_raw_mode()?;
    let backend = tui::backend::CrosstermBackend::new(&stdout);
    let mut terminal = tui::Terminal::new(backend)?;
    terminal.clear()?;
    terminal
}

pub async fn draw_frame<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, scanning: bool) {
    let selected_device = app.devices.get_selected_device().await;

    terminal
        .draw(|rect| {
            let size = rect.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
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
                devices_list(&app.devices),
                main_chunks[0],
                &mut app.devices.list_state,
            );
            rect.render_widget(get_logger_widget(), right_chunks[1]);
            rect.render_widget(get_device_details(selected_device), right_chunks[0]);

            if let Some(popup) = &app.popup {
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

                rect.render_widget(popup.get_widget(), popup_chunk);
            } else {
                rect.render_widget(main_commands(scanning), chunks[2]);
            }
        })
        .unwrap();
}
