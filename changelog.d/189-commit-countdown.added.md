- `widget::countdown` and the island `CommitCountdown`, the commit-confirm
  countdown (F2): a ring and the time left with the deadline in words, drawn
  by the server from the seconds left the console supplies (`Countdown`,
  `CountdownWords::of_page`) and counted down in the browser by the page's
  monotonic clock; the time is a `timer` that is not live, a polite region
  says the time left at 60, 30 and 10 seconds (`ANNOUNCE_AT`) and when the
  time is up, and reduced motion stops the ring's sweep.
  `view::lifecycle::trial_actions` draws it with Roll back now and the
  primary Confirm.
