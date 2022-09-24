use tui::style::{Color, Style};
use tui_logger::TuiLoggerWidget;

use super::statics::blue_box;

pub fn get_logger_widget() -> TuiLoggerWidget<'static> {
    tui_logger::TuiLoggerWidget::default()
        .block(blue_box(None))
        .style_error(Style::default().fg(Color::Red))
        .style_warn(Style::default().fg(Color::Yellow))
        .style_info(Style::default().fg(Color::White))
        .output_level(Some(tui_logger::TuiLoggerLevelOutput::Long))
        .output_file(false)
        .output_target(false)
        .output_line(false)
        .output_timestamp(Some("%F %H:%M:%S%.3f".to_string()))
}
