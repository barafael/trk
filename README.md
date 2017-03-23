# trk
Make time sheets with git integration.

### Notes and TODO items ###

* Pomodoro integration?
* How to handle branches? Like commits? Assume they are feature branches and append "... started working on topic x"?
* The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time (and maybe git author?).
* The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a tohtml impl. trk end pushes the ending session to the end of the Timesheet.
* Timeout after [commit or branch change] without meta?
* Which options should trk status take and what should it output? Open a browser window with report?
