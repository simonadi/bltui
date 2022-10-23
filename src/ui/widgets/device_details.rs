use tui::{
    layout::Alignment,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::Paragraph,
};

use crate::bluetooth::devices::Device;

use super::{statics::blue_box, text_style};

pub fn get_device_details(selected_device: Option<Device>) -> Paragraph<'static> {
    let device_details_str = if let Some(device) = selected_device {
        Text::from(vec![
            Spans::from(Span::styled(
                device.name,
                text_style().add_modifier(Modifier::BOLD),
            )),
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
            // Spans::from(vec![
            //     Span::raw("Paired : "),
            //     if device.paired {
            //         Span::styled("yes", Style::default().fg(Color::Green))
            //     } else {
            //         Span::styled("no", Style::default().fg(Color::Red))
            //     },
            // ]),
            // Spans::from(vec![
            //     Span::raw("Trusted : "),
            //     if device.trusted {
            //         Span::styled("yes", Style::default().fg(Color::Green))
            //     } else {
            //         Span::styled("no", Style::default().fg(Color::Red))
            //     },
            // ]),
        ])
    } else {
        Text::from(vec![Spans::from(vec![Span::raw("")])])
    };

    Paragraph::new(device_details_str)
        .style(text_style())
        .alignment(Alignment::Left)
        .block(blue_box(Some(String::from("Details"))))
}
