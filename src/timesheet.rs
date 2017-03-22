extern crate time;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum Event {
    BeginPeriod,
    EndPeriod,
    BeginSession,
    EndSession,
    Pause,
    Proceed,
    Meta { text: String },
    Commit { hash: u64 },
    Branch { name: String },
}

#[derive(Debug)]
pub struct Session {
    pub id: u64,
    pub user: &'static str,
    events: Vec<Event>,
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
    pub fn append_event(&mut self, e: Event) {
        match e {
            Event::BeginPeriod => self.events.push(e),
            Event::EndPeriod => self.events.push(e),
            Event::BeginSession => self.events.push(e),
            Event::EndSession => self.events.push(e),
            Event::Pause => self.events.push(e),
            Event::Proceed => self.events.push(e),
            Event::Meta { .. } => self.events.push(e),
            Event::Commit { .. } => self.events.push(e),
            Event::Branch { .. } => self.events.push(e),
        }
    }
}
