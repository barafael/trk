use clap::{Parser, Subcommand};

use crate::sheet::timesheet::Timesheet;
use crate::util::{
    get_seconds, git_commit_trk, git_pull, git_push, parse_hhmm_to_seconds, set_to_trk_dir,
};
use std::process;

mod config;
mod sheet;
mod util;

#[derive(Debug, clap::Parser)]
#[clap(version, author, about)]
pub struct Arguments {
    #[clap(subcommand)]
    command: Command,
}

/// Create timesheets from git history and meta info
#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Initialise trk in this directory and give name (should match git user name)
    Init {
        /// User name. Default is git user name if set, empty otherwise.
        name: Option<String>,
    },
    /// Begin session
    Begin {
        /// Begin in the past, specify how long ago.  Time must be after the last event though.
        ago: Option<String>,
    },
    /// End session
    End {
        /// End in the past, specify how long ago.  Time must be after the last event though.
        ago: Option<String>,
    },
    /// Pause current session
    Pause {
        /// Pause note
        note: Option<String>,
        /// Begin pause in the past, specify how long ago.  Time must be after the last event though.
        ago: Option<String>,
    },
    /// Resume currently paused session
    Resume {
        /// Resume in the past, specify how long ago.  Time must be after the last event though.
        ago: Option<String>,
    },
    /// Add a note about current work or pause
    Note {
        /// Note content
        content: String,
        /// Add a note in the past, specify how long ago.  Time must be after the last event though.
        ago: Option<String>,
    },
    /// Add a commit to the event list
    Commit {
        ///Commit hash id
        hash: String,
    },
    /// Add a branch to the session's branch list
    Branch {
        /// Branch name
        name: String,
    },
    /// Show information about git commits/branches in the report
    SetShowCommits {
        /// on_or_off
        on_off: bool,
    },
    /// Set git repo url to use for turning commit hashes to links
    SetRepoUrl {
        /// url to repository
        url: String,
    },
    /// Prints the current WIP for session or sheet
    Status {
        /// Session or Sheet
        id: String,
    },
    /// Generate html report for current session or entire sheet and save it to {timesheet|session}.html
    Report {
        /// Session or Sheet
        id: String,

        /// How long the record should go back
        ago: Option<String>,
    },
    /// Temporary: clears all sessions and updates all timestamps
    Clear,
}

fn main() {
    /* Handle command line arguments with clap */
    let arguments = Arguments::parse();

    let sheet = Timesheet::load_from_file();

    /* Gets a value for config if supplied by user, or defaults to "default.conf" */
    /* let config = matches.value_of("config").unwrap_or("default.conf");
    println!("[UNUSED] Value for config: {}", config); */

    /* Special case for init because t_sheet can and should be None before initialisation
     * Also, check for .trk directory only after this */
    if let Command::Init { name } = arguments.command {
        match sheet {
            Some(..) => println!("Already initialised."),
            None => match Timesheet::init(name) {
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
    if let Command::Clear = arguments.command {
        println!("Clearing timesheet.");
        Timesheet::clear();
        git_commit_trk("Cleared timesheet");
        return;
    }

    /* Ignore commit or branch on uninitialised trk,
     * which occur when post-commit/post-checkout hooks run
     */
    if matches!(arguments.command, Command::Commit { hash: _ })
        || matches!(arguments.command, Command::Branch { name: _ })
    {
        match sheet {
            Some(..) => {}
            None => process::exit(0),
        }
    }

    /* Pull new changes first */
    git_pull();
    /* Unwrap the timesheet and continue only if timesheet file exists */
    let mut sheet = if let Some(file) = sheet {
        file
    } else {
        println!("No timesheet file! You might have to init first.");
        return;
    };

    /* Variable to hold git commit message */
    let message = match arguments.command {
        Command::Begin { ago } => {
            let timestamp: Option<u64> =
                parse_hhmm_to_seconds(&ago.unwrap_or_default()).map(|ago| get_seconds() - ago);
            sheet.new_session(timestamp);
            "begin new session"
        }
        Command::End { ago } => {
            let timestamp: Option<u64> =
                parse_hhmm_to_seconds(&ago.unwrap_or_default()).map(|ago| get_seconds() - ago);
            sheet.end_session(timestamp);
            "end session"
        }
        Command::Pause { note, ago } => {
            let timestamp: Option<u64> =
                parse_hhmm_to_seconds(&ago.unwrap_or_default()).map(|ago| get_seconds() - ago);
            match note {
                Some(note_text) => sheet.pause(timestamp, Some(note_text)),
                None => sheet.pause(timestamp, None),
            }
            "pause session"
        }
        Command::Resume { ago } => {
            let timestamp: Option<u64> =
                parse_hhmm_to_seconds(&ago.unwrap_or_default()).map(|ago| get_seconds() - ago);
            sheet.resume(timestamp);
            "resume session"
        }
        Command::Note { content, ago } => {
            let timestamp: Option<u64> =
                parse_hhmm_to_seconds(&ago.unwrap_or_default()).map(|ago| get_seconds() - ago);
            sheet.note(timestamp, content);
            "add note to session"
        }
        Command::Commit { hash } => {
            sheet.add_commit(hash);
            "add commit to session"
        }
        Command::Branch { name } => {
            sheet.add_branch(name);
            "add branch to branchlist"
        }
        Command::Status { id } => {
            match id.as_str() {
                "session" => println!("{}", sheet.last_session_status()),
                "sheet" => println!("{}", sheet.timesheet_status()),
                text => {
                    println!(
                        "What do you mean by {text}? Should be either 'sheet' or 'session'."
                    );
                }
            }
            return;
        }
        Command::Report { id, ago } => {
            match id.as_str() {
                "session" => sheet.report_last_session(),
                "sheet" => {
                    let timestamp: Option<u64> = parse_hhmm_to_seconds(&ago.unwrap_or_default())
                        .map(|ago| get_seconds() - ago);
                    sheet.report_sheet(timestamp);
                }
                text => {
                    println!(
                        "What do you mean by {text}? Should be either 'sheet' or 'session'."
                    );
                }
            }
            return;
        }
        Command::SetShowCommits { on_off } => {
            sheet.show_commits(on_off);
            "set show_commits"
        }
        Command::SetRepoUrl { url } => {
            sheet.set_repo_url(url);
            "set repo url"
        }
        Command::Init { name: _ } | Command::Clear => unreachable!(),
    };
    sheet.write_files();
    git_commit_trk(message);
    git_push();
}
