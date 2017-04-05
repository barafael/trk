extern crate time;
extern crate serde_json;

use std::io::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{Local, TimeZone};

use std::fs;
use std::path::Path;
use std::error::Error;
use std::fs::OpenOptions;

use std::process::Command;

use std::fmt::Write as strwrite;

trait HasTEX {
    fn to_tex(&self) -> String;
}

trait HasHTML {
    fn to_html(&self) -> String;
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Event {
    Pause { time: u64 },
    MetaPause { time: u64, meta_info: String },
    Resume { time: u64 },
    Meta { time: u64, text: String },
    Commit { time: u64, hash: u64 },
    Branch { time: u64, name: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct Session {
    start: u64,
    end: u64,
    running: bool,
    events: Vec<Event>,
}

impl Session {
    fn new() -> Session {
        let seconds = get_seconds();
        Session {
            start: seconds,
            end: seconds - 1,
            running: true,
            events: Vec::<Event>::new(),
        }
    }

    fn update_end(&mut self) {
        self.end = get_seconds();
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn finalize(&mut self) {
        self.running = false;
    }

    fn status(&self) -> String {
        format!("{:?}", self)
    }

    fn is_paused(&self) -> bool {
        match self.events.len() {
            0 => false,
            n => {
                match self.events[n - 1] {
                    Event::Pause { .. } |
                    Event::MetaPause { .. } => true,
                    _ => false,
                }
            }
        }
    }

    fn push_event(&mut self, event: Event) -> bool {
        /* Cannot push if session is already finalized! */
        if !self.is_running() {
            println!("Already finalized, cannot push event");
            return false;
        }
        self.update_end();
        /* TODO: add logic */
        match event {
            Event::Pause { .. } |
            Event::MetaPause { .. } => {
                if !self.is_paused() {
                    self.events.push(event);
                    true
                } else {
                    println!("Already paused!");
                    false
                }
            }
            Event::Resume { .. } => {
                if self.is_paused() {
                    self.events.push(event);
                    true
                } else {
                    println!("Currently not paused!");
                    false
                }
            }
            Event::Meta { time: ref metatime, text: ref metatext } => {
                if self.is_paused() {
                    /* morph last pause into a MetaPause */
                    let pause = self.events.pop().unwrap();
                    match pause {
                        Event::Pause { time: pausetime } => {
                            self.push_event(Event::MetaPause {
                                                time: pausetime,
                                                meta_info: metatext.to_string(),
                                            })
                        }
                        Event::MetaPause { time: pausetime, meta_info } => {
                            let meta_info = meta_info + "\n" + metatext;
                            self.push_event(Event::MetaPause {
                                                time: pausetime,
                                                meta_info: meta_info.to_string(),
                                            })
                        }
                        _ => unreachable!(),
                    };
                } else {
                    self.events.push(Event::Meta {
                                         time: *metatime,
                                         text: metatext.clone(),
                                     })
                };
                true
            }
            Event::Commit { .. } |
            Event::Branch { .. } => {
                if self.is_paused() {
                    let now = get_seconds();
                    self.push_event(Event::Resume { time: now });
                }
                self.events.push(event);
                true
            }
        }
    }
}

impl HasHTML for Event {
    fn to_html(&self) -> String {
        match self {
            &Event::Pause { time } => {
                format!("<div class=\"entry pause\">{}:\tStarted a pause</div>",
                        ts_to_date(time))
            }
            &Event::MetaPause { time, ref meta_info } => {
                format!("<div class=\"entry metapause\">{}:\t{}</div>",
                        ts_to_date(time),
                        meta_info)
            }
            &Event::Resume { time } => {
                format!("<div class=\"entry resume\">{}:\tResumed work</div>",
                        ts_to_date(time))
            }
            &Event::Meta { time, ref text } => {
                format!("<div class=\"entry meta\">{}:\tNote: {}</div>",
                        ts_to_date(time),
                        text)
            }
            &Event::Commit { time, hash } => {
                format!("<div class=\"entry commit\">{}:\tCommit id: {}</div>",
                        ts_to_date(time),
                        hash)
            }
            &Event::Branch { time, ref name } => {
                format!("<div class=\"entry branch\">{}:\tBranch name: {}</div>",
                        ts_to_date(time),
                        name)
            }
        }
    }
}

impl HasHTML for Session {
    fn to_html(&self) -> String {
        let mut html = String::from(format!("<section class=\"session\">\n<h1 class=\"sessionheading\">Session on {}</h1>\n",
                                            ts_to_date(self.start)));
        for event in &self.events {
            write!(&mut html, "{}\n", event.to_html()).unwrap();
        }
        write!(&mut html, "</section>").unwrap();
        html
    }
}

impl HasHTML for Timesheet {
    fn to_html(&self) -> String {
        let mut html = String::from("<!DOCTYPE html>\n");
        write!(&mut html, "<html>\n<head>\n<link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\">\n</head>").unwrap();
        write!(&mut html, "<body>\n").unwrap();
        for session in &self.sessions {
            write!(&mut html, "{}\n", session.to_html()).unwrap();
        }
        write!(&mut html, "</body>\n</html>").unwrap();
        html
    }
}
// Maybe every event added to a session should set the end date? And begin just always opens a new
// session?
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
        // ts_to_date(get_seconds());
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

    // TODO: Check if this function could be used in more places
    fn get_last_session(&self) -> Option<&Session> {
        match self.sessions.len() {
            0 => None,
            n => Some(&self.sessions[n - 1]),
        }
    }

    fn get_last_session_mut(&mut self) -> Option<&mut Session> {
        match self.sessions.len() {
            0 => None,
            n => Some(&mut self.sessions[n - 1]),
        }
    }

    pub fn new_session(&mut self) -> bool {
        let n_sessions = self.sessions.len();
        let push = match n_sessions {
            0 => true,
            _ => {
                if self.sessions[n_sessions - 1].is_running() {
                    println!("Last session is still running!");
                    false
                } else {
                    true
                }
            }
        };
        if push {
            self.sessions.push(Session::new());
            self.save_to_file();
        }
        push
    }

    pub fn end_session(&mut self) {
        match self.get_last_session_mut() {
            Some(session) => {
                session.update_end();
                session.finalize();
            }
            None => println!("No session to finalize!"),
        }
        self.save_to_file();
    }

    pub fn pause(&mut self, metatext: Option<String>) {
        match self.get_last_session_mut() {
            Some(session) => {
                let now = get_seconds();
                match metatext {
                    Some(meta_info) => {
                        session.push_event(Event::MetaPause {
                                               time: now,
                                               meta_info: meta_info.to_string(),
                                           });
                    }
                    None => {
                        session.push_event(Event::Pause { time: now });
                    }
                }
            }
            None => println!("No session to pause!"),
        }
        self.save_to_file();
    }

    pub fn resume(&mut self) {
        match self.get_last_session_mut() {
            Some(session) => {
                let now = get_seconds();
                session.push_event(Event::Resume { time: now });
            }
            None => println!("No session to pause!"),
        }
        self.save_to_file();
    }

    pub fn push_meta(&mut self, metatext: String) {
        match self.get_last_session_mut() {
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
        match self.get_last_session_mut() {
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
        match self.get_last_session_mut() {
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

    pub fn report(&self) -> bool {
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        /* TODO: make all commands run regardless of where trk is executed
         * (and not just in root which is assumed here */

        let path = Path::new("./timesheet.html");
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path);

        match file {
            Ok(mut file) => {
                file.write_all(self.to_html().as_bytes()).unwrap();
                /* save was successful */
                true
            }
            Err(why) => {
                println!("{}", why.description());
                false
            }
        }
    }

    pub fn last_session_report(&self) -> bool {
        let path = Path::new("./session.html");
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path);

        match file {
            Ok(mut file) => {
                match self.get_last_session() {
                    Some(session) => {
                        file.write_all(session.to_html().as_bytes()).unwrap();
                        /* save was successful */
                        true
                    }
                    None => false,
                }
            }
            Err(why) => {
                println!("{}", why.description());
                false
            }
        }
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
        let n_sessions = self.sessions.len();
        match n_sessions {
            0 => None,
            n => {
                Some(self.sessions[n - 1].status())
            }
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

pub fn ts_to_date(timestamp: u64) -> String {
    Local.timestamp(timestamp as i64, 0).to_string()

}
