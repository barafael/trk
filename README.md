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

### Dependencies

Soft dependencies are html-tidy and git, but `trk` works without them too. (At the moment `trk` complains a lot if html-tidy is installled even though html-tidy is not necessary for the program to work correctly). When you run without git (or rather without a name set in .gitconfig) you have to provide one like `trk init <name>`.

### TODO:
- [] Rename ev_type to ty
- [] naming improvements: is\_valid\_ts, timestamp, time
- [] remove trailing \_opt from options which are de-opted anyway. Just rebind the names
- [] Simplify operations on session vec
- [] Simplify option handling with `if let`, `map`, `map_or`
- [] Multiple imports
- [] Move `extern crate`s to root and reorder `extern crate` and `use`'s
- [] Split up into timesheet.rs into session.rs, timesheet.rs, traits.rs, and util.rs
- [] Move HTML templating to own lib or better use a crate like Maud
- [] Use Result instead of bools (with error enums?)
- [] Consider using `format!` instead of `write!` with `String::new()`
- [] Improve timestamp logic:
- [] Clarify when ts + 1 is used (preferably improve logic so it becomes unnecessary)
- [] Dedup checking logic for valid timestamp
- [] Check output of `get_seconds()` anyway
* TODO: run 'make sync_hook' on init? Make dependency?
* TODO: support different natural language durations (one week, since=date, or maybe place pins...)
* TODO/nicetohave: Run this on a server instead of the local machine.
- [x] TODO: add a 'set' command, for example to set git_repo_url (in order to make the commit messages links to the repo)
- [x] TODO: flush to html every load and just open browser on report.
- [x] TODO: Format output - leave out commits and branches, for example
- [x] TODO: improve status output (was just debug output)
- [x] TODO: better reporting if file is not present xor corrupt
- [x] TODO: add past event adding for begin and end
- [x] TODO: Find a way to query time sheets up to a point in the past
- [x] TODO: Open a new session if a commit or branch is pushed when no session is running. 
- [x] TODO: Convert unix timestamps to date strings (locale?)
- [x] The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time (and maybe git author?).
- [x] The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a tohtml impl.
