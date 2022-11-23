use std::path::PathBuf;

use dirs::home_dir;
use log::info;

fn get_logs_dir() -> PathBuf {
    let mut logs_dir = home_dir().unwrap();
    logs_dir.push(".bltui");
    logs_dir.push("logs");
    logs_dir
}

fn get_log_file_path(logs_dir: PathBuf) -> PathBuf {
    let mut log_file = logs_dir;
    let datetime = time::OffsetDateTime::now_utc();
    log_file.push(datetime.format(&time::format_description::well_known::Rfc3339).unwrap());
    log_file.set_extension("log");
    log_file
}

pub fn init_tui_logger(log_level: log::LevelFilter) {
    tui_logger::init_logger(log_level).unwrap();
    tui_logger::set_default_level(log_level);
}

pub fn init_file_logging() -> Result<(), Box<dyn std::error::Error>> {
    let logs_dir = get_logs_dir();
    std::fs::create_dir_all(&logs_dir)?;
    let log_file = get_log_file_path(logs_dir);
    tui_logger::set_log_file(log_file.to_str().unwrap())?;
    info!("Logging to file {}", log_file.to_str().unwrap());
    Ok(())
}
