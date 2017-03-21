extern crate time;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub user: &'static str,
}

impl Session {
    pub fn new() -> Session {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Session {
            id: seconds,
            user: "Rafael",
        }
    }
}
