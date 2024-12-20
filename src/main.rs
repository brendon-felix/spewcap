mod commands;
mod serial;
mod settings;
use crate::settings::get_settings;
use crate::commands::command_handler;
use anyhow::Result;

fn main() -> Result<()> {
    let settings = get_settings();
    command_handler(settings.clone())?;
    serial::open(settings)?;
    Ok(())
}
