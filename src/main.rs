#[macro_use]
extern crate clap;

mod timesheet;

fn main() {
    let mut ts = timesheet::Session::new();

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
       )
            .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    // You can handle information about subcommands by requesting their matches by name
    if matches.is_present("beginperiod") {
        ts.append_event(timesheet::Event::BeginPeriod)
    }
    if matches.is_present("beginsession") {
        ts.append_event(timesheet::Event::BeginSession)
    }
    if matches.is_present("endperiod") {
        ts.append_event(timesheet::Event::EndPeriod)
    }
    if matches.is_present("endsession") {
        ts.append_event(timesheet::Event::EndSession)
    }

    if matches.is_present("pause") {
        ts.append_event(timesheet::Event::Pause)
    }
    if matches.is_present("proceed") {
        ts.append_event(timesheet::Event::Proceed)
    }

    match matches.subcommand() { 
        ("meta", Some(sub_input)) => {
            let metatext = sub_input.value_of("text").unwrap();
            ts.append_event(timesheet::Event::Meta { text: metatext.to_string() });
        }
        ("commit", Some(sub_input)) => {
            let commit_hash = sub_input.value_of("hash").unwrap();
            let hash_parsed = u64::from_str_radix(commit_hash, 16).unwrap();
            ts.append_event(timesheet::Event::Commit { hash: hash_parsed });
        }
        ("branch", Some(sub_input)) => {
            let branch_name = sub_input.value_of("name").unwrap();
            ts.append_event(timesheet::Event::Branch { name: branch_name.to_string() });
        }
        _ => {}
    }
    println!("{:?}", ts);
}
