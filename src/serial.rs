use std::sync::{Arc, Mutex};
use serialport5::{self, SerialPort, SerialPortBuilder};
use std::io::{self, BufWriter, Read, Write};
use colored::Colorize;

use crate::settings::Settings;
use crate::state::State;
use crate::utils::{quit_requested, print_message, print_error, sleep};

pub enum ConnectionStatus {
    Connected,
    NotConnected,
    Disconnected,
}

struct Buffer {
    buffer: [u8; 1024],
    index: usize,
    line_index: usize,
}
impl Buffer {
    fn new() -> Self {
        Buffer {
            buffer: [0; 1024],
            index: 0,
            line_index: 0,
        }
    }
    fn write(&mut self, data_buffer: &[u8], data_size: usize) {
        let remaining_buffer_space = self.buffer.len() - self.index;
        let num_bytes = remaining_buffer_space.min(data_size); // only use the remaining space available
        self.buffer[self.index.. self.index + num_bytes].copy_from_slice(&data_buffer[..num_bytes]);
        self.index += num_bytes;
    }
    fn parse_line(&mut self) -> Option<&str> {
        if let Some(newline_index) = self.buffer[self.line_index..self.index].iter().position(|&b| b == b'\n') {
            let line_end = self.line_index + newline_index + 1;
            let line_bytes = &self.buffer[self.line_index..line_end];
            self.line_index = line_end;
            match std::str::from_utf8(line_bytes) {
                Ok(line) => Some(line),
                Err(e) if e.valid_up_to() > 0 => {
                    let valid_bytes = &line_bytes[..e.valid_up_to()];
                    match std::str::from_utf8(valid_bytes) {
                        Ok(valid_line) => Some(valid_line),
                        Err(_) => None
                    }
                }
                Err(_) => None
            }
        } else {
            None
        }
    }
    fn shift_remaining(&mut self) {
        let remaining_bytes = self.index - self.line_index;
        self.buffer.copy_within(self.line_index..self.index, 0);
        self.line_index = 0;
        self.index = remaining_bytes;
    }
}

pub fn connect_loop(settings: Settings, shared_state: Arc<Mutex<State>>) {
    let mut first_attempt = true;
    let port_name = &settings.port;
    loop {
        if quit_requested(&shared_state) { break; }
        match open_serial_port(port_name, settings.baud_rate) {
            Some(port) => {
                print_status(port_name, ConnectionStatus::Connected);
                let mut stdout = Box::new(BufWriter::with_capacity(1024, io::stdout()));
                let status = read_loop(port, &shared_state, &mut stdout);
                match status {
                    ConnectionStatus::Connected => break, // still connected means we are quitting
                    ConnectionStatus::Disconnected => print_status(port_name, ConnectionStatus::Disconnected),
                    ConnectionStatus::NotConnected => print_status(port_name, ConnectionStatus::NotConnected),
                }
            },
            None => {
                if first_attempt {
                    print_status(port_name, ConnectionStatus::NotConnected);
                }
                sleep(500); // wait before retrying
            }
        }
        first_attempt = false;
    }
}

fn print_status(port_name: &str, status: ConnectionStatus) {
    match status {
        ConnectionStatus::Connected => print_message(format!("{} {}", port_name, "connected".green())),
        ConnectionStatus::NotConnected => print_message(format!("{} {}", port_name, "not connected".yellow())),
        ConnectionStatus::Disconnected => print_message(format!("{} {}", port_name, "disconnected".red())),
    }
}

fn open_serial_port(port: &str, baud_rate: u32) -> Option<SerialPort> {
    let baud_rate = baud_rate;
    SerialPortBuilder::new()
        .baud_rate(baud_rate)
        .open(port).ok()
}

fn read_loop<W: Write>(mut port: SerialPort, shared_state: &Arc<Mutex<State>>, stdout: &mut W) -> ConnectionStatus {
    let mut line_buffer = Buffer::new();
    let mut data_buffer = [0; 512];
    loop {
        if quit_requested(&shared_state) { return ConnectionStatus::Connected; }
        match port.bytes_to_read() {
            Ok(0) => sleep(100),
            Ok(_) => match port.read(&mut data_buffer) {
                Ok(data_size) => line_buffer.write(&data_buffer, data_size),
                _ => return ConnectionStatus::Disconnected,
            },
            _ => return ConnectionStatus::Disconnected,
        }
        while let Some(line) = line_buffer.parse_line() {
            output_line(line, stdout, &shared_state);
        }
        line_buffer.shift_remaining(); // move incomplete line to buffer start
        if let Err(e) = stdout.flush() {
            print_error(&format!("Failed to flush stdout: {e}"));
        }
    }
}

fn output_line<W: Write>(line: &str, stdout: &mut W, shared_state: &Arc<Mutex<State>>) {
    let mut state = shared_state.lock().unwrap();
    if state.capture_paused { return; }

    if let Err(e) = stdout.write_all(line.as_bytes()) {
        print_error(&format!("Failed to write to stdout: {e}"));
    }
    if let Some(log) = &mut state.active_log {
        if log.enabled {
            log.write_line(line);
        }
    }
}