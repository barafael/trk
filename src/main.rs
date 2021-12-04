use crate::sheet::timesheet::Timesheet;
use crate::util::{
    get_seconds, git_commit_trk, git_pull, git_push, parse_hhmm_to_seconds, set_to_trk_dir,
};
use clap::clap_app;
use clap::AppSettings::SubcommandRequiredElseHelp;
use std::process;

mod config;
mod sheet;
mod util;

fn main() {
    /* Handle command line arguments with clap */
    let arguments = clap_app!(trk =>
        (setting: SubcommandRequiredElseHelp)
        (version: "0.9")
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
            (@subcommand set_show_commits =>
                    (about: "Show information about git commits/branches in the report")
                    (version: "0.1")
                    (author: "mediumendian@gmail.com")
                    (@arg on_off: +required "on or off")
            )
            (@subcommand set_repo_url =>
                    (about: "Set git repo url to use for turning commit hashes to links")
                    (version: "0.1")
                    (author: "mediumendian@gmail.com")
                    (@arg url: +required "url to repository")
            )
            (@subcommand status =>
                (about: "Prints the current WIP for session or sheet")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg sheet_or_session: +required "session or sheet")
            )
            (@subcommand report =>
                (about:
"Generate html report for current session or entire sheet and save it to {timesheet|session}.html")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg sheet_or_session: +required "session or sheet")
                (@arg ago: "How long the record should go back")
            )
            (@subcommand clear =>
                (about: "Temporary: clears all sessions and updates all timestamps")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
       )
            .get_matches();

    let sheet = Timesheet::load_from_file();

    /* Gets a value for config if supplied by user, or defaults to "default.conf" */
    /* let config = matches.value_of("config").unwrap_or("default.conf");
    println!("[UNUSED] Value for config: {}", config); */

    /* Special case for init because t_sheet can and should be None before initialisation
     * Also, check for .trk directory only after this */
    if let Some(command) = arguments.subcommand_matches("init") {
        match sheet {
            Some(..) => println!("Already initialised."),
            None => match Timesheet::init(command.value_of("name")) {
                Some(..) => {
                    println!("Init successful.");
                    git_commit_trk("initialise trk");
                }
                None => println!("Could not initialize."),
            },
        }
        return;
    }

    /* Set current dir to the next upper directory containing a .trk directory */
    if !set_to_trk_dir() {
        println!("Fatal: not a .trk directory (or subdirectory of one).");
        process::exit(0);
    }

    /* Special case for clear because t_sheet can be None when clearing (corrupt file) */
    if let Some(command) = arguments.subcommand_matches("clear") {
        match sheet {
            Some(..) => {
                println!("Clearing timesheet.");
                Timesheet::clear();
                git_commit_trk("Cleared timesheet");
            }
            None => match Timesheet::init(command.value_of("name")) {
                Some(..) => {
                    println!("Reinitialised timesheet.");
                    git_commit_trk("Reinitialised timesheet.");
                }
                None => println!("Could not initialize."),
            },
        }
        return;
    }

    /* Ignore commit or branch on uninitialised trk,
     * which occur when post-commit/post-checkout hooks run
     */
    if arguments.subcommand_matches("commit").is_some()
        || arguments.subcommand_matches("branch").is_some()
    {
        match sheet {
            Some(..) => {}
            None => process::exit(0),
        }
    }

    /* Pull new changes first */
    git_pull();
    /* Variable to hold git commit message */
    let message;
    /* Unwrap the timesheet and continue only if timesheet file exists */
    let mut sheet = if let Some(file) = sheet {
        file
    } else {
        println!("No timesheet file! You might have to init first.");
        return;
    };

    match arguments.subcommand() {
        ("begin", Some(arg)) => {
            let timestamp: Option<u64> = parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| get_seconds() - ago);
            sheet.new_session(timestamp);
            message = "begin new session";
        }
        ("end", Some(arg)) => {
            let timestamp: Option<u64> = parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| get_seconds() - ago);
            sheet.end_session(timestamp);
            message = "end session";
        }
        ("pause", Some(arg)) => {
            let timestamp: Option<u64> = parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| get_seconds() - ago);
            let note_text = arg.value_of("note_text");
            match note_text {
                Some(note_text) => sheet.pause(timestamp, Some(note_text.to_string())),
                None => sheet.pause(timestamp, None),
            }
            message = "pause session";
        }

        ("resume", Some(arg)) => {
            let timestamp: Option<u64> = parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| get_seconds() - ago);
            sheet.resume(timestamp);
            message = "resume session";
        }
        ("note", Some(arg)) => {
            let timestamp: Option<u64> = parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                .map(|ago| get_seconds() - ago);
            let note_text = arg.value_of("note_text").unwrap();
            sheet.note(timestamp, note_text.to_string());
            message = "add note to session";
        }
        ("commit", Some(arg)) => {
            let commit_hash = arg.value_of("hash").unwrap();
            sheet.add_commit(commit_hash.to_string());
            message = "add commit to session";
        }
        ("branch", Some(arg)) => {
            let branch_name = arg.value_of("name").unwrap();
            sheet.add_branch(branch_name.to_string());
            message = "add branch to branchlist";
        }
        ("status", Some(arg)) => {
            match arg.value_of("sheet_or_session") {
                Some("session") => println!("{}", sheet.last_session_status()),
                Some("sheet") => println!("{}", sheet.timesheet_status()),
                Some(text) => {
                    println!(
                        "What do you mean by {}? Should be either 'sheet' or 'session'.",
                        text
                    );
                }
                _ => unreachable!(),
            }
            return;
        }
        ("report", Some(arg)) => {
            match arg.value_of("sheet_or_session") {
                Some("session") => sheet.report_last_session(),
                Some("sheet") => {
                    let timestamp: Option<u64> =
                        parse_hhmm_to_seconds(arg.value_of("ago").unwrap_or(""))
                            .map(|ago| get_seconds() - ago);
                    sheet.report_sheet(timestamp);
                }
                Some(text) => {
                    println!(
                        "What do you mean by {}? Should be either 'sheet' or 'session'.",
                        text
                    );
                }
                _ => unreachable!(),
            }
            return;
        }
        ("set_show_commits", Some(arg)) => {
            match arg.value_of("on_off") {
                Some("on") => sheet.show_commits(true),
                Some("off") => sheet.show_commits(false),
                Some(text) => {
                    println!(
                        "What do you mean by {}? Should be either 'on' or 'off'.",
                        text
                    );
                }
                _ => unreachable!(),
            }
            message = "set show_commits";
        }
        ("set_repo_url", Some(arg)) => match arg.value_of("url") {
            Some(repo_url) => {
                sheet.set_repo_url(repo_url.to_string());
                message = "set repo url";
            }
            _ => unreachable!(),
        },
        _ => unreachable!(),
    }
    sheet.write_files();
    git_commit_trk(message);
    git_push();
}
