# trk
Make time sheets with git integration.

trk is supposed to generate time sheets based on its inputs and the git history. For example:

10:00 $> trk begin

10:01 $> trk meta 'blablabla'

10:10 $> trk commit 56345636 #automated, via git hook

10:11 $> trk pause

10:15 $> trk proceed

...

18:00 $> trk end

```json
{
  "start": 1491231869,
  "end": 1491231868,
  "user": "Rafael Bachmann",
  "sessions": [
    {
      "end": 1491232120,
      "events": [
        {
          "Commit": {
            "hash": 1446270518
          }
        },
        {
          "Pause": {
            "time": 1491232022
          }
        },
        {
          "Proceed": {
            "time": 1491232027
          }
        }
      ],
      "start": 1491231897
    }
  ]
}
```

TODO: This json string should be converted to a nice representation as time sheet output.

### Notes and TODO items ###

* Pomodoro integration?
* How to handle branches? Like commits? Assume they are feature branches and append "... started working on topic x"?
* The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time (and maybe git author?).
* The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a tohtml impl. trk end pushes the ending session to the end of the Timesheet.
* Timeout after [commit or branch change] without meta?
* Which options should trk status take and what should it output? Open a browser window with report?
