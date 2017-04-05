#[macro_use]
extern crate clap;
use clap::AppSettings::SubcommandRequiredElseHelp;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate nom;
use nom::IResult::Done;

extern crate chrono;
use chrono::Duration;

use std::str;

named!(duration(&[u8]) -> Duration,
    do_parse!(
        hour: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        min: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        (Duration::minutes(hour * 60 + min))
    )
);

mod timesheet;

fn main() {
    /* Handle command line arguments with clap */
    let arguments = clap_app!(trk =>
        (setting: SubcommandRequiredElseHelp)
        (version: "0.1")
        (author: "Rafael B. <mediumendian@gmail.com>")
        (about: "Create timesheets from git history and meta info")
            (@arg CONFIG: -c --config +takes_value "[UNUSED] Sets a custom config file")
            (@arg debug: -d ... "[UNUSED] Sets the level of debugging information")


            (@subcommand init =>
                (about: "Initialise trk in this directory")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
                (@arg name: "User name. Default is git user name if set, empty otherwise.")
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
            )
            // TODO: needed or covered by meta handling for pause?
            (@subcommand metapause =>
                (about: "Pause current session and give meta info about the pause")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg metatext: +required "Meta information about pause")
            )
            (@subcommand retropause =>
                (about: "Pause current session after taking a break, (set length of break)")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg length: +required "How long the pause was")
                (@arg metatext: "Meta information about pause")
            )
            (@subcommand resume =>
                (about: "Resume currently paused session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
            (@subcommand meta =>
                (about: "Give meta info about current work or started pause")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg metatext: +required "Meta information about work")
            )
            (@subcommand commit =>
                (about: "Add a commit to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg hash: +required "Commit hash id")
            )
            (@subcommand branch =>
                (about: "Add a branch change to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg name: +required "The branch's name")
            )
            (@subcommand status =>
                (about: "Prints the current WIP for session or sheet (eventually as html/latex)")
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
    // let config = matches.value_of("config").unwrap_or("default.conf");
    // println!("[UNUSED] Value for config: {}", config);

    let sheet_opt: Option<timesheet::Timesheet> = timesheet::Timesheet::load_from_file();

    /* Special case for init because t_sheet can and should be None before initialisation */
    if let Some(command) = arguments.subcommand_matches("init") {
        match sheet_opt {
            Some(..) => println!("Already initialised!"),
            None => {
                match timesheet::Timesheet::init(command.value_of("name")) {
                    Some(..) => println!("Init successful."),
                    None => println!("Could not initialize."),
                }
            }
        }
        return;
    }

    /* Unwrap the timesheet and continue only if sessions file exists */
    let mut sheet = match sheet_opt {
        Some(file) => file,
        None => {
            println!("No sessions file! You might have to init first.");
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
        ("pause", Some(..)) => {
            sheet.pause(None);
        }
        ("metapause", Some(arg)) => {
            let metatext = arg.value_of("metatext").unwrap();
            sheet.pause(Some(metatext.to_string()));
        }
        /*("retropause", Some(arg)) => {
            println!("{:?}", parse_to_seconds("30:00"));
            let length_in_seconds = parse_to_seconds(arg.value_of("length").unwrap());
            let metatext = arg.value_of("metatext");
            match metatext {
                Some(metatext) => sheet.retropause(length_in_seconds, Some(metatext.to_string())),
                None => sheet.retropause(length_in_seconds, None),
            }
        }*/
        ("resume", Some(..)) => {
            sheet.resume();
        }
        ("meta", Some(arg)) => {
            let metatext = arg.value_of("metatext").unwrap();
            sheet.push_meta(metatext.to_string());
        }
        ("commit", Some(arg)) => {
            let commit_hash = arg.value_of("hash").unwrap();
            let hash_parsed = u64::from_str_radix(commit_hash, 16).unwrap();
            sheet.push_commit(hash_parsed);
        }
        ("branch", Some(arg)) => {
            let branch_name = arg.value_of("name").unwrap();
            sheet.push_branch(branch_name.to_string());
        }
        ("status", Some(which)) => {
            match which.value_of("which") {
                Some("session") => println!("{:?}", sheet.last_session_status()),
                Some("sheet") => println!("{:?}", sheet.timesheet_status()),
                Some(text) => println!("What do you mean by {}?", text),
                None => {}
            }
        }
        ("clear", Some(..)) => {
            println!("Clearing sessions!");
            timesheet::Timesheet::clear_sessions();
        }
        _ => unreachable!(),
    }
}

fn parse_to_seconds(timestr: &str) -> Option<u64> {
    match duration(timestr.as_bytes()) {
        Done(_, out) => Some(out.num_seconds() as u64),
        _ => None,
    }
}
