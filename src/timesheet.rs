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
    Pause { time: u64 },
    Proceed { time: u64 },
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
        let seconds = get_seconds();
        Session {
            start: seconds,
            finish: seconds - 1,
            events: Vec::<Event>::new(),
        }
    }

    pub fn push_event(&mut self, e: Event) -> bool {
        /* TODO: add logic */
        match e {
            Event::Pause { time } => {
                assert!(time > self.start);
                let i = self.events.len();
                match i {
                    0 => false, // can't start a session with a pause
                    _ => {
                        self.events.push(e);
                        true
                    }
                }
            }

            Event::Proceed { .. } => {
                let mut i = self.events.len();
                let mut pushed = false;
                while i > 0 {
                    match self.events[i - 1] {
                        Event::Pause { .. } => {
                            self.events.push(e);
                            pushed = true;
                            break;
                        }
                        _ => i -= 1,
                    }
                }
                pushed
            }
            Event::Meta { .. } => {
                println!("pushing meta");
                self.events.push(e);
                true
            }
            Event::Commit { .. } => {
                self.events.push(e);
                true
            }
            Event::Branch { .. } => {
                self.events.push(e);
                true
            }
        }
    }

    pub fn finalize(&mut self) {
        self.finish = get_seconds();
        assert!(self.finish > self.start);
    }

    fn status(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timesheet {
    begin: u64,
    /* is this field necessary? */
    user: String,
    sessions: Vec<Session>,
}

impl Timesheet {
    pub fn new(name: &str) -> Timesheet {
        let now = SystemTime::now();
        let seconds = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        Timesheet {
            begin: seconds,
            user: name.to_string(),
            sessions: Vec::<Session>::new(),
        }
    }

    pub fn push_event(&mut self, e: Event) -> bool {
        let result = match self.get_last_session() {
            Some(s) => s.push_event(e),
            None => false,
        };
        self.save_to_file();
        result
    }
    pub fn get_last_session(&mut self) -> Option<&mut Session> {
        match self.sessions.len() {
            0 => None,
            n => Some(&mut self.sessions[n - 1]),
        }
    }

    pub fn new_session(&mut self) {
        /* TODO: assert valid session logic here */
        self.sessions.push(Session::new());
        self.save_to_file();
    }

    pub fn finalize_last(&mut self) {
        match self.get_last_session() {
            Some(s) => s.finalize(),
            None => println!("No session to finalize!"),
        }
        self.save_to_file();
    }

    pub fn last_status(&mut self) -> Option<String> {
        match self.get_last_session() {
            Some(s) => Some(s.status()),
            None => None,
        }
    }

    pub fn save_to_file(&self) -> bool {
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        /* TODO: make all commands run regardless of where trk is executed (and not just in root
         * which is assumed here */
        let path = Path::new("./.trk/sessions.trk");
        match fs::remove_file(&path) {
            Ok(..) => {}
            Err(..) => {}
        }

        let file = OpenOptions::new().write(true).create(true).open(&path);

        match file {
            Ok(mut file) => {
                /* Convert the sheet to a JSON string. */
                let serialized =
                    serde_json::to_string(&self).expect("Could not write serialized time sheet!");
                file.write_all(serialized.as_bytes()).unwrap();
                /* save was successful */
                true
            }
            Err(why) => {
                println!("{}", why.description());
                false
            }
        }
    }
}

/* Initializes the .trk/sessions.trk file which holds the serialized timesheet */
pub fn init(name: &str) -> bool {
    /* Check if file already exists(no init permitted) */
    if is_init() {
        println!("Already initialized!");
        false
    } else {
        /* file does not exist, initialize */
        let ts = Timesheet::new(name);
        ts.save_to_file()
    }
}

pub fn is_init() -> bool {
    if Path::new("./.trk/sessions.trk").exists() {
        match load_from_file() {
            Some(..) => true,
            None => false,
        }
    } else {
        false
    }
}

pub fn clear_sessions() {
    /* Try to get name */
    let temp_ts = load_from_file().unwrap();
    let name = temp_ts.user;

    let path = Path::new("./.trk/sessions.trk");
    fs::remove_file(&path).expect("Could not remove file!");
    init(&name);
}

/** Return an Some(Timesheet) struct if a sessions.trk file is present and valid
 * in the .trk directory, and None otherwise.
 * TODO: improve error handling
 * */

pub fn load_from_file() -> Option<Timesheet> {
    let path = Path::new("./.trk/sessions.trk");
    let file = OpenOptions::new().read(true).open(&path);
    match file {
        Ok(mut f) => {
            let mut serialized = String::new();
            match f.read_to_string(&mut serialized) {
                Ok(..) => serde_json::from_str(&serialized).unwrap_or(None),
                Err(..) => {
                    println!("Reading the string failed!");
                    None
                }
            }
        }
        Err(..) => None,
    }
}

pub fn get_seconds() -> u64 {
    let now = SystemTime::now();
    now.duration_since(UNIX_EPOCH).unwrap().as_secs()
}
