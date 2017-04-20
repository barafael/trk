use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{Local, TimeZone};
use chrono::Duration;

use nom::IResult::Done;
use nom;

/* For running git and html-tidy */
use std::process::Command;

/* For from::utf8 */
use std::str;

pub fn get_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn ts_to_date(timestamp: u64) -> String {
    Local
        .timestamp(timestamp as i64, 0)
        .format("%Y-%m-%d, %H:%M")
        .to_string()
}

pub fn sec_to_hms_string(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds - hours * 3600) / 60;
    let seconds = seconds - minutes * 60 - hours * 3600;
    match (hours, minutes, seconds) {
        (0, 0, 1) => format!("1 second"),
        (0, 0, s) => format!("{} seconds", s),
        (0, 1, _) => format!("1 minute"),
        (0, m, _) => format!("{} minutes", m),
        /* Range matching: slightly dubious feature here */
        (1, 0...4, _)   => format!("1 hour"),
        (h, 0...4, _)   => format!("{} hours", h),
        (h, 56...59, _) => format!("{} hours", h + 1),
        (h, m, _)       => format!("{} hours and {} minutes", h, m),
    }
}

/* For parsing time in HH:MM format. */
named!(duration_hhmm(&[u8]) -> Duration,
    do_parse!(
        hour: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        min:  map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        /*tag!(":") >>
        sec: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>*/
        (Duration::minutes(hour * 60 + min))
    )
);

pub fn parse_hhmm_to_seconds(timestr: &str) -> Option<u64> {
    match duration_hhmm(timestr.as_bytes()) {
        Done(_, out) => Some(out.num_seconds() as u64),
        _ => None,
    }
}

pub fn git_author() -> Option<String> {
    if let Ok(output) = Command::new("git")
           .arg("config")
           .arg("user.name")
           .output() {
        if output.status.success() {
            let output = String::from_utf8_lossy(&output.stdout);
            /* Remove trailing newline character */
            let mut output = output.to_string();
            output
                .pop()
                .expect("Empty name in git config? Not even a newline?!?");
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

pub fn git_commit_message(hash: &str) -> Option<String> {
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

pub fn format_file(filename: &str) {
    if let Ok(_) = Command::new("tidy")
           .arg("--tidy-mark")
           .arg("no")
           .arg("-i")
           .arg("-m")
           .arg(filename)
           .output() {
    } else {
        println!("tidy-html not found!");
    }
}
