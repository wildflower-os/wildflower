use super::timer;

use crate::api::fs::{FileIO, IO};

use alloc::format;

#[derive(Debug, Clone)]
pub struct BootTime;

impl BootTime {
    pub fn new() -> Self {
        Self {}
    }

    pub fn size() -> usize {
        // Must be at least 20 + 1 + 6 bytes: "<seconds>.<nanoseconds>"
        32
    }
}

impl FileIO for BootTime {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, ()> {
        let time = format!("{:.6}", boot_time());
        let n = time.len();
        if buf.len() >= n {
            buf[0..n].clone_from_slice(time.as_bytes());
            Ok(n)
        } else {
            Err(())
        }
    }

    fn write(&mut self, _buf: &[u8]) -> Result<usize, ()> {
        Err(())
    }

    fn close(&mut self) {}

    fn poll(&mut self, event: IO) -> bool {
        match event {
            IO::Read => true,
            IO::Write => false,
        }
    }
}

/// Returns the number of seconds since boot.
///
/// This clock is monotonic.
pub fn boot_time() -> f64 {
    timer::time_between_ticks() * timer::ticks() as f64
}

#[test_case]
fn test_boot_time() {
    assert!(boot_time() > 0.0);
}
