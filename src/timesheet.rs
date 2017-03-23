extern crate time;
extern crate serde_json;

use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Begin,
    End,
    Pause,
    Proceed,
    Meta { text: String },
    Commit { hash: u64 },
    Branch { name: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub id: u64,
    /* is this field necessary? At least un-hardcode*/
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
        /* TODO: add logic */
        match e {
            Event::Begin => self.events.push(e),
            Event::End => self.events.push(e),
            Event::Pause => self.events.push(e),
            Event::Proceed => self.events.push(e),
            Event::Meta { .. } => self.events.push(e),
            Event::Commit { .. } => self.events.push(e),
            Event::Branch { .. } => self.events.push(e),
        }
    }

    /* TODO return formatted string instead */
    pub fn status(&self) {
        println!("{:?}", self.events);
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct Timesheet {
    id: u64,
    /* is this field necessary? At least un-hardcode*/
    user: String,
    sessions: Vec<Session>,
}

impl Timesheet {
    pub fn new() -> Timesheet {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Timesheet {
            id: seconds,
            user: "Rafael".to_string(),
            sessions: Vec::<Session>::new(),
        }
    }

    pub fn push_session(&mut self, s: Session) {
        /* TODO: check for valid session logic here */
        self.sessions.push(s);
    }
}

/* Initializes the .trk/sessions.trk file which holds the serialized timesheet */
pub fn init() -> bool {
    /* Check if file already exists(no init permitted) */
    if is_init() {
        false
    } else {
        /* file does not exist, do an init */
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        let path = Path::new("./.trk/sessions.trk");
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path);

        match file {
            Ok(mut file) => {
                let sheet = Timesheet::new();
                /* Convert the sheet to a JSON string. */
                let serialized =
                    serde_json::to_string(&sheet).expect("Could not write serialized time sheet!");
                file.write_all(serialized.as_bytes()).unwrap();
            }
            Err(why) => println!("{}", why.description()),
        }
        /* init was successful */
        true
    }
}

pub fn is_init() -> bool {
    Path::new("./.trk/sessions.trk").exists()
}
