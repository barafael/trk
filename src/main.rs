#[macro_use]
extern crate clap;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;

mod timesheet;

fn main() {
    let mut session = timesheet::Session::new();

    /* Handle command line arguments with clap */
    let matches = clap_app!(trk =>
        (version: "0.1")
        (author: "Rafael B. <mediumendian@gmail.com>")
        (about: "Create timesheets from git history and meta info")
            (@arg CONFIG: -c --config +takes_value "[UNUSED] Sets a custom config file")
            (@arg debug: -d ... "[UNUSED] Sets the level of debugging information")

            (@subcommand beginperiod =>
                (about: "Begin period")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
            )
            (@subcommand beginsession =>
                (about: "Begin session")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
            )
            (@subcommand endperiod =>
                (about: "End period")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
            )
            (@subcommand endsession =>
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

    match matches.subcommand() { 
        ("beginperiod", Some(..)) => session.push_event(timesheet::Event::BeginPeriod),
        ("endperiod", Some(..)) => session.push_event(timesheet::Event::EndPeriod),
        ("beginsession", Some(..)) => session.push_event(timesheet::Event::BeginSession),
        ("endsession", Some(..)) => session.push_event(timesheet::Event::EndSession),
        ("pause", Some(..)) => session.push_event(timesheet::Event::Pause),
        ("proceed", Some(..)) => session.push_event(timesheet::Event::Proceed),
        ("meta", Some(sub_input)) => {
            let metatext = sub_input.value_of("text").unwrap();
            session.push_event(timesheet::Event::Meta { text: metatext.to_string() });
        }
        ("commit", Some(sub_input)) => {
            let commit_hash = sub_input.value_of("hash").unwrap();
            let hash_parsed = u64::from_str_radix(commit_hash, 16).unwrap();
            session.push_event(timesheet::Event::Commit { hash: hash_parsed });
        }
        ("branch", Some(sub_input)) => {
            let branch_name = sub_input.value_of("name").unwrap();
            session.push_event(timesheet::Event::Branch { name: branch_name.to_string() });
        }
        ("status", Some(..)) => {
            session.status();
        }
        ("clear", Some(..)) => {
            // TODO: really do it
            println!("Clearing session and period!");
        }
        _ => {}
    }

    // Convert the session to a JSON string.
    let serialized = serde_json::to_string(&session).unwrap();

    // Prints serialized session
    println!("serialized = {}", serialized);

    // Convert the JSON string back to a Session.
    let deserialized: timesheet::Session = serde_json::from_str(&serialized).unwrap();

    // Prints deserialized Session
    println!("deserialized = {:?}", deserialized);
}
