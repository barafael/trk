#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;


mod timesheet;

fn main() {
    /* Handle command line arguments with clap */
    let arguments = clap_app!(trk =>
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
            (@subcommand metapause =>
                (about: "Pause current session and give a reason")
                (version: "0.1")
                (author: "mediumendian@gmail.com")
                (@arg reason: +required "Meta information about pause")
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
            sheet.pause();
        }
        ("metapause", Some(arg)) => {
            let reason = arg.value_of("reason").unwrap();
            sheet.metapause(reason);
        }
        ("proceed", Some(..)) => {
            sheet.proceed();
        }
        ("meta", Some(arg)) => {
            let metatext = arg.value_of("text").unwrap();
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
