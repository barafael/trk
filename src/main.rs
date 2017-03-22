#[macro_use]
extern crate clap;
use clap::App;

mod timesheet;

fn main() {
    let mut ts = timesheet::Session::new();

    let matches = clap_app!(myapp =>
        (version: "0.1")
        (author: "Rafael B. <mediumendian@gmail.com>")
        (about: "Create timesheets from git history and meta info")
            (@arg CONFIG: -c --config +takes_value "[UNUSED] Sets a custom config file")
            (@arg debug: -d ... "Sets the level of debugging information")
            (@subcommand test =>
                (about: "controls testing features")
                (version: "0.0")
                (author: "Someone E. <someone_else@other.com>")
                (@arg verbose: -v --verbose "Print test information verbosely")
                (@arg sayhello: -s --say "Greet")
            )
            (@subcommand begin =>
                (about: "Begin Session or Week")
                (version: "0.1")
                (author:  "Rafael B. <mediumendian@gmail.com>")
                (@subcommand week =>
                    (about: "Begin week")
                    (version: "0.1")
                    (author:  "Rafael B. <mediumendian@gmail.com>")
                )
                (@subcommand session =>
                    (about: "Begin session")
                    (version: "0.1")
                    (author:  "Rafael B. <mediumendian@gmail.com>")
                )
            )
        )
            .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let config = matches.value_of("config").unwrap_or("default.conf");
    println!("Value for config: {}", config);

    // Vary the output based on how many times the user used the "verbose" flag
    // (i.e. 'trk -v -v -v' or 'trk -vvv' vs 'trk -v'
    match matches.occurrences_of("v") {
        0 => println!("No verbose info"),
        1 => println!("Some verbose info"),
        2 => println!("Tons of verbose info"),
        3 | _ => println!("Don't be crazy"),
    }

    // You can handle information about subcommands by requesting their matches by name
    // (as below), requesting just the name used, or both at the same time
    if let Some(matches) = matches.subcommand_matches("begin") {
        if matches.is_present("week") {
            ts.append_event(timesheet::Event::WeekBegin)
        } else if matches.is_present("session") {
            ts.append_event(timesheet::Event::SessionBegin)
        }
    }
    println!("{:?}", ts);
}
