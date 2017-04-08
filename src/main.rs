/* Command Line Argument Parser */
#[macro_use]
extern crate clap;
use clap::AppSettings::SubcommandRequiredElseHelp;

#[macro_use]
extern crate serde_derive;

/* For parsing time */
#[macro_use]
extern crate nom;
use nom::IResult::Done;

/* For time handling */
extern crate chrono;
use chrono::Duration;

/* For from::utf8 */
use std::str;

mod timesheet;

fn main() {
    /* Handle command line arguments with clap */
    let arguments = clap_app!(trk =>
        (setting: SubcommandRequiredElseHelp)
        (version: "0.1")
        (author: "Rafael B. <mediumendian@gmail.com>")
        (about: "Create timesheets from git history and meta info")
            /* (@arg CONFIG: -c --config +takes_value "[UNUSED] Sets a custom config file") */
            /* (@arg debug: -d ... "[UNUSED] Sets the level of debugging information") */

            (@subcommand init =>
                (about: "Initialise trk in this directory and give name (should match git user name)")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
                (@arg name: "Optional: user name. Default is git user name if set, empty otherwise.")
            )
            (@subcommand begin =>
                (about: "Begin session")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
            )
            (@subcommand end =>
                (about: "End session")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
            )
            (@subcommand pause =>
                (about: "Pause current session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg time: "Optional: Time in the past (after the last event though!) which pause should be added.")
            )
            (@subcommand resume =>
                (about: "Resume currently paused session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg time: "Optional: Time in the past (after the last event though!) which resume should be added.")
            )
            (@subcommand note =>
                (about: "Add a note about current work or pause")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg time: "Optional: Time in the past (after the last event though!) which note should be added.")
                (@arg note_text: +required "Note text")
            )
            (@subcommand commit =>
                (about: "Add a commit to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg hash: +required "Commit hash id")
            )
            (@subcommand status =>
                (about: "Prints the current WIP for session or sheet")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg which: +required "session or sheet")
                )
            (@subcommand report =>
                (about: "Generate html report for current session or entire sheet and save it to {timesheet|session}.html")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg which: +required "session or sheet")                
            )
            (@subcommand clear =>
                (about: "Temporary: clears all sessions and updates all timestamps")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
       )
            .get_matches();

    /* Gets a value for config if supplied by user, or defaults to "default.conf" */
    /* let config = matches.value_of("config").unwrap_or("default.conf");
    println!("[UNUSED] Value for config: {}", config); */

    let sheet_opt: Option<timesheet::Timesheet> = timesheet::Timesheet::load_from_file();

    /* Special case for init because t_sheet can and should be None before initialisation */
    if let Some(command) = arguments.subcommand_matches("init") {
        match sheet_opt {
            Some(..) => println!("Already initialised."),
            None => {
                match timesheet::Timesheet::init(command.value_of("name")) {
                    Some(..) => println!("Init successful."),
                    None => println!("Could not initialize."),
                }
            }
        }
        return;
    }

    /* Special case for clear because t_sheet can be None when clearing (corrupt file) */
    if let Some(command) = arguments.subcommand_matches("clear") {
        match sheet_opt {
            Some(..) => {
                println!("Clearing timesheet.");
                timesheet::Timesheet::clear();
            }
            None => {
                match timesheet::Timesheet::init(command.value_of("name")) {
                    Some(..) => println!("Reinitialised timesheet."),
                    None => println!("Could not initialize."),
                }
            }
        }
        return;
    }

    /* Unwrap the timesheet and continue only if timesheet file exists */
    let mut sheet = match sheet_opt {
        Some(file) => file,
        None => {
            println!("No or corrupt timesheet file! You might have to init first.");
            return;
        }
    };

    match arguments.subcommand() {
        ("begin", Some(..)) => {
            sheet.new_session();
        }
        ("end", Some(..)) => {
            sheet.end_session();
        }
        ("pause", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("time").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            sheet.pause(timestamp, None);
        }

        ("resume", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("time").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            sheet.resume(timestamp);
        }
        ("note", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("time").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            let note_text = arg.value_of("note_text").unwrap();
            sheet.note(timestamp, note_text.to_string());
        }
        ("commit", Some(arg)) => {
            let commit_hash = arg.value_of("hash").unwrap();
            sheet.commit(commit_hash.to_string());
        }
        ("status", Some(arg)) => {
            match arg.value_of("which") {
                Some("session") => println!("{:?}", sheet.last_session_status()),
                Some("sheet") => println!("{:?}", sheet.timesheet_status()),
                Some(text) => {
                    println!("What do you mean by {}? Should be either 'sheet' or 'session'.",
                             text)
                }
                None => {}
            }
        }
        ("report", Some(arg)) => {
            match arg.value_of("which") {
                Some("session") => {
                    sheet.last_session_report();
                }
                Some("sheet") => {
                    sheet.report();
                }
                Some(text) => {
                    println!("What do you mean by {}? Should be either 'sheet' or 'session'.",
                             text)
                }
                None => {}
            }
        }

        _ => unreachable!(),
    }
}

/* For parsing time in HH:MM:SS format. TODO: extend to other formats or find better solution */
named!(duration(&[u8]) -> Duration,
    do_parse!(
        hour: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        min: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        sec: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        (Duration::seconds(hour * 60 * 60 + min * 60 + sec))
    )
);

fn parse_to_seconds(timestr: &str) -> Option<u64> {
    match duration(timestr.as_bytes()) {
        Done(_, out) => Some(out.num_seconds() as u64),
        _ => None,
    }
}
