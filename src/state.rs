use crate::log::Log;
use std::sync::{Arc, Mutex};

pub struct State {
    pub capture_paused: bool,
    pub active_log: Option<Log>,
    pub quit_requested: bool,
}

pub fn init_state() -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(State {
        capture_paused: false,
        active_log: None,
        quit_requested: false,
    }))
}
