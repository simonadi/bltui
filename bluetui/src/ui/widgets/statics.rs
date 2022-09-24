use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
};

pub fn blue_box(title: Option<String>) -> Block<'static> {
    let block = Block::default()
        .borders(Borders::all())
        .border_style(Style::default().fg(Color::Blue));

    if let Some(title_str) = title {
        block.title(Span::styled(title_str, Style::default().fg(Color::White)))
    } else {
        block
    }
}

pub fn commands<'a>(scanning: bool) -> Paragraph<'a> {
    Paragraph::new(Spans::from(vec![
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
    .block(blue_box(None))
}

pub fn title<'a>() -> Paragraph<'a> {
    Paragraph::new("bluetui")
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .block(blue_box(None))
}
