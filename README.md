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

Turns into something like this:

```html
<!DOCTYPE html>
<html>
   <head>
      <title>Timesheet for Rafael Bachmann</title>
      <link rel="stylesheet" type="text/css" href="style.css">
   </head>
   <body>
      <section class="session">
         <h1 class="sessionheading">Session on 2017-04-05 14:03:25 +02:00</h1>
         <div class="entry pause">2017-04-05 14:03:30 +02:00:	Started a pause</div>
         <div class="entry resume">2017-04-05 14:03:37 +02:00:	Resumed work</div>
         <div class="entry commit">2017-04-05 14:03:55 +02:00:	Commit id: 213141</div>
         <div class="entry commit">2017-04-05 14:04:47 +02:00:	Commit id: 213141</div>
         <div class="entry pause">2017-04-05 14:04:56 +02:00:	Started a pause</div>
         <div class="entry resume">2017-04-05 14:04:59 +02:00:	Resumed work</div>
      </section>
      <section class="session">
         <h1 class="sessionheading">Session on 2017-04-05 17:50:50 +02:00</h1>
         <div class="entry meta">2017-04-05 17:51:02 +02:00:	Note: test content bla</div>
         <div class="entry metapause">2017-04-05 17:51:20 +02:00:	test meta pause</div>
         <div class="entry resume">2017-04-05 17:51:43 +02:00:	Resumed work</div>
         <div class="entry commit">2017-04-05 17:51:49 +02:00:	Commit id: 213141</div>
      </section>
   </body>
</html>
```
Which can be styled by style.css.

### Notes and TODO items ###

* TODO: Find a way to add an event in the past
* TODO: Find a way to query time sheets from certain time periods (one week, since=date, or maybe place pins...)
* Which options should trk status take and what should it output? Open a browser window with report?
* TODO: Format output - leave out commits and branches, for example
* TODO/nicetohave: Run this on a server instead of the local machine. What happens to git commits and the like?
* How to handle branches? Like commits? Assume they are feature branches and append "... started working on topic x"?
* The session struct is a list of sequential events with timestamps. A session struct is identified by it's starting time (and maybe git author?).
* The Timesheet struct is deserialized on every run, worked on, and then serialized again. It has a tohtml impl.
