use thiserror::Error;

#[derive(Error, Debug)]
pub enum SpewcapError {
    #[error("Serial port error: {0}")]
    SerialPort(#[from] serialport5::Error),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    // #[error("Configuration error: {0}")]
    // Config(String),
    
    #[error("Settings error: {0}")]
    Settings(String),
    
    #[error("Dialog error: {0}")]
    Dialog(String),
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("Thread join error: {0}")]
    ThreadJoin(String),
    
    #[error("Terminal error: {0}")]
    Terminal(String),
    
    #[error("Invalid baud rate: {0}")]
    InvalidBaudRate(u32),
    
    #[error("Invalid port: {0}")]
    InvalidPort(String),
    
    #[error("Invalid file path: {0}")]
    InvalidFilePath(String),
    
    #[error("Log error: {0}")]
    Log(String),
    
    #[error("Signal handling error: {0}")]
    Signal(String),
    
    #[error("No serial ports found")]
    NoPortsFound,
    
    // #[error("User cancelled operation")]
    // UserCancelled,
}

pub type Result<T> = std::result::Result<T, SpewcapError>;
