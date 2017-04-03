#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;


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
                (about: "prints the current WIP for session or entire sheet (eventually as json)")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg which: +required "session or sheet")
                )
            (@subcommand clear =>
                (about: "temporary: clears the deserialized file")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
            )
       )
            .get_matches();

    /* Gets a value for config if supplied by user, or defaults to "default.conf" */
    // let config = matches.value_of("config").unwrap_or("default.conf");
    // println!("[UNUSED] Value for config: {}", config);

    let t_sheet: Option<timesheet::Timesheet> = timesheet::Timesheet::load_from_file();

    match matches.subcommand() {
        ("init", Some(name)) => {
            match t_sheet {
                Some(..) => println!("Already initialised!"),
                None => {
                    let author = match name.value_of("name") {
                        Some(n) => Some(n),
                        None => None,
                    };
                    match timesheet::Timesheet::init(author) {
                        Some(..) => println!("Init successful."),
                        None => println!("Could not initialize."),
                    }
                }
            }
        }
        ("begin", Some(..)) => {
            match t_sheet {
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
        ("status", Some(which)) => {
            match t_sheet {
                Some(sheet) => {
                    match which.value_of("which") {
                        Some("session") => println!("{:?}", sheet.last_session_status()),
                        Some("sheet") => println!("{:?}", sheet.timesheet_status()),
                        Some(text) => println!("What do you mean by {}?", text),
                        None => {}
                    }
                }

                None => println!("No sheet open!"),
            }
        }
        ("clear", Some(..)) => {
            println!("Clearing sessions!");
            timesheet::Timesheet::clear_sessions();
        }
        _ => {}
    }
}
