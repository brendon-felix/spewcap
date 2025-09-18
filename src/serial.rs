use colored::Colorize;
use serialport5::{self, SerialPort, SerialPortBuilder};
use std::io::{self, BufWriter, Read, Write};
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::settings::Settings;
use crate::state::State;
use crate::utils::{get_log_state, print_error, print_message, quit_requested, sleep_ms};
use crate::error::Result;
use crate::validation;

pub enum ConnectionStatus {
    Connected,
    NotConnected,
    Disconnected,
}

struct Buffer {
    buffer: Vec<u8>,
    index: usize,
    line_index: usize,
}
impl Buffer {
    fn new() -> Self {
        Buffer {
            buffer: Vec::with_capacity(8192),
            index: 0,
            line_index: 0,
        }
    }
    
    fn write(&mut self, data_buffer: &[u8], data_size: usize) {
        if self.index + data_size > self.buffer.len() {
            self.buffer.resize(self.index + data_size, 0);
        }

        self.buffer[self.index..self.index + data_size].copy_from_slice(&data_buffer[..data_size]);
        self.index += data_size;
    }
    
    fn next_line(&mut self) -> Option<&str> {
        if let Some(newline_index) = self.buffer[self.line_index..self.index]
            .iter()
            .position(|&b| b == b'\n')
        {
            let line_end = self.line_index + newline_index + 1;
            let line_bytes = &self.buffer[self.line_index..line_end];
            self.line_index = line_end;
            match std::str::from_utf8(line_bytes) {
                Ok(line) => Some(line),
                Err(e) if e.valid_up_to() > 0 => {
                    let valid_bytes = &line_bytes[..e.valid_up_to()];
                    match std::str::from_utf8(valid_bytes) {
                        Ok(valid_line) => Some(valid_line),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        } else {
            None
        }
    }
    
    fn shift_remaining(&mut self) {
        let remaining_bytes = self.index - self.line_index;
        if remaining_bytes > 0 && self.line_index > 0 {
            self.buffer.copy_within(self.line_index..self.index, 0);
        }
        self.line_index = 0;
        self.index = remaining_bytes;

        // Improved memory management: prevent unbounded growth
        if self.buffer.capacity() > 16384 && remaining_bytes < 4096 {
            self.buffer.shrink_to(8192);
        }
        
        // Additional safety: if buffer gets extremely large, reset it
        if self.buffer.capacity() > 65536 {
            self.buffer = Vec::with_capacity(8192);
            self.index = 0;
            self.line_index = 0;
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        // Explicitly clear buffer to ensure memory is released
        self.buffer.clear();
        self.buffer.shrink_to_fit();
    }
}

pub fn connect_loop(settings: Settings, shared_state: State) -> Result<()> {
    let mut first_attempt = true;
    let port_name = &settings.port;
    loop {
        if quit_requested(&shared_state) {
            break;
        }
        match open_serial_port(port_name, settings.baud_rate) {
            Some(port) => {
                print_status(port_name, ConnectionStatus::Connected);
                let mut stdout = Box::new(BufWriter::with_capacity(1024, io::stdout()));
                let status = read_loop(port, &shared_state, &mut stdout);
                match status {
                    ConnectionStatus::Connected => break, // still connected means we are quitting
                    ConnectionStatus::Disconnected => {
                        print_status(port_name, ConnectionStatus::Disconnected)
                    }
                    ConnectionStatus::NotConnected => {
                        print_status(port_name, ConnectionStatus::NotConnected)
                    }
                }
            }
            None => {
                if first_attempt {
                    print_status(port_name, ConnectionStatus::NotConnected);
                }
                sleep_ms(500); // wait before retrying
            }
        }
        first_attempt = false;
    }
    Ok(())
}

fn print_status(port_name: &str, status: ConnectionStatus) {
    match status {
        ConnectionStatus::Connected => {
            print_message(format!("{} {}", port_name, "connected".green()))
        }
        ConnectionStatus::NotConnected => {
            print_message(format!("{} {}", port_name, "not connected".yellow()))
        }
        ConnectionStatus::Disconnected => {
            print_message(format!("{} {}", port_name, "disconnected".red()))
        }
    }
}

fn open_serial_port(port: &str, baud_rate: u32) -> Option<SerialPort> {
    // Validate inputs before attempting to open the port
    if let Err(e) = validation::validate_port_name(port) {
        print_error(&format!("Invalid port: {e}"));
        return None;
    }
    
    if let Err(e) = validation::validate_baud_rate(baud_rate) {
        print_error(&format!("Invalid baud rate: {e}"));
        return None;
    }
    
    SerialPortBuilder::new()
        .baud_rate(baud_rate)
        .read_timeout(Some(Duration::from_millis(100)))  // 100ms timeout
        .open(port)
        .ok()
}

fn read_loop<W: Write>(
    mut port: SerialPort,
    shared_state: &State,
    stdout: &mut W,
) -> ConnectionStatus {
    let mut line_buffer = Buffer::new();
    let mut data_buffer = [0; 2048];
    loop {
        if quit_requested(&shared_state) {
            return ConnectionStatus::Connected;
        }

        match port.read(&mut data_buffer) {
            Ok(0) => sleep_ms(10),
            Ok(data_size) => {
                line_buffer.write(&data_buffer, data_size);
                let mut lines_processed = 0;
                while let Some(line) = line_buffer.next_line() {
                    output_line(line, stdout, &shared_state);
                    lines_processed += 1;
                    // yield occasionally for very high throughput
                    if lines_processed % 100 == 0 {
                        if let Err(e) = stdout.flush() {
                            print_error(&format!("Failed to flush stdout: {e}"));
                        }
                        std::thread::yield_now();
                    }
                }
                line_buffer.shift_remaining();
                if lines_processed > 0 {
                    if let Err(e) = stdout.flush() {
                        print_error(&format!("Failed to flush stdout: {e}"));
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::TimedOut => {
                continue;
            }
            Err(_) => {
                return ConnectionStatus::Disconnected;
            }
        }
    }
}

fn output_line<W: Write>(line: &str, stdout: &mut W, shared_state: &State) {
    if shared_state.capture_paused.load(Ordering::Relaxed) {
        return;
    }

    if let Err(e) = stdout.write_all(line.as_bytes()) {
        print_error(&format!("Failed to write to stdout: {e}"));
    }

    let mut log_state = match get_log_state(shared_state) {
        Ok(state) => state,
        Err(e) => {
            print_error(&format!("Failed to acquire lock on log state during serial output: {e}"));
            return;
        }
    };
    if let Some(log) = &mut log_state.active_log {
        if log.is_enabled() {
            log.write_line(line);
        }
    }
}
