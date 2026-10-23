- A relative time for family rule 11: `view::time::relative(raw, now)`
  draws a `<time>` that reads `13 s ago`, `30 min ago`, `2 h ago` the same
  day, `yesterday`, then days, weeks, months and years, against the now the
  console hands it, with the full timestamp as its title and the machine
  form as its `datetime`; `view::time::relative_reading` gives the words
  alone. The words are in the library's catalog, so every console rounds
  and says them alike. A moment after now reads as its full timestamp.
