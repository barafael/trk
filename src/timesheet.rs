extern crate time;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
enum Event {
    WeekBegin,
    WeekEnd,
    SessionBegin,
    SessionEnd,
    Break,
    Continue,
    Meta { text: String },
    Commit { hash: u64 },
    Branch { text: String },
}

#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub user: &'static str,
    events: Vec<Event>
}

impl Session {
    pub fn new() -> Session {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Session {
            id: seconds,
            user: "Rafael",
            events: Vec::<Event>::new(),
        }
    }
}
