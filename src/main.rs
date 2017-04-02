#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;

use std::process::Command;

mod timesheet;

fn main() {
    /* Handle command line arguments with clap */
    let matches = clap_app!(trk =>
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
            (@subcommand proceed =>
                (about: "Proceed with currently paused session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
            (@subcommand meta =>
                (about: "Proceed with currently paused session")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg text: +required "Meta information about work")
            )
            (@subcommand commit =>
                (about: "add a commit to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg hash: +required "Commit hash id")
            )
            (@subcommand branch =>
                (about: "add a topic to the event list")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg name: +required "New branch name")
            )
            (@subcommand status =>
                (about: "prints the current WIP")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
            (@subcommand clear =>
                (about: "temporary: clears the deserialized file")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
       )
            .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("[UNUSED] Value for config: {}", config);

    let t_sheet = timesheet::load_from_file();

    match matches.subcommand() {
        ("init", Some(name)) => {
            match t_sheet {
                Some(..) => println!("Already initialised!"),
                None => {
                    let git_name = git_name().unwrap_or("".to_string());
                    let author = name.value_of("name").unwrap_or(&git_name);
                    match timesheet::init(author) {
                        true => println!("Init successful."),
                        false => println!("Could not initialize."),
                    }
                }
            }
        }
        ("begin", Some(..)) => {
            match t_sheet {
                //TODO: prevent double begin
                Some(mut sheet) => {
                    sheet.new_session();
                }
                None => println!("No sheet open! Did you init?"),
            }
        }
        ("end", Some(..)) => {
            match t_sheet {
                Some(mut sheet) => sheet.finalize_last(),
                None => println!("No sheet open!"),
            }
        }
        ("pause", Some(..)) => {
            match t_sheet {
                Some(mut sheet) => {
                    if !sheet.push_event(timesheet::Event::Pause {
                                             time: timesheet::get_seconds(),
                                         }) {
                        println!("Can't pause now!");
                    }
                }
                None => println!("No sheet open!"),
            }
        }
        ("proceed", Some(..)) => {
            match t_sheet {
                Some(mut sheet) => {
                    if !sheet.push_event(timesheet::Event::Proceed {
                                             time: timesheet::get_seconds(),
                                         }) {
                        println!("Can't proceed now!");
                    }
                }
                None => println!("No sheet open!"),
            }
        }
        ("meta", Some(sub_arg)) => {
            match t_sheet {
                Some(mut sheet) => {
                    let metatext = sub_arg.value_of("text").unwrap();
                    if !sheet.push_event(timesheet::Event::Meta { text: metatext.to_string() }) {
                        println!("Can't meta now!");
                    }
                }
                None => println!("No sheet open!"),
            }
        }
        ("commit", Some(sub_arg)) => {
            match t_sheet {
                Some(mut sheet) => {
                    let commit_hash = sub_arg.value_of("hash").unwrap();
                    let hash_parsed = u64::from_str_radix(commit_hash, 16).unwrap();
                    if !sheet.push_event(timesheet::Event::Commit { hash: hash_parsed }) {
                        println!("Can't commit now!");
                    }
                }
                None => println!("No sheet open!"),
            }
        }
        ("branch", Some(sub_arg)) => {
            match t_sheet {
                Some(mut sheet) => {
                    let branch_name = sub_arg.value_of("name").unwrap();
                    if !sheet.push_event(timesheet::Event::Branch {
                                             name: branch_name.to_string(),
                                         }) {
                        println!("Can't change branch now!");
                    }
                }
                None => println!("No sheet open!"),
            }
        }
        ("status", Some(..)) => {
            match t_sheet {
                Some(mut sheet) => {
                    println!("{:?}", sheet.last_status());
                }

                None => println!("No sheet open!"),
            }
        }
        ("clear", Some(..)) => {
            println!("Clearing sessions!");
            timesheet::clear_sessions();
        }
        _ => {}
    }
}

pub fn git_name() -> Option<String> {
    if let Ok(output) = Command::new("git").arg("config").arg("user.name").output() {
        if output.status.success() {
            let s = String::from_utf8_lossy(&output.stdout);
            /* remove trailing newline character */
            let mut s = s.to_string();
            s.pop().expect("Empty name in git config!?!");
            Some(s)
        } else {
            let s = String::from_utf8_lossy(&output.stderr);
            println!("git config user.name failed! {}", s);
            None
        }
    } else {
        None
    }
}
