extern crate serde_json;
extern crate url;

use std::io::prelude::*;

use std::time::{SystemTime, UNIX_EPOCH};

use std::process;

use std::env;

use self::url::Url;

extern crate url_open;
use self::url_open::UrlOpen;

use chrono::{Local, TimeZone};

use std::fs;
use std::path::Path;
use std::error::Error;
use std::fs::OpenOptions;

use std::collections::HashSet;

/* For running git and tidy */
use std::process::Command;

/* Alias to avoid naming conflict for write_all!() */
use std::fmt::Write as std_write;

#[derive(Serialize, Deserialize, Debug)]
enum EventType {
    Pause,
    Resume,
    Note,
    Commit { hash: String },
}


#[derive(Serialize, Deserialize, Debug)]
struct Event {
    time    : u64,
    note    : Option<String>,
    ev_type : EventType
}

#[derive(Serialize, Deserialize, Debug)]
struct Session {
    start    : u64,
    end      : u64,
    running  : bool,
    branches : HashSet<String>,
    events   : Vec<Event>,
}

impl Session {
    fn new(timestamp: Option<u64>) -> Session {
        match timestamp {
            Some(timestamp) =>
                Session {
                    start    : timestamp,
                    end      : timestamp + 1,
                    running  : true,
                    branches : HashSet::<String>::new(),
                    events   : Vec::<Event>::new(),
                },
            None => {
                let now = get_seconds();
                Session {
                    start    : now,
                    end      : now + 1,
                    running  : true,
                    branches : HashSet::<String>::new(),
                    events   : Vec::<Event>::new(),
                }
            }
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    fn is_paused(&self) -> bool {
        match self.events.len() {
            0 => false,
            n => {
                match self.events[n - 1].ev_type {
                    EventType::Pause => true,
                    _ => false,
                }
            }
        }
    }

    fn update_end(&mut self) {
        self.end = match self.events.len() {
            0 => self.end,
            n => {
                let event = &self.events[n - 1];
                event.time + 1
            }
        }
    }

    fn finalize(&mut self, timestamp: Option<u64>) {
        let timestamp = match timestamp {
            None => get_seconds(),
            Some(timestamp) => {
                let ts_ok = match self.events.len() {
                    0 => timestamp > self.start,
                    n => {
                        let last = &self.events[n - 1];
                        timestamp > last.time
                    }
                };
                if ts_ok {
                    timestamp
                } else {
                    println!("That is not a valid timestamp!");
                    process::exit(0);
                }
            }
        };
        if self.is_running() {
            if self.is_paused() {
                self.push_event(Some(timestamp), None, EventType::Resume);
            }
            self.running = false;
            self.end = timestamp + 1;
        }
    }

    fn push_event(&mut self,
                  timestamp_opt : Option<u64>,
                  note_opt      : Option<String>,
                  type_of_event : EventType)
            -> bool {
        /* Cannot push if session is already finalized. */
        if !self.is_running() {
            println!("Already finalized, cannot push event.");
            return false;
        }
        let timestamp_opt = match timestamp_opt {
            None => {
                self.update_end();
                get_seconds()
            }
            Some(ts) => {
                let valid_ts = match self.events.len() {
                    0 => if ts > self.start { true } else { false },
                    n => {
                        match self.events[n - 1].time {
                            last_time if last_time < ts => true,
                            _ => false,
                        }
                    }
                };
                if valid_ts {
                    self.end = ts + 1;
                    ts
                } else {
                    println!("That timestamp is before the last event.");
                    return false;
                }
            }
        };
        /* TODO: improve logic */
        /* TODO: binding names */
        /* TODO: can the event struct be initialized just once?
         * Set the type only... */
        match type_of_event {
            // TODO: fix this, so both note and ago work...
            EventType::Pause => {
                if self.is_paused() {
                    println!("Already paused.");
                    false
                } else {
                    self.events
                        .push(Event {
                                  time    : timestamp_opt,
                                  note    : note_opt,
                                  ev_type : EventType::Pause,
                              });
                    true
                }
            }
            EventType::Resume => {
                if self.is_paused() {
                    self.events
                        .push(Event {
                                  time    : timestamp_opt,
                                  note    : note_opt,
                                  ev_type : EventType::Resume,
                              });
                    true
                } else {
                    println!("Currently not paused.");
                    false
                }
            }
            EventType::Note => {
                if self.is_paused() {
                    /* Add note.text to previous pause (last of events vec) */
                    /* If self.is_paused(), then self.len() is always at least 1 */
                    let len = self.events.len();
                    let pause = &mut self.events[len - 1];
                    match pause.note {
                        Some(ref mut already) => {
                            // TODO: handle long strings (also in other types)
                            // TODO: there must be another way other than <br>
                            already.push_str("<br>");
                            already.push_str(&note_opt.unwrap());
                        }
                        None => pause.note = note_opt,
                    }
                } else {
                    self.events
                        .push(Event {
                                  time    : timestamp_opt,
                                  note    : note_opt,
                                  ev_type : EventType::Note,
                              })
                };
                true
            }
            /* For now, allow commit adding only in 'real time' */
            EventType::Commit { hash } => {
                if self.is_paused() {
                    self.push_event(None, None, EventType::Resume);
                }
                /* Commit message must be provided */
                if note_opt.is_none() {
                    println!("No commit message found for commit {}.", hash);
                }
                self.events
                    .push(Event {
                              time    : get_seconds(),
                              note    : note_opt,
                              ev_type : EventType::Commit { hash },
                          });
                true
            }
        }
    }

    pub fn pause_time(&self) -> u64 {
        let mut pause_time = 0;
        let mut last_pause_ts = 0;
        for event in &self.events {
            match event.ev_type {
                EventType::Pause => last_pause_ts = event.time,
                EventType::Resume => pause_time += event.time - last_pause_ts,
                _ => {}
            }
        }
        pause_time
    }

    pub fn working_time(&self) -> u64 {
        let pause_time = self.pause_time();
        self.end - self.start - pause_time
    }

    fn add_branch(&mut self, name: String) {
        if self.is_running() {
            self.branches.insert(name);
        }
    }

    fn status(&self) -> String {
        let mut status = String::new();
        write!(&mut status,
r#"Session running since {}.
"#,
            sec_to_hms_string(get_seconds() - self.start)).unwrap();
        if self.is_paused() {
            write!(&mut status,
r#"Session is paused since {}.
"#,
    sec_to_hms_string(get_seconds() - self.events[self.events.len() - 1]
                      .time)).unwrap();
        } else {
            let last = match self.events.len() {
                0 => "No events in this session yet!\n".to_string(),
                n => format!("Last event: {:?}, {} ago.\n",
                             self.events[n - 1].ev_type,
                             sec_to_hms_string(
                                 get_seconds() - self.events[n - 1].time))
                };
            write!(&mut status, "{}", last).unwrap();
        }
        let mut branch_str = String::new();
        for branch in &self.branches {
            branch_str.push_str(branch);
            branch_str.push_str(" ");
        }
        write!(&mut status, "Worked on branches: {}", branch_str).unwrap();
        status
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Timesheet {
    start    : u64,
    end      : u64,
    user     : String,
    sessions : Vec<Session>,
}

impl Timesheet {
    /** Initializes the .trk/timesheet.json file which holds
     * the serialized timesheet
     * Returns Some(newTimesheet) if operation succeeded */
    pub fn init(author_name: Option<&str>) -> Option<Timesheet> {
        /* Check if file already exists (no init permitted) */
        if Timesheet::is_init() {
            println!("Timesheet is already initialized!");
            None
        } else {
            /* File does not exist, initialize */
            let git_author_name = &git_author().unwrap_or("".to_string());
            let author_name = match author_name {
                Some(name) => name,
                None => {
                    if git_author_name == "" {
                        println!("Empty name not permitted.
    Please run with 'trk init <name>'");
                        process::exit(0);
                    }
                    git_author_name
                }
            };
            let now = get_seconds();
            let sheet = Timesheet {
                start    : now,
                end      : now + 1,
                user     : author_name.to_string(),
                sessions : Vec::<Session>::new(),
            };
            if sheet.write_files() {
                Some(sheet)
            } else {
                None
            }
        }
    }

    fn is_init() -> bool {
        if Path::new("./.trk/timesheet.json").exists() {
            match Timesheet::load_from_file() {
                Some(..) => true,
                /* Else, loading failed */
                None => false,
            }
        } else {
            /* File doesn't even exist */
            false
        }
    }

    pub fn new_session(&mut self, timestamp: Option<u64>) -> bool {
        let possible = match self.get_last_session_mut() {
            None => true,
            Some(session) => {
                if session.is_running() {
                    println!("Last session is still running.");
                    false
                } else {
                    true
                }
            }
        };
        if possible {
            match timestamp {
                Some(timestamp) => {
                    let timestamp_ok =
                        match self.get_last_session() {
                            None => timestamp > self.start,
                            Some(last_session) =>
                                timestamp > last_session.end,
                    };
                    if timestamp_ok {
                        self.sessions
                            .push(Session::new(Some(timestamp)));
                    } else {
                        println!("That timestamp is invalid.");
                        process::exit(0);
                    }
                }
                None => {
                    self.sessions.push(Session::new(None));
                }
            };

            self.write_files();
        }
        possible
    }

    pub fn end_session(&mut self, timestamp: Option<u64>) {
        match self.get_last_session_mut() {
            Some(session) => {
                // TODO: should it be possible to end a session multiple times?
                // Each time sets the end date later...
                session.update_end();
                session.finalize(timestamp);
            }
            None => println!("No session to finalize."),
        }
        self.write_files();
    }

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

    pub fn pause(&mut self, timestamp_opt: Option<u64>, note_opt: Option<String>) {
        match self.get_last_session_mut() {
            Some(session) => {
                session.push_event(timestamp_opt, note_opt, EventType::Pause);
            }
            None => println!("No session to pause."),
        }
        self.write_files();
    }

    pub fn resume(&mut self, timestamp_opt: Option<u64>) {
        match self.get_last_session_mut() {
            Some(session) => {
                session.push_event(timestamp_opt, None, EventType::Resume);
            }
            None => println!("No session to pause."),
        }
        self.write_files();
    }

    pub fn note(&mut self, timestamp_opt: Option<u64>, note_text: String) {
        match self.get_last_session_mut() {
            Some(session) => {
                session.push_event(timestamp_opt, Some(note_text), EventType::Note);
            }
            None => println!("No session to add note to."),
        }
        self.write_files();
    }

    pub fn commit(&mut self, hash: String) {
        let new_needed = match self.get_last_session() {
            Some(session) => !session.is_running(),
            None => true,
        };
        if new_needed {
            self.new_session(None);
            self.write_files();
        }
        match self.get_last_session_mut() {
            Some(session) => {
                let message = git_commit_message(&hash)
                    .unwrap_or(String::new());
                session.push_event(
                    None,
                    Some(message),
                    EventType::Commit { hash });
            }
            None => println!("No session to add commit to."),
        }
        self.write_files();
    }

    pub fn branch(&mut self, name: String) {
        match self.get_last_session_mut() {
            Some(session) => {
                session.add_branch(name);
            }
            None => {},
        }
        self.write_files();
    }

    pub fn write_to_html(&self) -> bool {
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
                format_file("timesheet.html");
                /* Save was successful */
                true
            }
            Err(why) => {
                println!("Could not report sheet! {}", why.description());
                false
            }
        }
    }

    pub fn write_last_session_html(&self) -> bool {
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
                        let html = format!(
r#"<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" type="text/css" href="style.css">
  <title>{} for {}</title>
</head>
<body>
{}
</body>
</html>"#,
            "Session",
            "Rafael Bachmann",
            session.to_html());
                        file.write_all(html.as_bytes()).unwrap();
                        format_file("session.html");
                        /* Save was successful */
                        true
                    }
                    /* TODO: write empty file anyway? */
                    None => true,
                }
            }
            Err(why) => {
                println!("Could not write report to session.html! {}",
                         why.description());
                false
            }
        }
    }

    fn write_to_json(&self) -> bool {
        if !Path::new("./.trk").exists() {
            match fs::create_dir("./.trk") {
                Ok(_) => {}
                _ => {
                    println!("Could not create .trk directory.");
                    process::exit(0);
                }
            }
        }

        let path = Path::new("./.trk/timesheet.json");
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path);

        match file {
            Ok(mut file) => {
                /* Convert the sheet to a JSON string. */
                let serialized =
                    serde_json::to_string(&self)
                        .expect("Could not write serialized time sheet.");
                file.write_all(serialized.as_bytes()).unwrap();
                /* Save was successful */
                true
            }
            Err(why) => {
                println!("Could not open timesheet.json file: {}",
                         why.description());
                false
            }
        }
    }

    fn write_files(&self) -> bool {
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        /* TODO: make all commands run regardless of where trk is executed
         * (and not just in root which is assumed here */
        self.write_to_json() && self.write_to_html() && self.write_last_session_html()
    }

    /** Return a Some(Timesheet) struct if a timesheet.json file
     * is present and valid in the .trk directory, and None otherwise.
     * TODO: improve error handling
     * */
    pub fn load_from_file() -> Option<Timesheet> {
        let path = Path::new("./.trk/timesheet.json");
        let file = OpenOptions::new().read(true).open(&path);
        match file {
            Ok(mut file) => {
                let mut serialized = String::new();
                match file.read_to_string(&mut serialized) {
                    Ok(..) => serde_json::from_str(&serialized)
                        .unwrap_or(None),
                    Err(..) => {
                        println!("IO error while reading the timesheet file.");
                        process::exit(0);
                    }
                }
            }
            Err(..) => None,
        }
    }

    /* TODO: Check why timestamps aren't overwritten */
    pub fn clear() {
        /* Try to get user name */
        let sheet = Timesheet::load_from_file();
        let name: Option<String> = sheet.map(|s| s.user.clone());

        let path = Path::new("./.trk/timesheet.json");
        if path.exists() {
            match fs::remove_file(&path) {
                Ok(..) => {}
                Err(why) => println!("Could not remove sessions file: {}",
                                     why.description()),
            }
        }
        match name {
            Some(name) => {
                /* Overwrite file */
                Timesheet::init(Some(&name));
            }
            None => {
                Timesheet::init(None);
            }
        }
    }

    pub fn timesheet_status(&self) -> String {
        let mut status = String::new();
        write!(&mut status, "Sheet started on {}\n",
              sec_to_hms_string(get_seconds()
                                - self.start)).unwrap();
        match self.sessions.len() {
            0 => write!(&mut status, "No sessions yet.\n").unwrap(),
            n => write!(&mut status, "{} sessions so far.\nLast session:\n{}",
                        n,
                        self.sessions[n - 1].status()).unwrap(),
        };
        status
    }

    pub fn last_session_status(&self) -> String {
        let status = self.get_last_session()
            .map(|session| session.status());
        match status {
            None => "No session yet.".to_string(),
            Some(status) => status,
        }
    }

    pub fn report_last_session(&self) {
        // We assume that we are in a valid directory.
        let mut p = env::current_dir().unwrap();
        p.push("session.html");
        let path = p.as_path();
        let mut path_str = "file://".to_string();
        path_str.push_str(path.to_str().unwrap());
        Url::parse(&path_str).unwrap().open();
    }

    pub fn report_sheet(&self) {
        // We assume that we are in a valid directory.
        let mut p = env::current_dir().unwrap();
        p.push("timesheet.html");
        let path = p.as_path();
        let mut path_str = "file://".to_string();
        path_str.push_str(path.to_str().unwrap());
        Url::parse(&path_str).unwrap().open();
   }

   pub fn pause_time(&self) -> u64 {
        let mut pause_time = 0;
        for session in &self.sessions {
            pause_time += session.pause_time();
        }
        pause_time
    }

    pub fn working_time(&self) -> u64 {
        let mut work_time = 0;
        for session in &self.sessions {
            work_time += session.working_time();
        }
        work_time
    }
}

trait HasTEX {
    fn to_tex(&self) -> String;
}

trait HasHTML {
    fn to_html(&self) -> String;
}

impl HasHTML for Event {
    fn to_html(&self) -> String {
        match self.ev_type {
            EventType::Pause => {
                match self.note {
                    Some(ref info) => {
                        format!(
r#"<div class="entry pause">{}: Started a pause
    <p class="pausenote">{}</p>
</div>"#,
            ts_to_date(self.time),
            info.clone())
                    }
                    None => {
                        format!(
r#"<div class="entry pause">{}: Started a pause
</div>"#,
            ts_to_date(self.time))
                    }
                }
            }
            EventType::Resume => {
                format!(
r#"<div class="entry resume">{}: Resumed work
</div>"#,
            ts_to_date(self.time))
            }
            /* An EventType::Note note is a Some because it's
             * 'constructor' function takes a String
             * (and not Option<String>)
             */
            EventType::Note => {
                match self.note {
                    Some(ref text) => {
                        format!(
r#"<div class="entry note">{}: Note: {
}</div>"#,
            ts_to_date(self.time),
            text)
                    }
                    None => unreachable!(),
                }
            }
            /* It is safe to unwrap an EventType::Commit note because if
             * a commit has no message something went really wrong
             * (like parsing the output of `git log` in git_commit_message()
             */
            EventType::Commit { ref hash } => {
                match self.note {
                    Some(ref text) => format!(
r#"<div class="entry commit">{}: Commit id: {}
  <br>    message: {}
</div>"#,
            ts_to_date(self.time),
            hash,
            text),
                    None => unreachable!(),
                }
            }
        }
    }
}

impl HasHTML for Session {
    fn to_html(&self) -> String {
        let mut html = format!(
r#"<section class="session">
    <h1 class="sessionheader">Session on {}</h1>"#,
                               ts_to_date(self.start));

        for event in &self.events {
            write!(&mut html, "{}", event.to_html()
                   ).unwrap();
        }

        write!(&mut html,
r#"<h2 class="sessionfooter">Ended on {}</h2>"#,
           ts_to_date(self.end)
        ).unwrap();

        let mut branch_str = String::new();
        for branch in &self.branches {
            branch_str.push_str(branch);
            branch_str.push_str(" ");
        }
        write!(&mut html,
r#"<section class="summary">
    <p>Worked on branches: {}</p>
    <p>Worked for {} </p>
    <p>Paused for {}</p>
</div></section>"#,
            branch_str,
            sec_to_hms_string(self.working_time()),
            sec_to_hms_string(self.pause_time())
        ).unwrap();

        write!(&mut html, "</section>").unwrap();
        html
    }
}

impl HasHTML for Timesheet {
    fn to_html(&self) -> String {
        let mut sessions_html = String::new();
        for session in &self.sessions {
            write!(&mut sessions_html, "{}", session.to_html()
                   ).unwrap();
        }
        let mut html = format!(
r#"<!DOCTYPE html>
<html>
    <head>
        <link rel="stylesheet" type="text/css" href="style.css">
        <title>{} for {}</title>
    </head>
    <body>
    {}"#,
            "Timesheet",
            "Rafael Bachmann",
            sessions_html);

        write!(&mut html,
r#"<section class="summary">
    <p>Worked for {} </p>
    <p>Paused for {}</p>
</div></section>"#,
           sec_to_hms_string(self.working_time()),
           sec_to_hms_string(self.pause_time())
        ).unwrap();
        write!(&mut html, "</body>\n</html>").unwrap();
        html
    }
}

pub fn get_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn git_author() -> Option<String> {
    if let Ok(output) = Command::new("git")
           .arg("config")
           .arg("user.name")
           .output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            /* Remove trailing newline character */
            let mut output = output.to_string();
            output.pop().expect("Empty name in git config? Not even a newline?!?");
            Some(output)
        } else {
            let output = String::from_utf8_lossy(&output.stderr);
            println!("git config user.name failed. {}", output);
            None
        }
    } else {
        None
    }
}

fn git_commit_message(hash: &str) -> Option<String> {
    if let Ok(output) = Command::new("git")
           .arg("log")
           .arg("--format=%B")
           .arg("-n")
           .arg("1")
           .arg(hash)
           .output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            Some(output.to_string())
        } else {
            let output = String::from_utf8_lossy(&output.stderr);
            println!("git log --format=%B -n 1 <hash> failed. {}", output);
            None
        }
    } else {
        None
    }
}

fn format_file(filename: &str) {
    if let Ok(_) = Command::new("tidy")
           .arg("--tidy-mark")
           .arg("no")
           .arg("-i")
           .arg("-m")
           .arg(filename)
           .output() {}
    else {
        println!("tidy-html not found!");
    }
}

pub fn ts_to_date(timestamp: u64) -> String {
    Local
        .timestamp(timestamp as i64, 0)
        .format("%Y-%m-%d, %H:%M")
        .to_string()
}

pub fn sec_to_hms_string(seconds: u64) -> String {
    let hours   = seconds / 3600;
    let minutes = (seconds - hours * 3600) / 60;
    let seconds = seconds - minutes * 60 - hours * 3600;
    match (hours, minutes, seconds) {
        (0, 0, 1)       => format!("1 second"),
        (0, 0, s)       => format!("{} seconds", s),
        (0, 1, _)       => format!("1 minute"),
        (0, m, _)       => format!("{} minutes", m),
        /* Range matching: slightly dubious feature here */
        (1, 0...4, _)   => format!("1 hour"),
        (h, 0...4, _)   => format!("{} hours", h),
        (h, 56...59, _) => format!("{} hours", h + 1),
        (h, m, _)       => format!("{} hours and {} minutes", h, m),
    }
}
