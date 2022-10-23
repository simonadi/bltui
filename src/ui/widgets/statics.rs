use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

use super::text_style;

pub fn blue_box(title: Option<String>) -> Block<'static> {
    let block = Block::default()
        .style(Style::default().fg(Color::Black))
        .borders(Borders::all())
        .border_style(Style::default().fg(Color::Blue));

    if let Some(title_str) = title {
        block.title(Span::styled(title_str, Style::default().fg(Color::White)))
    } else {
        block
    }
}

pub fn main_commands<'a>(scanning: bool) -> Paragraph<'a> {
    Paragraph::new(Spans::from(vec![
        Span::raw("⇵: move through devices   "),
        Span::raw(format!(
            "s: {}   ",
            if scanning {
                "stop scanning"
            } else {
                "start scanning"
            }
        )),
        Span::raw("c: connect   "),
        Span::raw("d: disconnect   "),
        Span::raw("q: quit"),
    ]))
    .style(text_style())
    .block(blue_box(None))
}

pub fn popup_commands<'a>() -> Paragraph<'a> {
    Paragraph::new(Spans::from(vec![
        Span::raw("⇵: move   "),
        Span::raw("↲: confirm"),
    ]))
    .style(text_style())
    .block(blue_box(None))
}

pub fn title<'a>() -> Paragraph<'a> {
    Paragraph::new("bltui")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(blue_box(None))
}
