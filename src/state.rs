use colored::Colorize;
use std::fmt;

use std::sync::{Arc, Mutex};
use crate::log::Log;
use crate::settings::Settings;

pub enum ConnectionStatus {
    Connected,
    NotConnected,
    Disconnected,
}
impl fmt::Display for ConnectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConnectionStatus::Connected => write!(f, "{}", "CONNECTED".bold().green()),
            ConnectionStatus::NotConnected => write!(f, "{}", "NOT CONNECTED".bold().yellow()),
            ConnectionStatus::Disconnected => write!(f, "{}", "DISCONNECTED".bold().red())
        }
    }
}

// #[derive(Default)]
pub struct State {
    pub connection_status: ConnectionStatus,
    pub timestamps: bool,
    pub log: Option<Log>,
    // pub output_en: bool,
    pub quitting: bool,
}

pub fn init_state(settings: &Settings) -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(
        State{
            connection_status: ConnectionStatus::NotConnected,
            timestamps: settings.timestamps,
            log: None,
            // output_en: true,
            quitting: false,
        }
    ))
}
