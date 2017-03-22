extern crate time;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum Event {
    WeekBegin,
    WeekEnd,
    SessionBegin,
    SessionEnd,
    Break,
    Continue,
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
            Event::WeekBegin => {
                println!("Week begin!");
                self.events.push(e)
            }
            Event::WeekEnd => println!("Week end!"),

            Event::SessionBegin => println!("Session begin!"),
            Event::SessionEnd => println!("Session end!"),

            Event::Break => println!("Break begin!"),
            Event::Continue => println!("Continue work"),

            Event::Meta { text } => println!("Meta: {}", text),
            Event::Commit { hash } => println!("Commit hash: {}", hash),
            Event::Branch { name } => println!("Change to branch {}", name),
        }
    }
}
