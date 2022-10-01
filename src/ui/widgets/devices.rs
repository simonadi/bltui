use tui::{
    style::{Color, Style},
    text::{Span, Spans, Text},
    widgets::List,
};

use crate::bluetooth::devices::{Device, Devices};

use super::statics::blue_box;

impl From<Device> for Text<'_> {
    fn from(device: Device) -> Text<'static> {
        Text::from(vec![Spans::from(vec![
            Span::from(if device.name == "Unknown" {
                format!("{} ({})", device.name, device.address)
            } else {
                device.name
            }),
            Span::styled(
                if device.connected { " (Connected)" } else { "" },
                Style::default().fg(Color::Green),
            ),
        ])])
    }
}

pub fn devices_list<'a>(devices: &Devices) -> List<'a> {
    List::new(devices.list_items())
        .highlight_style(Style::default().bg(Color::White).fg(Color::Black))
        .highlight_symbol("->")
        .block(blue_box(Some(format!(
            "Devices ({}/{})",
            {
                if let Some(index) = devices.list_state.selected() {
                    index + 1
                } else {
                    0
                }
            },
            devices.len()
        ))))
}
