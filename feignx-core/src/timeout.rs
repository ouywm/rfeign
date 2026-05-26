use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Timeout {
    pub connect: Duration,
    pub read: Duration,
    pub write: Duration,
}

impl Default for Timeout {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(5),
            read: Duration::from_secs(30),
            write: Duration::from_secs(30),
        }
    }
}