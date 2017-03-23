extern crate time;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: u64,
    // is this field necessary?
    pub user: String,
    events: Vec<Event>,
}

impl Session {
    pub fn new() -> Session {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Session {
            id: seconds,
            user: "Rafael".to_string(),
            events: Vec::<Event>::new(),
        }
    }
    pub fn push_event(&mut self, e: Event) {
        // TODO: add logic
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
    pub fn status(&self) { // return formatted string
        println!("{:?}", self.events);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Period {
    id: u64,
    // unused field is pointless
    user: String,
    sessions: Vec<Session>,
}

impl Period {
    pub fn new() -> Period {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Period {
            id: seconds,
            user: "Rafael".to_string(),
            sessions: Vec::<Session>::new(),
        }
    }

    pub fn push_session(&mut self, s: Session) {
        // Checking for valid session here
        self.sessions.push(s);
    }
}
