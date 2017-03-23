extern crate time;
extern crate serde_json;

use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use std::fs;
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
    pub start: u64,
    pub finish: u64,
    events: Vec<Event>,
}

impl Session {
    pub fn new() -> Session {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Session {
            start: seconds,
            finish: seconds-1,
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

    pub fn finalize(&mut self) {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.finish = seconds;
    }
    pub fn status(&self) -> String {
        format!("{:?}", self.events)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timesheet {
    id: u64,
    /* is this field necessary? At least un-hardcode*/
    user: String,
    sessions: Vec<Session>,
}

impl Timesheet {
    pub fn new(name: &str) -> Timesheet {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Timesheet {
            id: seconds,
            user: name.to_string(),
            sessions: Vec::<Session>::new(),
        }
    }

    pub fn get_last_session(&self) -> Option<&Session> {
        match self.sessions.len() {
            0 => None,
            n => Some(&self.sessions[n-1])
        }
    }

    pub fn push_session(&mut self, s: Session) {
        /* TODO: check for valid session logic here */
        self.sessions.push(s);
    }
}

/* Initializes the .trk/sessions.trk file which holds the serialized timesheet */
pub fn init(name: &str) -> bool {
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
                /* leave an empty timesheet, not an empty file */
                let sheet = Timesheet::new(name);
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

pub fn clear_sessions() {
    let temp_ts = load_from_file().unwrap();
    let name = temp_ts.user;
    let path = Path::new("./.trk/sessions.trk");
    fs::remove_file(&path).expect("Could not remove file!");
    init(&name);
}

pub fn load_from_file() -> Option<Timesheet> {
    let path = Path::new("./.trk/sessions.trk");
    let file = OpenOptions::new().read(true).create(true).open(&path);
    match file {
        Ok(mut f) => {
            let mut serialized = String::new();
            f.read_to_string(&mut serialized).unwrap();
            let deserialized: Timesheet = serde_json::from_str(&serialized).unwrap();
            Some(deserialized)
        }
        Err(_) => None,
    }
}
