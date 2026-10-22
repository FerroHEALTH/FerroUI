- Breaking: `view::results::ResultsPane` takes `numeric`, whether the
  answer has a column the chart can draw. A console sets it from its result
  set. An answer with no row disables both exports and offers no chart, an
  answer with no numeric column offers no chart, and the pane says why in a
  quiet line.
- Every number a message takes, and every count a list card, a panel
  section, a binder tab, a navigation entry or a count cell draws, is
  grouped in threes in the marks of the page's language ("1-25 of 312,000
  rows") and never breaks between its digits. A count past what a float
  holds exactly (`Arg::from(u64)`) is written grouped as text.
- A table header wraps between its words and hyphenates a long word in the
  page's language; a heading and a field label wrap a long word (a German
  or Dutch compound) instead of widening the page.
