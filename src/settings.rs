use std::path::{Path, PathBuf};

use clap::Parser;
use dirs::home_dir;
use log::LevelFilter;
use serde::Deserialize;

use crate::Error;

#[derive(Deserialize, Default)]
struct Config {
    adapter: Option<String>,
    log_path: Option<PathBuf>,
}

impl Config {
    fn read_from(config_path: &Path) -> Result<Config, Error> {
        let content = std::fs::read_to_string(config_path)?;

        match toml::from_str(&content) {
            Ok(conf) => Ok(conf),
            Err(_) => Err(Error::InvalidConfigFile(config_path.to_path_buf())),
        }
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct CliSettings {
    /// Activate debug/trace messages
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// Display devices with an unknown name
    #[arg(short = 'u', long, action)]
    show_unknown: bool,

    /// Log to file (/$homedir/.bltui/logs)
    #[arg(short, long, action)]
    log_to_file: bool,

    /// Specify which adapter to use
    #[arg(short, long)]
    adapter: Option<String>,
}

impl CliSettings {
    fn get_log_level(&self) -> LevelFilter {
        match self.debug {
            0 => LevelFilter::Info,
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => LevelFilter::Trace,
        }
    }
}

fn get_bltui_folder() -> PathBuf {
    match std::env::var("BLTUI_FOLDER") {
        Ok(path_str) => PathBuf::from(path_str),
        Err(_) => {
            let mut path = home_dir().expect("Could not get home directory");
            path.push(".bltui");
            path
        }
    }
}

pub struct LogSettings {
    pub level: LevelFilter,
    pub log_to_file: bool,
    pub folder: PathBuf,
}

pub struct AppSettings {
    pub log_settings: LogSettings,
    pub adapter: Option<String>,
    pub show_unknown: bool,
}

impl AppSettings {
    pub fn parse() -> AppSettings {
        let bltui_folder = get_bltui_folder();
        let mut config_path = bltui_folder.clone();
        config_path.push("config.toml");
        let file_config = match Config::read_from(&config_path) {
            Ok(conf) => conf,
            Err(err) => match err {
                Error::InvalidConfigFile(path) => {
                    panic!("Invalid config file at {:?}", path);
                }
                Error::IOError(_) => Config::default(),
                _ => {
                    panic!("Unexpected error");
                }
            },
        };
        let cli_settings = CliSettings::parse();

        AppSettings {
            log_settings: LogSettings {
                level: cli_settings.get_log_level(),
                log_to_file: cli_settings.log_to_file,
                folder: if file_config.log_path.is_some() {
                    file_config.log_path.unwrap()
                } else {
                    let mut log_folder = bltui_folder;
                    log_folder.push("logs");
                    log_folder
                },
            },
            adapter: {
                if cli_settings.adapter.is_some() {
                    cli_settings.adapter
                } else {
                    file_config.adapter
                }
            },
            show_unknown: cli_settings.show_unknown,
        }
    }
}
