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

    /// Log to file (/$homedir/.bluetui/logs)
    #[clap(short, long, action)]
    log_to_file: bool,
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

    let mut app = App::new(logging_level, cli.show_unknown, cli.log_to_file).await;

    app.start().await;

    Ok(())
    // return Ok(());
}
