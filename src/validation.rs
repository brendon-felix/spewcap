use std::path::Path;
use serialport5::available_ports;
use crate::error::{Result, SpewcapError};

const STANDARD_BAUD_RATES: &[u32] = &[
    110, 300, 600, 1200, 2400, 4800, 9600, 14400, 19200, 38400, 57600, 115200, 230400, 460800, 921600
];

pub fn validate_baud_rate(baud_rate: u32) -> Result<u32> {
    if STANDARD_BAUD_RATES.contains(&baud_rate) {
        Ok(baud_rate)
    } else {
        Err(SpewcapError::InvalidBaudRate(baud_rate))
    }
}

pub fn validate_port_name(port_name: &str) -> Result<String> {
    let available_ports = available_ports()
        .map_err(|e| SpewcapError::SerialPort(e))?;
    
    let port_exists = available_ports
        .iter()
        .any(|port| port.port_name == port_name);
    
    if port_exists {
        Ok(port_name.to_string())
    } else {
        Err(SpewcapError::InvalidPort(port_name.to_string()))
    }
}

pub fn validate_file_path(path: &str) -> Result<String> {
    let path_buf = Path::new(path);
    
    if !path_buf.is_absolute() && !path_buf.exists() {
        let current_dir = std::env::current_dir()
            .map_err(|e| SpewcapError::InvalidFilePath(format!("Cannot access current directory: {e}")))?;
        
        let full_path = current_dir.join(path_buf);
        validate_directory_writable(full_path.parent().unwrap_or(&current_dir))?;
    } else if path_buf.is_absolute() {
        if let Some(parent) = path_buf.parent() {
            if !parent.exists() {
                return Err(SpewcapError::InvalidFilePath(format!("Parent directory does not exist: {}", parent.display())));
            }
            validate_directory_writable(parent)?;
        }
    }
    
    Ok(path.to_string())
}

pub fn validate_directory_path(path: &str) -> Result<String> {
    let path_buf = Path::new(path);
    
    if !path_buf.exists() {
        return Err(SpewcapError::InvalidFilePath(format!("Directory does not exist: {}", path)));
    }
    
    if !path_buf.is_dir() {
        return Err(SpewcapError::InvalidFilePath(format!("Path is not a directory: {}", path)));
    }
    
    validate_directory_writable(path_buf)?;
    
    Ok(path.to_string())
}

fn validate_directory_writable(dir: &Path) -> Result<()> {
    let temp_file = dir.join(".spewcap_write_test");
    
    match std::fs::write(&temp_file, b"test") {
        Ok(_) => {
            let _ = std::fs::remove_file(temp_file);
            Ok(())
        }
        Err(e) => Err(SpewcapError::InvalidFilePath(format!(
            "Directory is not writable: {} ({})", 
            dir.display(), 
            e
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_baud_rate_valid() {
        assert!(validate_baud_rate(115200).is_ok());
        assert!(validate_baud_rate(9600).is_ok());
    }

    #[test]
    fn test_validate_baud_rate_invalid() {
        assert!(validate_baud_rate(123456).is_err());
        assert!(validate_baud_rate(0).is_err());
    }

    #[test]
    fn test_validate_file_path_current_dir() {
        assert!(validate_file_path("test.log").is_ok());
    }
}
