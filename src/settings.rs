use std::fs;
use toml;
use std::path::PathBuf;
use serialport5::{available_ports, SerialPortType};
use dialoguer::Select;
use serde::Deserialize;
use clap::Parser;

use crate::utils;

macro_rules! merge_config {
    ($config:expr, $args:expr, $( $field:ident ),*) => {
        $(
            if let Some(value) = $args.$field {
                $config.$field = Some(value);
            }
        )*
    }
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub port: String,
    pub baud_rate: u32,
    pub timestamps: bool,
    pub log_folder: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct Args {
    /// Port name (eg. "COM3", "/dev/ttyUSB0")
    #[arg(short, long)]
    port: Option<String>,

    /// Baud rate
    #[arg(short, long)]
    baud_rate: Option<u32>,

    /// Prepend timestamps in log
    #[arg(short, long)]
    timestamps: bool,

    /// Prepend timestamps in log
    #[arg(short, long)]
    log_on_start: bool,

    /// Folder path to save logs
    #[arg(short = 'f', long)]
    log_folder: Option<String>,

    #[arg(long)]
    list: bool
}

#[derive(Default, Deserialize, Debug)]
pub struct Config {
    port: Option<String>,
    baud_rate: Option<u32>,
    timestamps: Option<bool>,
    log_folder: Option<String>,
    pub log_on_start: Option<bool>,
    // pub clear_on_start: Option<bool>,
    pub disable_welcome: Option<bool>,
}
impl Config {
    fn load(file_path: PathBuf) -> Option<Self> {
        let toml_str = fs::read_to_string(file_path).ok()?;
        toml::from_str(&toml_str).ok()?
    }
    fn use_args(&mut self, args: Args) {
        merge_config!(self, args, port, baud_rate, log_folder);
        self.timestamps = Some(args.timestamps);
        self.log_on_start = Some(args.log_on_start);
    }
    pub fn select_missing(&mut self) -> Result<(), String> {
        if self.port.is_none() { self.port = Some(select_port()?); }
        if self.baud_rate.is_none() { self.baud_rate = Some(select_baud_rate()?); }
        Ok(())
    }
}

fn select_port() -> Result<String, String> {
    let ports = available_ports().expect("Could not find available ports!");
    if ports.is_empty() {
        eprintln!("No serial ports found!");
        std::process::exit(0);
    }
    let port_names: Vec<&str> = ports.iter().map(|port| port.port_name.as_str()).collect();
    let port_descriptions: Vec<String> = ports.iter().map(|port| {
        let mut description = format!("{}", port.port_name);
        if let SerialPortType::UsbPort(info) = &port.port_type {
            description = format!("{} {}", info.manufacturer.as_deref().unwrap_or(""), info.product.as_deref().unwrap_or("")).trim().to_string();
        }
        description
    }).collect();
    let selection = Select::new()
        .with_prompt("Select serial port")
        .default(0) // default is the first option
        .items(&port_descriptions)
        .interact()
        .map_err(|e| format!("No serial port selected: {e}"))?;
    Ok(port_names[selection].to_string())
}

fn select_baud_rate() -> Result<u32, String> {
    let options = [4800, 9600, 19200, 38400, 57600, 115200, 230400, 460800, 921600];
    let selection = Select::new()
        .with_prompt("Select baud rate")
        .default(5) // default is 115200 at index 5
        .items(&options)
        .interact()
        .map_err(|e| format!("No baud rate selected: {e}"))?;
    Ok(options[selection])
}

pub fn get_config() -> Config {
    let args = Args::parse();
    let curr_dir = utils::get_curr_directory();
    let path_in_curr_dir = curr_dir.join("spewcap_config.toml");
    let exe_dir = utils::get_exe_directory().unwrap_or(utils::get_curr_directory());
    let path_in_exe_dir = exe_dir.join("spewcap_config.toml");
    let mut config = Config::load(path_in_curr_dir).unwrap_or(
        Config::load(path_in_exe_dir)
            .unwrap_or(Config::default())
    );
    config.use_args(args);
    config
}

pub fn get_settings(config: &Config) -> Result<Settings, String> {
    let port = config.port.clone().ok_or(format!("Could not set port"))?;
    let baud_rate = config.baud_rate.ok_or(format!("Could not set baud rate"))?;
    let timestamps = config.timestamps.unwrap_or(false);
    let log_folder = config.log_folder.as_ref()
        .map(|f| PathBuf::from(f));
    Ok(Settings {
        port,
        baud_rate,
        timestamps,
        log_folder,
    })
}