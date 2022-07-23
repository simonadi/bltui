use chesapeake::app::App;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let timestamp = chrono::Utc::now();
    let log_dir = format!("logs/{}", timestamp.date());
    std::fs::create_dir_all(&log_dir).expect("Could not create log directory");
    // simple_logging::log_to_file(
    //     format!("{}/{}.log", &log_dir, timestamp),
    //     log::LevelFilter::Debug,
    // )
    // .expect("Could not create log file");
    let mut app = App::new().await;

    app.start().await;

    Ok(())
}
