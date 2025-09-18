use crate::log::LogFile;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

pub struct SharedState {
    pub capture_paused: AtomicBool,
    pub quit_requested: AtomicBool,
    pub log_state: Mutex<LogState>,
}

pub struct LogState {
    pub active_log: Option<LogFile>,
}

pub type State = Arc<SharedState>;

pub fn init_state() -> State {
    Arc::new(SharedState {
        capture_paused: AtomicBool::new(false),
        quit_requested: AtomicBool::new(false),
        log_state: Mutex::new(LogState { active_log: None }),
    })
}
