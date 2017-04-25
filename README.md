# trk
Track time with annotated pauses, notes, and git commits. It is meant to run in a git directory but can run without it. When running in a git directory, commits are automatically added to the time sheet. `trk` will generate a html report.

A `trk` timesheet is a sequence of sessions, which contain events in the order they were added. A timesheet is created by `trk init`. A session can start with `trk begin` and ends with `trk end`. Pauses can be handled in a running session by `trk pause` and `trk resume`. Notes can be added by `trk note <note>`. For status output, say `trk status {sheet|session}`. To open the html report in the default browser: `trk report {sheet|session}`. `trk help` will list all possible commands.

An example:

```
10:00 $> trk init # do this once
Init successful.

10:00 $> trk begin

10:01 $> trk note 'blablabla'

10:11 $> trk pause

10:12 $> trk note "coffee" # automatically added as note on the pause

10:15 $> trk resume

18:00 $> trk end

```

```json
{
  "end": 1491549156,
  "sessions": [
    {
      "end": 1491549238,
      "events": [
        {
          "Note": {
            "text": "blablabla",
            "time": 1491549186
          }
        },
        {
          "Pause": {
            "note": "coffee",
            "time": 1491549190
          }
        },
        {
          "Resume": {
            "time": 1491549212
          }
        }
      ],
      "running": false,
      "start": 1491549174
    }
  ],
  "start": 1491549157,
  "user": "Rafael Bachmann"
}
```

Turns into something like this:

```html
<!DOCTYPE html>
<html>
<head>
  <link rel="stylesheet" type="text/css" href="style.css">
  <title>Timesheet for Rafael Bachmann</title>
</head>
<body>
  <section class="session">
    <h1 class="sessionheader">Session on 2017-04-07, 09:12:54</h1>
    <div class="entry note">
      2017-04-07, 09:13:06: Note: blablabla
    </div>
    <div class="entry pause">
      2017-04-07, 09:13:10: Started a pause
      <p class="pausenote">coffee</p>
    </div>
    <div class="entry resume">
      2017-04-07, 09:13:32: Resumed work
    </div>
    <h2 class="sessionfooter">Ended on 2017-04-07, 09:13:58</h2>
  </section>
  </body>
</html>
```

Which can be styled by style.css:

![sheet.png](https://github.com/medium-endian/trk/blob/master/sheet.png)

## Installation

Currently the best way to install this is to install rust nightly via rustup.rs, clone this repo and then run `cargo build --release` in it.

You might want to add `/home/rafael/Code/trk/target/release` to your `$PATH` to have the `trk` executable available.
For development, you might want to add `/home/rafael/Code/trk/target/debug` to your `$PATH` (in that case build with `cargo build`).
You could also install properly to `~/bin/` or something.

If you use `trk` together with `git`, it is recommended that you place `/.trk/` in your .gitignore file. `trk` will place a `.git` directory in `.trk` just to track itself, also you probably don't want to directly check in the `trk` internal files.

To automatically add abbreviated git commits or branch summaries to the history, you can copy the files `post-commit` (for commits) or `post-checkout` (for branches) to your projects `.git/hooks` directory. If those files already exist, just append the lines from the appropriate hook. All the hooks do is call `trk` with some meta info.

## Soft Dependencies

`trk` is useful together with `html-tidy` and `git`, but it also works without them. (At the moment `trk` complains a lot if html-tidy is installled even though html-tidy is not at all necessary for the program to work correctly). When you run without git (or rather without `user.name` set in `.gitconfig`) you have to provide one as in `trk init <name>`.

## TODO:
- [x] Rename ev_type to ty
- [x] Naming improvements: is\_valid\_ts, timestamp, time
- [x] Remove trailing \_opt from options which are de-opted anyway. Just rebind the names
- [x] Find out if `use util::*;` and similar is bad
- [x] Move `extern crate`s to root and reorder `extern crate` and `use`'s and `mod`s
- [x] Check if clap code should be put in a yaml file (not for now)
- [x] Simplify operations on session vec (last(), last_mut())
- [ ] Redesign timesheet to handle timestamps nicely and deduplicate all timestamp logic
- [ ] Clarify when ts + 1 is used (preferably improve logic so it becomes unnecessary)
- [ ] Simplify option handling with `if let`, `map`, `map_or` WIP
- [x] Multiple imports
- [x] Add work/pause summary to status output
- [x] Add settings/config struct
- [x] Split up into timesheet.rs into session.rs, timesheet.rs, traits.rs, and util.rs
- [ ] Move HTML templating to own lib or better use a crate like Maud
- [ ] Use Result instead of bools (with error enums?)
- [ ] Use `format!` instead of `write!` with `String::new()` WIP
- [ ] Check output of `get_seconds()` anyway
- [x] Include stylesheets and gitignore in binary
- [ ] Include Commit hooks in binary
- [x] Fix underflow in session.rs work_time()
- [x] Set the current directory to the next higher directory which contains a `.trk` directory
- [x] Set the current directory correctly even if started from within a .trk directory
- [ ] Support different natural language durations (one week, since=date, or maybe place pins...)
- [x] Add a 'set' command, for example to set git_repo_url (in order to make the commit messages links to the repo)
- [x] Flush to html every load and just open browser on report.
- [x] Format output - leave out commits and branches, for example
- [x] Improve status output (was just debug output)
- [x] Better reporting if file is not present xor corrupt
- [x] Add past event adding for begin and end
- [x] Find a way to query time sheets up to a point in the past
- [x] Open a new session if a commit or branch is pushed when no session is running. 
- [x] Convert unix timestamps to date strings
- [x] The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time
- [x] The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a to_html() implementation.
- [ ] nicetohave: Run this on a server instead of the local machine.

