use std::io::prelude::*;
use std::{process, env};
use std::path::Path;
use std::error::Error;
use std::fs::{self, OpenOptions};
/* Alias to avoid naming conflict for write_all!() */
use std::fmt::Write as std_write;

use url::Url;
use url_open::UrlOpen;

use serde_json::{from_str, to_string};

use util::*;
use config::*;
use timesheet::traits::HasHTML;

use timesheet::session::Session;
use timesheet::session::EventType;

#[derive(Serialize, Deserialize, Debug)]
pub struct Timesheet {
    start            : u64,
    end              : u64,
    config           : Config,
    sessions         : Vec<Session>,
}

impl Timesheet {
    /* TODO: check if all goes right if write_files
       is called only before the end of main() */
    /** Initializes the .trk/timesheet.json file which holds
     * the serialized timesheet
     * Returns Some(newTimesheet) if operation succeeded */
    pub fn init(author_name: Option<&str>) -> Option<Timesheet> {
        /* Check if file already exists (no init permitted) */
        if Timesheet::is_init() {
            println!("Timesheet is already initialized!");
            return None;
        }

        /* File does not exist, initialize */
        let git_author_name = git_author();
        let author_name = match author_name {
            Some(name) => name,
            None => {
                match git_author_name {
                    Some(ref git_name) => git_name,
                    None => {
                        println!("Empty name not permitted. \
                                  Please run with 'trk init <name>'");
                        process::exit(0);
                    }
                }
            }
        };
        let mut config = Config::new();
        config.user_name = Some(author_name.to_string());
        let now = get_seconds();
        let sheet = Timesheet {
            start        : now,
            end          : now + 1,
            config       : config,
            sessions     : Vec::<Session>::new(),
        };
        if sheet.write_files() {
            Some(sheet)
        } else {
            None
        }
    }

    fn is_init() -> bool {
        Path::new("./.trk/timesheet.json").exists() && Timesheet::load_from_file().is_some()
    }

    pub fn new_session(&mut self, timestamp: Option<u64>) -> bool {
        let possible = self.sessions.last_mut()
            .map_or(true, |session| {
                if session.is_running() {
                    println!("Last session is still running.");
                    false
                } else {
                    true
                }
        });
        if possible {
            match timestamp {
                Some(timestamp) => {
                    let is_valid_ts = match self.sessions.last() {
                        None => timestamp > self.start,
                        Some(last_session) => timestamp > last_session.end,
                    };
                    if is_valid_ts {
                        self.sessions.push(Session::new(Some(timestamp)));
                    } else {
                        println!("That timestamp is invalid.");
                        process::exit(0);
                    }
                }
                None => {
                    self.sessions.push(Session::new(None));
                }
            };
        }
        possible
    }

    pub fn end_session(&mut self, timestamp: Option<u64>) {
        match self.sessions.last_mut() {
            Some(session) => {
                // TODO This is always problematic - rethink.
                session.update_end();
                session.finalize(timestamp);
                self.end = session.end + 1;
            }
            None => println!("No session to finalize."),
        }
    }

    pub fn pause(&mut self, timestamp: Option<u64>, note: Option<String>) {
        match self.sessions.last_mut() {
            Some(session) => {
                session.push_event(timestamp, note, EventType::Pause);
            }
            None => println!("No session to pause."),
        }
    }

    pub fn resume(&mut self, timestamp: Option<u64>) {
        match self.sessions.last_mut() {
            Some(session) => {
                session.push_event(timestamp, None, EventType::Resume);
            }
            None => println!("No session to pause."),
        }
    }

    pub fn note(&mut self, timestamp: Option<u64>, note_text: String) {
        match self.sessions.last_mut() {
            Some(session) => {
                session.push_event(timestamp, Some(note_text), EventType::Note);
            }
            None => println!("No session to add note to."),
        }
    }

    pub fn add_commit(&mut self, hash: String) {
        let new_needed = self.sessions.last()
            .map_or(true, |session| !session.is_running());
        if new_needed {
            self.new_session(None);
        }
        match self.sessions.last_mut() {
            Some(session) => {
                let message = git_commit_message(&hash).unwrap_or(String::new());
                session.push_event(None, Some(message), EventType::Commit { hash });
            }
            None => println!("No session to add commit to."),
        }
    }

    pub fn add_branch(&mut self, name: String) {
        self.sessions.last_mut().map(|session| session.add_branch(name));
    }

    fn write_to_html(&self, ago: Option<u64>) -> bool {
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
                file.write_all(self.to_html(ago).as_bytes()).unwrap();
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

    fn write_last_session_html(&self) -> bool {
        let session = match self.sessions.last() {
            Some(session) => session,
            None => return true,
        };
        let path = Path::new("./session.html");
        let file = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&path);

        let mut file = match file {
            Ok(file) => file,
            Err(why) => {
                println!("Could not write report to session.html! {}",
                        why.description());
                return false;
            }
        };

        let stylesheets = if self.config.show_commits {
            r#"<link rel="stylesheet" type="text/css" href="style.css">
"#      } else {
            r#"<link rel="stylesheet" type="text/css" href="style.css">
<link rel="stylesheet" type="text/css" href="no_commit.css">
"#
        };

        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
  {}
  <title>{} for {}</title>
</head>
<body>
{}
</body>
</html>"#,
               stylesheets,
               "Session",
               "Rafael Bachmann",
               session.to_html());
    file.write_all(html.as_bytes()).unwrap();
    format_file("session.html");
    /* Save was successful */
    true
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
                    to_string(&self).expect("Could not write serialized time sheet.");
                file.write_all(serialized.as_bytes()).unwrap();
                /* Save was successful */
                true
            }
            Err(why) => {
                println!("Could not open timesheet.json file: {}", why.description());
                false
            }
        }
    }

    pub fn write_files(&self) -> bool {
        /* TODO: avoid time-of-check-to-time-of-use race risk */
        /* TODO: make all commands run regardless of where trk is executed
         * (and not just in root which is assumed here */
        self.write_to_json() && self.write_to_html(None) && self.write_last_session_html()
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
                    Ok(..) => from_str(&serialized).unwrap_or(None),
                    Err(..) => {
                        println!("IO error while reading the timesheet file.");
                        process::exit(0);
                    }
                }
            }
            Err(..) => None,
        }
    }

    pub fn clear() {
        /* Try to get user name */
        let sheet = Timesheet::load_from_file();
        /* In case there is a sheet, there must also be a name */
        let name: Option<String> = sheet.map(|s| s.config.user_name.unwrap());

        let path = Path::new("./.trk/timesheet.json");
        if path.exists() {
            fs::remove_file(&path).unwrap_or_else(|why| {
                println!("Could not remove sessions file: {}", why.description());
            });
        }
        Timesheet::init(name.as_ref().map(|s| s.as_str()));
    }

    pub fn timesheet_status(&self) -> String {
        let mut status = format!("Sheet running for {}\n",
               sec_to_hms_string(get_seconds() - self.start));
        match self.sessions.len() {
            0 => write!(&mut status, "No sessions yet.\n").unwrap(),
            n => {
                write!(&mut status,
                       "{} session(s) so far.\nLast session:\n{}",
                       n,
                       self.sessions[n - 1].status())
                        .unwrap()
            }
        };
        status
    }

    pub fn last_session_status(&self) -> String {
        let status = self.sessions.last().map(|session| session.status());
        status.unwrap_or("No session yet.".to_string())
    }

    fn open_local_html(&self, filename: String) {
        let file_url = match env::current_dir() {
            Ok(dir) => {
                match dir.join(&filename).to_str() {
                    Some(path) => format!("file://{}", path),
                    None => {
                        println!("Invalid filename: {}.", filename);
                        process::exit(0)
                    }
                }
            }
            Err(why) => {
                println!("Couldn't obtain current directory: {}", why.description());
                process::exit(0)
            }
        };
        match Url::parse(&file_url) {
            Ok(url) => url.open(),
            Err(why) => println!("Couldn't open file: {}", why.description()),
        }
    }

    pub fn report_last_session(&self) {
        self.write_to_html(None);
        self.open_local_html("session.html".to_string());
    }

    pub fn report_sheet(&self, ago: Option<u64>) {
        self.write_to_html(ago);
        self.open_local_html("timesheet.html".to_string());
        /* Leave complete sheet html */
        self.write_to_html(None);
    }

    pub fn show_commits(&mut self, on_off: bool) {
        self.config.show_commits = on_off;
    }

    pub fn set_repo_url(&mut self, repo: String) {
        let repo =
           if repo == "" { None } else { Some(repo) };
        self.config.repository = repo;
    }

    pub fn pause_time(&self) -> u64 {
        self.sessions.iter().fold(
            0, |total, session| total + session.pause_time())
    }

    pub fn work_time(&self) -> u64 {
        self.sessions.iter().fold(
            0, |total, session| total + session.work_time())
    }

    fn to_html(&self, ago: Option<u64>) -> String {
        let timestamp = ago.unwrap_or(self.start);
        let mut sessions_html = String::new();
        for session in &self.sessions {
            if session.start > timestamp {
		write!(&mut sessions_html, "{}<hr>", session.to_html()).unwrap();
            }
        }

        let stylesheets = if self.config.show_commits {
                "<link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\">\n".to_string()
        } else {
                r#"<link rel="stylesheet" type="text/css" href="style.css">
<link rel="stylesheet" type="text/css" href="no_commit.css">
"#.to_string()
        };

        let mut html = format!(r#"<!DOCTYPE html>
<html>
    <head>
        {}
        <title>{} for {}</title>
    </head>
    <body>
    {}"#,
              stylesheets,
              "Timesheet",
              "Rafael Bachmann",
              sessions_html);

        write!(&mut html,
               r#"<section class="summary">
    <p>Worked for {}</p>
    <p>Paused for {}</p>
</div></section>"#,
               sec_to_hms_string(self.work_time()),
               sec_to_hms_string(self.pause_time()))
               .unwrap();
        write!(&mut html, "</body>\n</html>").unwrap();
        html
    }
}
