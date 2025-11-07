use std::time::{SystemTime, UNIX_EPOCH};

/// Get current Unix timestamp in milliseconds
pub fn current_timestamp() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_millis() as u32
}

/// Get current Unix timestamp in seconds
pub fn current_timestamp_secs() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System time before UNIX epoch")
        .as_secs() as u32
}

/// Calculate time delta in milliseconds
pub fn time_delta_ms(start: u32, end: u32) -> i32 {
    (end as i32) - (start as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_timestamp_increasing() {
        let ts1 = current_timestamp();
        thread::sleep(Duration::from_millis(10));
        let ts2 = current_timestamp();
        assert!(ts2 > ts1);
    }

    #[test]
    fn test_time_delta() {
        let start = 1000u32;
        let end = 1500u32;
        assert_eq!(time_delta_ms(start, end), 500);

        // Test negative delta
        assert_eq!(time_delta_ms(end, start), -500);
    }
}