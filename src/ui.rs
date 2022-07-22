use crate::app::AppState;
use crossterm::terminal::enable_raw_mode;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, Paragraph},
    Terminal,
};

pub fn initialize_terminal() -> Terminal<CrosstermBackend<std::io::Stdout>> {
    let stdout = std::io::stdout();
    enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.clear().unwrap();
    terminal
}

fn blue_box(title: Option<String>) -> Block<'static> {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue));

    if let Some(title_str) = title {
        block.title(Span::styled(title_str, Style::default().fg(Color::White)))
    } else {
        block
    }
}

pub async fn draw_ui<B: Backend>(
    terminal: &mut Terminal<B>,
    state: &std::sync::Arc<tokio::sync::Mutex<AppState>>,
    scanning: bool,
) {
    let state = state.lock().await;
    let selected_device = { state.devices().get_selected_device().await };

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

            let title = Paragraph::new("Chesapeake")
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Center)
                .block(blue_box(None));

            let device_details_str = if let Some(device) = selected_device {
                Text::from(vec![
                    Spans::from(Span::raw(device.name)),
                    Spans::from(Span::raw(format!("Address : {}", device.address))),
                    Spans::from(Span::raw(format!(
                        "Signal strenth : {} dBm",
                        device.rssi.unwrap_or(0)
                    ))),
                    Spans::from(Span::raw(format!(
                        "Tx power : {} dBm",
                        device.tx_power.unwrap_or(0)
                    ))),
                    Spans::from(vec![
                        Span::raw("Connected : "),
                        if device.connected {
                            Span::styled("yes", Style::default().fg(Color::Green))
                        } else {
                            Span::styled("no", Style::default().fg(Color::Red))
                        },
                    ]),
                ])
            } else {
                Text::from(vec![Spans::from(vec![Span::raw("")])])
            };

            let device_details = Paragraph::new(device_details_str)
                .style(Style::default())
                .alignment(Alignment::Left)
                .block(blue_box(Some(String::from("Details"))));

            let devices = state.devices();
            let devices_items = devices.list_items();

            let list = List::new(devices_items)
                .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
                .highlight_symbol("->")
                .block(blue_box(Some(format!(
                    "Devices ({}/{})",
                    {
                        if let Some(index) = state.devices().list_state.selected() {
                            index + 1
                        } else {
                            0
                        }
                    },
                    state.devices().devices.len()
                ))));

            let commands_str = Spans::from(vec![
                Span::raw(format!(
                    "s: {}   ",
                    if scanning {
                        "stop scanning"
                    } else {
                        "start scanning"
                    }
                )),
                // Span::raw("h: hide unnamed   "),
                Span::raw("c: connect   "),
                Span::raw("d: disconnect   "),
                Span::raw("q: quit"),
            ]);

            let commands = Paragraph::new(commands_str).block(blue_box(None));

            // rect.render_widget(title, chunks[0]);
            rect.render_widget(title, chunks[0]);
            rect.render_widget(commands, chunks[2]);
            rect.render_stateful_widget(list, main_chunks[0], &mut state.devices().list_state);
            rect.render_widget(blue_box(None), right_chunks[1]);
            rect.render_widget(device_details, right_chunks[0]);
            // rect.render_widget(known_devices)

            if false {
                let block = Block::default()
                    .title("Popup")
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Blue));
                let vertical_chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(45),
                        Constraint::Length(4),
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

                rect.render_widget(block, popup_chunk);
            }
        })
        .unwrap();
}
