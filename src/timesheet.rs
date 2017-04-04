extern crate time;
extern crate serde_json;

use std::io::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

use std::fs;
use std::path::Path;
use std::error::Error;
use std::fs::OpenOptions;

use std::process::Command;

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Pause(u64),
    PauseMeta { time: u64, reason: String },
    Proceed(u64),
    Meta { time: u64, text: String },
    Commit { time: u64, hash: u64 },
    Branch { time: u64, name: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct Session {
    pub start: u64,
    pub end: u64,
    events: Vec<Event>,
}

impl Session {
    fn new() -> Session {
        let seconds = get_seconds();
        Session {
            start: seconds,
            end: seconds - 1,
            events: Vec::<Event>::new(),
        }
    }

    fn in_progress(&self) -> bool {
        self.start == (self.end + 1)
    }

    fn finalize(&mut self) {
        self.end = get_seconds();
    }

    fn status(&self) -> String {
        format!("{:?}", self)
    }

    fn is_paused(&self) -> bool {
        match self.events.len() {
            0 => false,
            n => {
                match self.events[n - 1] {
                    Event::Pause(..) |
                    Event::PauseMeta { .. } => true,
                    _ => false,
                }
            }
        }
    }

    fn push_event(&mut self, event: Event) -> bool {
        /* Cannot push if session is already finalized! */
        if !self.in_progress() {
            println!("Already finalized, cannot push event");
            return false;
        }
        /* TODO: add logic */
        match event {
            Event::Pause(..) |
            Event::PauseMeta { .. } => {
                if !self.is_paused() {
                    self.events.push(event);
                    true
                } else {
                    println!("Already paused!");
                    false
                }
            }
            Event::Proceed(..) => {
                if self.is_paused() {
                    self.events.push(event);
                    true
                } else {
                    println!("No pause in progress!");
                    false
                }
            }
            Event::Meta { .. } => {
                if self.is_paused() {
                    /* morph last pause into a pause_meta */
                } else {
                    self.events.push(event);
                };
                true
            }
            Event::Commit { .. } |
            Event::Branch { .. } => {
                if self.is_paused() {
                    let now = get_seconds();
                    self.push_event(Event::Proceed(now));
                }
                self.events.push(event);
                true
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timesheet {
    start: u64,
    end: u64,
    user: String,
    sessions: Vec<Session>,
}

impl Timesheet {
    /** Initializes the .trk/sessions.trk file which holds
     * the serialized timesheet
     * Returns Some(newTimesheet) if operation succeeded */
    pub fn init(author_name: Option<&str>) -> Option<Timesheet> {
        /* Check if file already exists (no init permitted) */
        if Timesheet::is_init() {
            None
        } else {
            /* File does not exist, initialize */
            let git_author_name = &git_author().unwrap_or("".to_string());
            let author_name = match author_name {
                Some(name) => name,
                None => git_author_name,
            };
            let sheet = Timesheet {
                start: get_seconds(),
                end: get_seconds() - 1,
                user: author_name.to_string(),
                sessions: Vec::<Session>::new(),
            };
            if sheet.save_to_file() {
                Some(sheet)
            } else {
                None
            }
        }
    }

    fn is_init() -> bool {
        if Path::new("./.trk/sessions.trk").exists() {
            match Timesheet::load_from_file() {
                Some(..) => true,
                /* else, loading failed */
                None => false,
            }
        } else {
            /* File doesn't even exist */
            false
        }
    }

    fn get_last_session(&mut self) -> Option<&mut Session> {
        match self.sessions.len() {
            0 => None,
            n => Some(&mut self.sessions[n - 1]),
        }
    }

    pub fn new_session(&mut self) -> bool {


        let nsessions = self.sessions.len();
        let pushed = match nsessions {
            0 => true,
            _ => {
                if !self.sessions[nsessions - 1].in_progress() {
                    true
                } else {
                    println!("Last session is still running!");
                    false
                }
            }
        };
        if pushed {
            self.sessions.push(Session::new());
            self.save_to_file();
        }
        pushed
    }

    pub fn end_session(&mut self) {
        match self.get_last_session() {
            Some(session) => session.finalize(),
            None => println!("No session to finalize!"),
        }
        self.save_to_file();
    }

    pub fn pause(&mut self) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Pause(now));
            }
            None => println!("No session to pause!"),
        }
        self.save_to_file();
    }

    pub fn metapause(&mut self, reason: &str) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::PauseMeta {
                                       time: now,
                                       reason: reason.to_string(),
                                   });
            }
            None => println!("No session to pause!"),
        }
        self.save_to_file();
    }

    pub fn proceed(&mut self) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Proceed(now));
            }
            None => println!("No session to pause!"),
        }
        self.save_to_file();
    }

    pub fn push_meta(&mut self, metatext: String) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Meta {
                                       time: now,
                                       text: metatext,
                                   });
            }
            None => println!("No session to add meta to!"),
        }
        self.save_to_file();
    }

    pub fn push_commit(&mut self, hash: u64) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Commit {
                                       time: now,
                                       hash: hash,
                                   });
            }
            None => println!("No session to add commit to!"),
        }
        self.save_to_file();
    }

    pub fn push_branch(&mut self, name: String) {
        match self.get_last_session() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Branch {
                                       time: now,
                                       name: name,
                                   });
            }
            None => println!("No session to change branch in!"),
        }
        self.save_to_file();
    }

    fn save_to_file(&self) -> bool {
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        /* TODO: make all commands run regardless of where trk is executed
         * (and not just in root which is assumed here */
        let path = Path::new("./.trk/sessions.trk");
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path);

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

    /** Return a Some(Timesheet) struct if a sessions.trk file is present and valid
     * in the .trk directory, and None otherwise.
     * TODO: improve error handling
     * */
    pub fn load_from_file() -> Option<Timesheet> {
        let path = Path::new("./.trk/sessions.trk");
        let file = OpenOptions::new().read(true).open(&path);
        match file {
            Ok(mut file) => {
                let mut serialized = String::new();
                match file.read_to_string(&mut serialized) {
                    Ok(..) => serde_json::from_str(&serialized).unwrap_or(None),
                    Err(..) => {
                        println!("Reading the string failed!");
                        None
                    }
                }
            }
            Err(..) => {
                // println!("{}", why.description());
                None
            }
        }
    }

    pub fn clear_sessions() {
        /* Try to get name */
        let sheet = Timesheet::load_from_file();
        let name: Option<String> = sheet.map(|s| s.user.clone());

        let path = Path::new("./.trk/sessions.trk");
        if path.exists() {
            match fs::remove_file(&path) {
                Ok(..) => {}
                Err(why) => println!("Could not remove sessions file: {}", why.description()),
            }
        }
        match name {
            Some(name) => {
                /* Will overwrite file */
                Timesheet::init(Some(&name));
            }
            None => {
                Timesheet::init(None);
            }
        }
    }

    pub fn timesheet_status(&self) -> String {
        format!("{:?}", self)
    }

    pub fn last_session_status(&self) -> Option<String> {
        let nsessions = self.sessions.len();
        match nsessions {
            0 => None,
            n => Some(self.sessions[n - 1].status()),
        }
    }
}

fn get_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
}

fn git_author() -> Option<String> {
    if let Ok(output) = Command::new("git").arg("config").arg("user.name").output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            /* remove trailing newline character */
            let mut output = output.to_string();
            output.pop().expect("Empty name in git config!?!");
            Some(output)
        } else {
            let output = String::from_utf8_lossy(&output.stderr);
            println!("git config user.name failed! {}", output);
            None
        }
    } else {
        None
    }
}
