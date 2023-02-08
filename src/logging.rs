use crate::settings::LogSettings;
use log::info;

pub fn initialize_logging(log_settings: LogSettings) -> Result<(), Box<dyn std::error::Error>> {
    tui_logger::init_logger(log_settings.level).unwrap();
    tui_logger::set_default_level(log_settings.level);

    let mut log_file_path = log_settings.folder.clone();

    let datetime = time::OffsetDateTime::now_utc();
    log_file_path.push(
        datetime
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap(),
    );
    log_file_path.set_extension("log");

    if log_settings.log_to_file {
        std::fs::create_dir_all(&log_settings.folder)?;
        tui_logger::set_log_file(log_file_path.to_str().unwrap())?;
        info!("Logging to file {}", log_file_path.to_str().unwrap());
    }
    Ok(())
}
