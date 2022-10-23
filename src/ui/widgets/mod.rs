use tui::style::{Color, Style};

pub(super) mod device_details;
pub(super) mod devices;
pub(super) mod logger;
pub mod popup;
pub mod statics;

pub(self) fn text_style() -> Style {
    Style::default().fg(Color::White)
}
