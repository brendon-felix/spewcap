use anyhow::{Context, Result, bail};
use std::sync::{Arc, Mutex};
use serialport5::{self, SerialPort, SerialPortBuilder};
use std::io::{self, BufWriter, Read, Write};
use colored::Colorize;
use std::fmt;
use crate::settings::Settings;
use crate::state::*;
use crate::utils::*;

enum Status {
    Connected,
    NotConnected,
    Disconnected,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Status::Connected => write!(f, "{}", "connected".bold().green()),
            Status::NotConnected => write!(f, "{}", "not connected".bold().yellow()),
            Status::Disconnected => write!(f, "{}", "disconnected".bold().red())
        }
    }
}

pub fn connect_loop(settings: Settings, shared_state: Arc<Mutex<State>>) {
    let mut first_attempt = true;
    let mut status: Status;
    loop {
        match open_serial_port(settings.port, settings.baud_rate) {
            Some(port) => {
                status = Status::Connected;
                print_status_msg(settings.port, status);
                print_separator("");
                let mut stdout = Box::new(BufWriter::with_capacity(1024, io::stdout()));
                let _ = read_serial_loop(port, Arc::clone(&shared_state), &mut stdout);
                status = Status::Disconnected;
                print_status_msg(settings.port, status);
            }
            None => {
                if first_attempt {
                    status = Status::NotConnected;
                    print_status_msg(settings.port, status);
                }
                sleep(500);
            }
        }
        first_attempt = false;
    }
}

fn print_status_msg(port_name: &str, status: Status) {
    print_separator(format!("{} {}", port_name, status));
}

fn open_serial_port(port: &str, baud_rate: u32) -> Option<SerialPort> {
    let baud_rate = baud_rate;
    SerialPortBuilder::new()
        .baud_rate(baud_rate)
        .open(port).ok()
}

fn read_serial_loop<W: Write>(mut port: SerialPort, shared_state: Arc<Mutex<State>>, stdout: &mut W) -> Result<()> {
    let mut buffer = [0; 1024];
    let mut buffer_index = 0;
    loop {
        let mut data = [0; 256];
        match port.read(&mut data) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                let remaining_buffer_space = buffer.len() - buffer_index;
                let bytes_to_copy = remaining_buffer_space.min(n); // only use the remaining space available
                buffer[buffer_index..buffer_index + bytes_to_copy].copy_from_slice(&data[..bytes_to_copy]);
                buffer_index += bytes_to_copy;
                let mut start_of_line = 0;

                while let Some(newline_index) = buffer[start_of_line..buffer_index].iter().position(|&b| b == b'\n') {
                    let end_index = start_of_line + newline_index + 1;
                    let line_bytes = &buffer[start_of_line..end_index];

                    if let Ok(line) = std::str::from_utf8(line_bytes) {
                        stdout.write_all(line.as_bytes()).context("Failed to write to stdout")?;
                        let mut state = shared_state.lock().unwrap();
                        if let Some(log) = &mut state.log {
                            if log.enabled {
                                log.write_line(line)?;
                            }
                        }
                        stdout.flush().context("Failed to flush stdout")?;
                    }
                    start_of_line = end_index;
                }
                let remaining_bytes = buffer_index - start_of_line;
                buffer.copy_within(start_of_line..buffer_index, 0);
                buffer_index = remaining_bytes;
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break Ok(()),
            Err(e) => {
                println!("Failed to read port: {}", e);
                bail!("");
            }
        }
    }
}
