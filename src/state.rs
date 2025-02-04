// use anyhow::{Result, Context};
// use std::sync::{Arc, Mutex};

// use std::mem::take;
// use crate::utils::*;
// use crate::settings::Settings;
use crate::log::Log;

#[derive(Default)]
pub struct State {
    // pub timestamps: bool,
    // pub settings: Settings,
    pub log: Option<Log>,
}



