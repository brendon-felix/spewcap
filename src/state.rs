// use std::sync::{Arc, Mutex};

// use std::mem::take;
// use crate::utils::*;
// use crate::settings::Settings;
use std::sync::{Arc, Mutex};
use crate::log::Log;

#[derive(Default)]
pub struct State {
    pub timestamps: bool,
    // pub settings: Settings,
    pub log: Option<Log>,
}

pub fn init_state(timestamps: bool) -> Arc<Mutex<State>> {
    Arc::new(Mutex::new(
        State{
            timestamps,
            log: None,
        }
    ))
}

pub fn shared_state(state: &Arc<Mutex<State>>) -> Arc<Mutex<State>> {
    Arc::clone(state)
}

