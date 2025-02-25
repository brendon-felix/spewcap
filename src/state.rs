use std::sync::{Arc, Mutex};
use crate::log::Log;

pub struct State {
    pub capture_paused: bool,
    pub active_log: Option<Log>,
    pub quit_requested: bool,
}

pub fn init_state() -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(
        State{
            capture_paused: false,
            active_log: None,
            quit_requested: false,
        }
    ))
}
