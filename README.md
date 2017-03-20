# trk
Make time sheets with git integration.

### Notes and TODO items ###

* Pomodoro integration?
* How to handle branches? Like commits? Assume they are feature branches and append "... started working on branch x"?
* Stats in the end. Append gitlog? Better not, since this is available anyway...
* The session struct is a list of sequential events with timestamps which is deserialized on every run, worked on, and then serialized again. It has a tostring and a tohtml impl. trk end session writes the files for the ending session to the week's directory. A session struct is identified by it's starting time (and maybe user).
* Timeout after commit and branch change without meta?
* Which options should trk status take?
* Should the serialized sessions be cleaned up from time to time?
* Is it okay to hardcode 'week' as a name for a period that is longer than a session? Or is something like 'sprint' a better name?
