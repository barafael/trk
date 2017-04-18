use std::fmt::Write;

/* For branch name dedup */
use std::collections::HashSet;

use util::*;

use std::process;

use timesheet::traits::HasHTML;

#[derive(Serialize, Deserialize, Debug)]
pub enum EventType {
    Pause,
    Resume,
    Note,
    Commit { hash: String },
}

#[derive(Serialize, Deserialize, Debug)]
struct Event {
    timestamp : u64,
    note      : Option<String>,
    ty        : EventType
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub start    : u64,
    pub end      : u64,
    running  : bool,
    branches : HashSet<String>,
    events   : Vec<Event>,
}

impl Session {
    pub fn new(timestamp: Option<u64>) -> Session {
        let timestamp = match timestamp {
            Some(timestamp) => timestamp,
            None => get_seconds(),
        };
        Session {
            start    : timestamp,
            end      : timestamp + 1,
            running  : true,
            branches : HashSet::<String>::new(),
            events   : Vec::<Event>::new(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.running
    }

    pub fn is_paused(&self) -> bool {
        match self.events.len() {
            0 => false,
            n => {
                match self.events[n - 1].ty {
                    EventType::Pause => true,
                    _ => false,
                }
            }
        }
    }

    pub fn update_end(&mut self) {
        self.end = match self.events.len() {
            0 => self.end,
            n => &self.events[n - 1].timestamp + 1,
        }
    }

    pub fn finalize(&mut self, timestamp: Option<u64>) {
        let timestamp = match timestamp {
            None => get_seconds(),
            Some(timestamp) => {
                let is_valid_ts = match self.events.len() {
                    0 => timestamp > self.start,
                    n => {
                        let last_ev = &self.events[n - 1];
                        timestamp > last_ev.timestamp
                    }
                };
                if is_valid_ts {
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

    pub fn push_event(&mut self,
                  timestamp : Option<u64>,
                  note      : Option<String>,
                  type_of_event : EventType)
                  -> bool {
        /* Cannot push if session is already finalized. */
        if !self.is_running() {
            println!("Already finalized, cannot push event.");
            return false;
        }

        let timestamp = match timestamp {
            None => {
                let now = get_seconds();
                self.end = now;
                now
            }
            Some(timestamp) => {
                let is_valid_ts = match self.events.len() {
                    0 => timestamp > self.start,
                    n => timestamp > self.events[n - 1].timestamp,
                };
                if is_valid_ts {
                    self.end = timestamp + 1;
                    timestamp
                } else {
                    println!("That timestamp is before the last event.");
                    return false;
                }
            }
        };
        /* TODO: improve logic */
        match type_of_event {
            // TODO: fix this, so both note and ago work...
            EventType::Pause => {
                if self.is_paused() {
                    println!("Already paused.");
                    false
                } else {
                    self.events
                        .push(Event {
                                  timestamp : timestamp,
                                  note      : note,
                                  ty      : EventType::Pause,
                              });
                    true
                }
            }
            EventType::Resume => {
                if !self.is_paused() {
                    println!("Currently not paused.");
                    false
                } else {
                    self.events
                        .push(Event {
                                  timestamp : timestamp,
                                  note      : note,
                                  ty        : EventType::Resume,
                              });
                    true
                }
            }
            EventType::Note => {
                if self.is_paused() {
                    /* Add note to previous pause (last of events vec) */
                    /* If self.is_paused(), then self.len() is always at least 1 */
                    let len = self.events.len();
                    let pause = &mut self.events[len - 1];
                    match pause.note {
                        Some(ref mut already) => {
                            // TODO: handle long strings (also in other types)
                            // TODO: there must be another way other than <br>
                            already.push_str("<br>");
                            already.push_str(&note.unwrap());
                        }
                        None => pause.note = note,
                    }
                } else {
                    self.events
                        .push(Event {
                                  timestamp : timestamp,
                                  note      : note,
                                  ty        : EventType::Note,
                              })
                };
                true
            }
            /* Commit adding possible only in present */
            EventType::Commit { hash } => {
                if self.is_paused() {
                    self.push_event(None, None, EventType::Resume);
                }
                /* Commit message must be provided */
                if note.is_none() {
                    println!("No commit message found for commit {}.", hash);
                }
                self.events
                    .push(Event {
                              timestamp : get_seconds(),
                              note      : note,
                              ty        : EventType::Commit { hash },
                          });
                true
            }
        }
    }

    pub fn pause_time(&self) -> u64 {
        let mut pause_time = 0;
        let mut last_pause_ts = 0;
        for event in &self.events {
            match event.ty {
                EventType::Pause => last_pause_ts = event.timestamp,
                EventType::Resume => pause_time += event.timestamp - last_pause_ts,
                _ => {}
            }
        }
        pause_time
    }

    pub fn working_time(&self) -> u64 {
        let pause_time = self.pause_time();
        self.end - self.start - pause_time
    }

    pub fn add_branch(&mut self, name: String) {
        if self.is_running() {
            self.branches.insert(name);
        }
    }

    pub fn status(&self) -> String {
        let mut status = String::new();
        write!(&mut status,
               r#"Session running since {}.
"#,
               sec_to_hms_string(get_seconds() - self.start))
                .unwrap();
        if self.is_paused() {
            write!(&mut status,
                   r#"    Paused since {}.
"#,
                   sec_to_hms_string(get_seconds() - self.events[self.events.len() - 1].timestamp))
                    .unwrap();
        } else {
            match self.events.len() {
                0 => write!(&mut status, "    No events in this session yet!\n").unwrap(),
                n => {
                    write!(&mut status,
                           "    Last event: {:?}, {} ago.\n",
                           &self.events[n - 1].ty,
                           sec_to_hms_string(get_seconds() - self.events[n - 1].timestamp))
                            .unwrap()
                }
            }
        }
        let branch_str = match self.branches.len() {
            0 => String::new(),
            n => {
                self.branches.iter().fold(
                    format!("Worked on {} branches: ", n),
                    |res, s| res + s + " "
                )
            }
        };
        status.push_str(&branch_str);
        status
    }
}

impl HasHTML for Event {
    fn to_html(&self) -> String {
        match self.ty {
            EventType::Pause => {
                match self.note {
                    Some(ref info) => {
                        format!(r#"<div class="entry pause">{}: Started a pause
    <p class="mininote">{}</p>
</div>"#,
                                ts_to_date(self.timestamp),
                                info.clone())
                    }
                    None => {
                        format!(r#"<div class="entry pause">{}: Started a pause
</div>"#,
                                ts_to_date(self.timestamp))
                    }
                }
            }
            EventType::Resume => {
                format!(r#"<div class="entry resume">{}: Resumed work
<hr>
</div>"#,
                        ts_to_date(self.timestamp))
            }
            /* An EventType::Note note is a Some because it's
             * 'constructor' function takes a String
             * (and not Option<String>)
             */
            EventType::Note => {
                match self.note {
                    Some(ref text) => {
                        format!(r#"<div class="entry note">{}: Note: {
}
<hr>
</div>"#,
                                ts_to_date(self.timestamp),
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
                    Some(ref text) => {
                        format!(r#"<div class="entry commit">{}: Commit id: {}
    <p class="mininote">message: {}</p>
  <hr>
</div>"#,
                                ts_to_date(self.timestamp),
                                hash,
                                text)
                    }
                    None => unreachable!(),
                }
            }
        }
    }
}

impl HasHTML for Session {
    fn to_html(&self) -> String {
        let mut html = format!(r#"<section class="session">
    <h1 class="sessionheader">Session on {}</h1>"#,
                               ts_to_date(self.start));

        for event in &self.events {
            html.push_str(&event.to_html());
        }

        write!(&mut html,
               r#"<h2 class="sessionfooter">Ended on {}</h2>"#,
               ts_to_date(self.end))
                .unwrap();

        let mut branch_str = String::new();
        match self.branches.len() {
            0 => {}
            n => {
                write!(&mut branch_str, "Worked on {} branches: ", n).unwrap();
                for branch in &self.branches {
                    write!(&mut branch_str, "{} ", branch).unwrap();
                }
            }
        };

        write!(&mut html,
               r#"<section class="summary">
    <p>{}</p>
    <p>Worked for {}</p>
    <p>Paused for {}</p>
</div></section>"#,
               branch_str,
               sec_to_hms_string(self.working_time()),
               sec_to_hms_string(self.pause_time()))
                .unwrap();

        write!(&mut html, "</section>").unwrap();
        html
    }
}