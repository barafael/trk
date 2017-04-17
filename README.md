# trk
Track time with annotated pauses, notes, and git commits. It is meant to run in a git directory but can run without it. When running in a git directory, commits are automatically added to the time sheet. trk will generate a html report.

For example:

```
10:00 $> trk init # do this only once
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

Soft dependencies are html-tidy and git, but trk works without them too. (At the moment trk complains a lot if html-tidy is installled. html-tidy is not necessary for the program to work correctly). When you run without git (or rather without a name set in .gitconfig) you have to provide one like 'trk init <name>'.

### TODO:
* TODO: run 'make sync_hook' on init? Make dependency?
[x] TODO: add a 'set' command, for example to set git_repo_url (in order to make the commit messages links to the repo)
[x] TODO: flush to html every load a
nd just open browser on report.
[x] TODO: Format output - leave out commits and branches, for example
[x] TODO: improve status output (was just debug output)
[x] TODO: better reporting if file is not present xor corrupt
[x] TODO: add past event adding for begin and end
[x] TODO: Find a way to query time sheets up to a point in the past
[x] TODO: Open a new session if a commit or branch is pushed when no session is running. 
[x] TODO: Convert unix timestamps to date strings (locale?)
* TODO: support different natural language durations (one week, since=date, or maybe place pins...)
* TODO/nicetohave: Run this on a server instead of the local machine. What happens to git commits and the like?
[x] The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time (and maybe git author?).
[x] The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a tohtml impl.
