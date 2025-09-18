use colored::Colorize;
use serialport5::{self, SerialPort, SerialPortBuilder};
use std::io::{self, BufWriter, Read, Write};
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::buffer::LineBuffer;
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
    let mut line_buffer = LineBuffer::new();
    let mut data_buffer = [0; 2048];
    
    loop {
        if quit_requested(&shared_state) {
            return ConnectionStatus::Connected;
        }

        match read_data_from_port(&mut port, &mut data_buffer) {
            ReadResult::Data(data_size) => {
                process_received_data(&mut line_buffer, &data_buffer, data_size, stdout, shared_state);
            }
            ReadResult::NoData => sleep_ms(10),
            ReadResult::Error => return ConnectionStatus::Disconnected,
        }
    }
}

enum ReadResult {
    Data(usize),
    NoData,
    Error,
}

fn read_data_from_port(port: &mut SerialPort, data_buffer: &mut [u8]) -> ReadResult {
    match port.read(data_buffer) {
        Ok(0) => ReadResult::NoData,
        Ok(data_size) => ReadResult::Data(data_size),
        Err(e) if e.kind() == io::ErrorKind::TimedOut => ReadResult::NoData,
        Err(_) => ReadResult::Error,
    }
}

fn process_received_data<W: Write>(
    line_buffer: &mut LineBuffer,
    data_buffer: &[u8],
    data_size: usize,
    stdout: &mut W,
    shared_state: &State,
) {
    line_buffer.write(data_buffer, data_size);
    let lines_processed = process_complete_lines(line_buffer, stdout, shared_state);
    
    if lines_processed > 0 {
        flush_output(stdout);
    }
}

fn process_complete_lines<W: Write>(
    line_buffer: &mut LineBuffer,
    stdout: &mut W,
    shared_state: &State,
) -> usize {
    let mut lines_processed = 0;
    
    while let Some(line) = line_buffer.next_line() {
        output_line(&line, stdout, shared_state);
        lines_processed += 1;
        
        // yield occasionally for very high throughput
        if lines_processed % 100 == 0 {
            flush_output(stdout);
            std::thread::yield_now();
        }
    }
    
    lines_processed
}

fn flush_output<W: Write>(stdout: &mut W) {
    if let Err(e) = stdout.flush() {
        print_error(&format!("Failed to flush stdout: {e}"));
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
