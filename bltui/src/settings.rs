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
        Ok(path_str) => {
            println!("Using BLTUI_FOLDER environment variable");
            PathBuf::from(path_str)
        }
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

pub struct DisplaySettings {
    pub show_unknown: bool,
}

pub struct AppSettings {
    pub log_settings: LogSettings,
    pub display_settings: DisplaySettings,
    pub adapter: Option<String>,
}

impl AppSettings {
    fn from_cli_and_file_settings(cli_settings: CliSettings, file_config: Config) -> AppSettings {
        AppSettings {
            display_settings: DisplaySettings {
                show_unknown: cli_settings.show_unknown,
            },
            log_settings: LogSettings {
                level: cli_settings.get_log_level(),
                log_to_file: cli_settings.log_to_file,
                folder: if file_config.log_path.is_some() {
                    file_config.log_path.unwrap()
                } else {
                    let mut log_folder = get_bltui_folder();
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
        }
    }

    pub fn parse() -> AppSettings {
        let mut config_path = get_bltui_folder();
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

        AppSettings::from_cli_and_file_settings(cli_settings, file_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> TempDir {
            let mut temp_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            temp_dir.push("temp");
            temp_dir.push(format!("{}", rand::thread_rng().gen_range(0..1000)));
            std::fs::create_dir_all(&temp_dir).unwrap();
            TempDir { path: temp_dir }
        }

        fn path(&self) -> &Path {
            self.path.as_path()
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.path).unwrap();
        }
    }

    #[test]
    fn test_get_bltui_folder_env() {
        let folder = PathBuf::from("/home/simon");
        std::env::set_var("BLTUI_FOLDER", folder.to_str().unwrap());
        let returned_folder = get_bltui_folder();
        std::env::remove_var("BLTUI_FOLDER");
        assert_eq!(folder, returned_folder);
    }

    #[test]
    fn test_get_bltui_folder_home() {
        let returned_folder = get_bltui_folder();
        let mut folder = home_dir().unwrap();
        folder.push(".bltui");
        assert_eq!(folder, returned_folder);
    }

    #[test]
    fn test_config_parsing_all() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let adapter = "hci0";
        let log_path = "/log/path";
        let config = format!("adapter = \"{}\"\nlog_path = \"{}\"", adapter, log_path);
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path()).unwrap();

        assert_eq!(config.adapter.unwrap(), adapter);
        assert_eq!(config.log_path.unwrap(), PathBuf::from(log_path));
    }

    #[test]
    fn test_config_parsing_adapter_only() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let adapter = "hci0";
        let config = format!("adapter = \"{}\"", adapter);
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path()).unwrap();

        assert_eq!(config.adapter.unwrap(), adapter);
        assert!(config.log_path.is_none());
    }

    #[test]
    fn test_config_parsing_log_path_only() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let log_path = "/log/path";
        let config = format!("log_path = \"{}\"", log_path);
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path()).unwrap();

        assert!(config.adapter.is_none());
        assert_eq!(config.log_path.unwrap(), PathBuf::from(log_path));
    }

    #[test]
    fn test_config_parsing_nothing() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let config = "";
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path()).unwrap();

        assert!(config.adapter.is_none());
        assert!(config.log_path.is_none());
    }

    #[test]
    fn test_config_parsing_invalid() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let config = "invalid";
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path());

        assert!(config.is_err());
    }

    #[test]
    fn test_config_parsing_non_existent() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path());

        assert!(config.is_err());
    }

    #[test]
    fn test_config_parsing_invalid_log_path() {
        let temp_dir = TempDir::new();
        let temp_dir_path = temp_dir.path();
        let log_path = "/log \\ path";
        let config = format!("log_path = \"{}\"", log_path);
        std::fs::write(temp_dir_path.join("config.toml"), config).unwrap();

        let config = Config::read_from(temp_dir_path.join("config.toml").as_path());

        assert!(config.is_err());
    }
}
