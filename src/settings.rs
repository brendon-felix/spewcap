// use clap::Parser;
use std::fs;
use toml;
// use serde::{Deserialize, Serialize};
use serde::Deserialize;
pub struct Settings {
    pub port: String,
    pub baud_rate: u32,
    pub timestamps: bool,
    // pub log_folder: &str,
    // pub clear_on_start: bool,
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            port: "COM7".to_string(),
            baud_rate: 115200,
            timestamps: false,
            // log_folder: r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs",
            // clear_on_start: false,
        }
    }
}
impl Settings {
    pub fn new(config: Option<Config>) -> Self {
        let default = Settings::default();
        if let Some(config) = config {
            Settings {
                port: config.port.unwrap_or(default.port),
                baud_rate: config.baud_rate.unwrap_or(default.baud_rate),
                timestamps: config.timestamps.unwrap_or(default.timestamps),
                // ..Settings::default()
            }
        } else {
            default
        }
    }
}



#[derive(Deserialize)]
pub struct Config {
    port: Option<String>,
    baud_rate: Option<u32>,
    timestamps: Option<bool>,
    // log_folder: Option<String>,
    // clear_on_start: Option<bool>,
}
impl Config {
    pub fn load(file_path: &str) -> Option<Self> {
        let toml_str = fs::read_to_string(file_path).ok()?;
        toml::from_str(&toml_str).ok()?
    }
}

// #[derive(Clone)]
// pub struct Settings {
//     pub port: String,
//     pub baud_rate: u32,
//     pub timestamps: bool,
//     pub log_folder: String,
//     pub clear_on_start: bool,
// }

// fn get_config(filename: String) -> Option<Config> {
    
// }

// pub fn get_settings() -> Settings {
//     // let args = Args::parse();
//     if let Some(config) = get_config("config.toml".to_string()) {
//         Settings {
//             port: args.port.or(config.port).expect("No port specified!"),
//             baud_rate: args.baud_rate.or(config.baud_rate).unwrap_or(115200),
//             timestamps: args.timestamps.or(config.timestamps).unwrap_or(false),
//             // log_folder: args.log_folder.or(config.log_folder)
//             //     .unwrap_or(r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs".to_string()),
//             // clear_on_start: config.clear_on_start.unwrap_or(false),
//         }
//     } else {
//         Settings {
//             port: args.port.expect("No port specified!"),
//             baud_rate: args.baud_rate.unwrap_or(115200),
//             timestamps: args.timestamps.unwrap_or(false),
//             // log_folder: args.log_folder
//             //     .unwrap_or(r"C:\Users\felixb\OneDrive - HP Inc\Debugs\Springs".to_string()),
//             // clear_on_start: false,
//         }
//     }
    
// }
