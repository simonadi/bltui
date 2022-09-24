use bluetui::app::App;
use clap::{Parser, Subcommand};
use log::info;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
struct CliArgs {
    /// Activate debug/trace messages
    #[clap(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Display devices with an unknown name
    #[clap(short = 'u', long, action)]
    show_unknown: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = CliArgs::parse();

    let logging_level = match cli.debug {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        2 => log::LevelFilter::Trace,
        _ => log::LevelFilter::Trace,
    };

    let mut app = App::new(logging_level, cli.show_unknown).await;

    app.start().await;

    Ok(())
    // return Ok(());

    // let timestamp = chrono::Utc::now();
    // let log_dir = format!("logs/{}", timestamp.date());
    // std::fs::create_dir_all(&log_dir).expect("Could not create log directory");

    // simple_logging::log_to_file(
    //     format!("{}/{}.log", &log_dir, timestamp),
    //     log::LevelFilter::Debug,
    // )
    // .expect("Could not create log file");
}
