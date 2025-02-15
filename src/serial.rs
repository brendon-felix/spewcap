use std::sync::{Arc, Mutex};
use serialport5::{self, SerialPort, SerialPortBuilder};
use std::io::{self, BufWriter, Read, Write};
use colored::Colorize;
use std::fmt;
use crate::settings::Settings;
use crate::state::State;
use crate::utils::{sleep, print_separator, try_create_log};

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
    fn get_line(&mut self) -> Option<&str> {
        if let Some(newline_index) = self.buffer[self.line_index..self.index].iter().position(|&b| b == b'\n') {
            let line_end = self.line_index + newline_index + 1;
            let line_bytes = &self.buffer[self.line_index..line_end];
            self.line_index = line_end;
            let line = std::str::from_utf8(line_bytes).expect("Could not read line");
            Some(line)
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
    // println!("SERIAL LOOP");
    let mut first_attempt = true;
    let mut status: Status;
    let port_name = &settings.port;
    loop {
        match open_serial_port(port_name, settings.baud_rate) {
            Some(port) => {
                status = Status::Connected;
                print_status_msg(port_name, status);
                {
                    let mut state = shared_state.lock().unwrap();
                    state.log = if settings.log_on_start {
                        try_create_log(state.timestamps)
                    } else {
                        None
                    };            
                }
                // print_separator("");
                let mut stdout = Box::new(BufWriter::with_capacity(1024, io::stdout()));
                let _ = read_loop(port, Arc::clone(&shared_state), &mut stdout);
                status = Status::Disconnected;
                print_status_msg(port_name, status);
            }
            None => {
                if first_attempt {
                    status = Status::NotConnected;
                    print_status_msg(port_name, status);
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

fn read_loop<W: Write>(mut port: SerialPort, shared_state: Arc<Mutex<State>>, stdout: &mut W) {
    let mut buffer = Buffer::new();
    loop {
        let mut data_buffer = [0; 256];
        match port.read(&mut data_buffer) {
            Ok(0) => return,
            Ok(data_size) => {
                buffer.write(&data_buffer, data_size);
                while let Some(line) = buffer.get_line() {
                    output_line(line, stdout, &shared_state);
                }
                buffer.shift_remaining();
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => break,
            Err(e) => {
                // println!("Failed to read port: {}", e);
                panic!("Failed to read port: {}", e);
            }
        }
    }
}

fn output_line<W: Write>(line: &str, stdout: &mut W, shared_state: &Arc<Mutex<State>>, ) {
    let mut state = shared_state.lock().unwrap();
    stdout.write_all(line.as_bytes()).expect("Failed to write to stdout");
    if let Some(log) = &mut state.log {
        if log.enabled {
            log.write_line(line);
        }
    }
    stdout.flush().expect("Failed to flush stdout");
}