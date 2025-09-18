use bytes::BytesMut;

pub struct LineBuffer {
    buffer: BytesMut,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self::with_capacity(8192)
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        LineBuffer {
            buffer: BytesMut::with_capacity(capacity),
        }
    }
    
    pub fn write(&mut self, data_buffer: &[u8], data_size: usize) {
        self.buffer.extend_from_slice(&data_buffer[..data_size]);
    }
    
    pub fn next_line(&mut self) -> Option<String> {
        let newline_pos = self.buffer.iter().position(|&b| b == b'\n')?;
        let line_bytes = self.buffer.split_to(newline_pos + 1);
        match String::from_utf8(line_bytes.to_vec()) {
            Ok(line) => Some(line),
            Err(e) => {
                // Try to recover valid UTF-8 portion
                let valid_up_to = e.utf8_error().valid_up_to();
                let bytes = e.into_bytes();
                if valid_up_to > 0 {
                    let valid_bytes = &bytes[..valid_up_to];
                    String::from_utf8(valid_bytes.to_vec()).ok()
                } else {
                    None
                }
            }
        }
    }
    
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.buffer.capacity()
    }
    
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.buffer.len()
    }
    
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }
}

impl Default for LineBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_write_and_read() {
        let mut buffer = LineBuffer::new();
        let data = b"Hello\nWorld\n";
        buffer.write(data, data.len());
        
        assert_eq!(buffer.next_line(), Some("Hello\n".to_string()));
        assert_eq!(buffer.next_line(), Some("World\n".to_string()));
        assert_eq!(buffer.next_line(), None);
    }

    #[test]
    fn test_buffer_partial_lines() {
        let mut buffer = LineBuffer::new();
        buffer.write(b"Hello", 5);
        assert_eq!(buffer.next_line(), None);
        buffer.write(b" World\n", 7);
        assert_eq!(buffer.next_line(), Some("Hello World\n".to_string()));
    }

    #[test]
    fn test_buffer_shift_remaining() {
        let mut buffer = LineBuffer::new();
        let data = b"Line1\nPartial";
        buffer.write(data, data.len());
        assert_eq!(buffer.next_line(), Some("Line1\n".to_string()));
        buffer.write(b"Line\n", 5);
        assert_eq!(buffer.next_line(), Some("PartialLine\n".to_string()));
    }

    #[test]
    fn test_empty_buffer() {
        let buffer = LineBuffer::new();
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
    }

    #[test]
    fn test_buffer_capacity() {
        let buffer = LineBuffer::with_capacity(1024);
        assert!(buffer.capacity() >= 1024);
    }
}
