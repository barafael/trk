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

/* for process termination */
use std::process;

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
                (@arg ago: "Optional: begin in the past, specify how long ago.
                    Time must be after the last event though.")
            )
            (@subcommand end =>
                (about: "End session")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
                (@arg ago: "Optional: end in the past, specify how long ago.
                    Time must be after the last event though.")
                )
            (@subcommand pause =>
                (about: "Pause current session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg note_text: "Optional: Pause note")
                (@arg ago: "Optional: pause in the past, specify how long ago.
                    Time must be after the last event though.")
            )
            (@subcommand resume =>
                (about: "Resume currently paused session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg ago: "Optional: resume in the past, specify how long ago.
                    Time must be after the last event though.")
            )
            (@subcommand note =>
                (about: "Add a note about current work or pause")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg note_text: +required "Note text")
                (@arg ago: "Optional: Add a note in the past, specify how long ago.
                    Time must be after the last event though.")
            )
            (@subcommand commit =>
                (about: "Add a commit to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg hash: +required "Commit hash id")
            )
            (@subcommand branch =>
                (about: "Add a branch to the session's branch list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg name: +required "branch name")
            )
            (@subcommand set =>
                (about: "Set different options")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@subcommand show_git_info =>
                    (about: "Show information about git commits/branches in the report")
                    (version: "0.1")
                    (author: "mediumendian@gmail.com")
                    (@arg on_off: +required "on or off")
                    )
                 (@subcommand repo_url =>
                    (about: "Set git repo url to use for turning commit hashes to links")
                    (version: "0.1")
                    (author: "mediumendian@gmail.com")
                    (@arg on_off: +required "url")
                    )
            )
            (@subcommand status =>
                (about: "Prints the current WIP for session or sheet")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg sheet_or_session: +required "session or sheet")
                )
            (@subcommand report =>
                (about: "Generate html report for current session or entire sheet and save it to {timesheet|session}.html")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg sheet_or_session: +required "session or sheet")
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

    /* Ignore commit or branch on uninitialised trk,
     * which occur when post-commit/post-checkout hooks run
     */
    if arguments.subcommand_matches("commit").is_some() ||
       arguments.subcommand_matches("branch").is_some() {
        match sheet_opt {
            Some(..) => {}
            None =>  process::exit(0),
        }
    }

    /* Unwrap the timesheet and continue only if timesheet file exists */
    let mut sheet = match sheet_opt {
        Some(file) => file,
        None => {
            println!("No file! You might have to init first.");
            return;
        }
    };

    match arguments.subcommand() {
        ("begin", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            sheet.new_session(timestamp);
        }
        ("end", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            sheet.end_session(timestamp);
        }
        ("pause", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            let note_text = arg.value_of("note_text");
            match note_text {
                Some(note_text) => sheet.pause(timestamp, Some(note_text.to_string())),
                None => sheet.pause(timestamp, None),
            }
        }

        ("resume", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            sheet.resume(timestamp);
        }
        ("note", Some(arg)) => {
            let timestamp: Option<u64> = parse_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| timesheet::get_seconds() - ago);
            let note_text = arg.value_of("note_text").unwrap();
            sheet.note(timestamp, note_text.to_string());
        }
        ("commit", Some(arg)) => {
            let commit_hash = arg.value_of("hash").unwrap();
            sheet.add_commit(commit_hash.to_string());
        }
        ("branch", Some(arg)) => {
            let branch_name = arg.value_of("name").unwrap();
            sheet.add_branch(branch_name.to_string());
        }
        ("status", Some(arg)) => {
            match arg.value_of("sheet_or_session") {
                Some("session") => println!("{}", sheet.last_session_status()),
                Some("sheet") => println!("{}", sheet.timesheet_status()),
                Some(text) => {
                    println!("What do you mean by {}? Should be either 'sheet' or 'session'.",
                             text)
                }
                _ => unreachable!()
            }
        }
        ("report", Some(arg)) => {
            match arg.value_of("sheet_or_session") {
                Some("session") => sheet.report_last_session(),
                Some("sheet") => sheet.report_sheet(),
                Some(text) => 
                    println!("What do you mean by {}? Should be either 'sheet' or 'session'.",
                             text),
                _ => unreachable!()
            }
        }
        ("set", Some(sub_arg)) => {
            match sub_arg.subcommand() {
                ("show_git_info", Some(arg)) => {
                    match arg.value_of("on_off") {
                        Some("on")   => sheet.toggle_show_git_info(true),
                        Some("off")  => sheet.toggle_show_git_info(false),
                        Some(text) => 
                            println!("What do you mean by {}? Should be either 'on' or 'off'.",
                                text),
                        _ => unreachable!()
                    }
                }
                ("repo_url", Some(arg)) => {
                    match arg.value_of("repo_url") {
                        Some(repo_url) => sheet.set_repo_url(repo_url.to_string()),
                        Some(text) => println!("What do you mean by {}?", text),
                        _ => unreachable!(),
                    }
                }
                (text, Some(arg)) => println!("What do you mean by {}?", text),
            }

        }
        _ => unreachable!(),
    }
}

/* For parsing time in HH:MM format. */
named!(duration(&[u8]) -> Duration,
    do_parse!(
        hour: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        tag!(":") >>
        min: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>
        /*tag!(":") >>
        sec: map_res!(map_res!(nom::digit, str::from_utf8), |s: &str| s.parse::<i64>()) >>*/
        (Duration::minutes(hour * 60 + min))
    )
);

fn parse_to_seconds(timestr: &str) -> Option<u64> {
    match duration(timestr.as_bytes()) {
        Done(_, out) => Some(out.num_seconds() as u64),
        _ => {
            println!("Error while parsing!");
            None
        }
    }
}
