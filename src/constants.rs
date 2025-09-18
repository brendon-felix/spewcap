use std::time::Duration;

pub const DEFAULT_BAUD_RATE: u32 = 115200;

pub const DEFAULT_LINE_BUFFER_CAPACITY: usize = 8192;
pub const STDOUT_BUFFER_CAPACITY: usize = 1024;
pub const SERIAL_READ_BUFFER_SIZE: usize = 2048;
pub const LOG_WRITER_BUFFER_CAPACITY: usize = 8192;
pub const LOG_LINE_BUFFER_INITIAL_CAPACITY: usize = 512;
pub const TIMESTAMP_BUFFER_INITIAL_CAPACITY: usize = 32;
pub const LINE_BUFFER_SHRINK_THRESHOLD: usize = 2048;
pub const LINE_BUFFER_SHRINK_TARGET: usize = 512;
pub const TIMESTAMP_BUFFER_SHRINK_THRESHOLD: usize = 128;
pub const TIMESTAMP_BUFFER_SHRINK_TARGET: usize = 32;

pub const COMMAND_POLL_PERIOD_MS: u64 = 100;
pub const SERIAL_READ_TIMEOUT_MS: u64 = 100;
pub const SERIAL_NO_DATA_SLEEP_MS: u64 = 10;
pub const SERIAL_RETRY_DELAY_MS: u64 = 500;
pub const SIGNAL_MONITOR_SLEEP_MS: u64 = 100;

pub const COMMAND_POLL_PERIOD: Duration = Duration::from_millis(COMMAND_POLL_PERIOD_MS);
pub const SERIAL_READ_TIMEOUT: Duration = Duration::from_millis(SERIAL_READ_TIMEOUT_MS);
pub const SERIAL_NO_DATA_SLEEP: Duration = Duration::from_millis(SERIAL_NO_DATA_SLEEP_MS);
pub const SERIAL_RETRY_DELAY: Duration = Duration::from_millis(SERIAL_RETRY_DELAY_MS);
pub const SIGNAL_MONITOR_SLEEP: Duration = Duration::from_millis(SIGNAL_MONITOR_SLEEP_MS);

pub const HIGH_THROUGHPUT_YIELD_THRESHOLD: usize = 100;
pub const LOG_FLUSH_INTERVAL: usize = 10;

pub const MILLIS_PER_HOUR: u128 = 3_600_000;
pub const MILLIS_PER_MINUTE: u128 = 60_000;
pub const MILLIS_PER_SECOND: u128 = 1_000;
