use std::sync::{Arc, Mutex};
use crate::log::Log;

#[derive(Default)]
pub struct State {
    pub timestamps: bool,
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
