- The code editor highlights a JSON or XML document again only from the
  first line a key changes, up to the line where the highlighting is the
  same as before, where before it highlighted the whole text on every key.
  The stylesheet's rules for a focused work-list row and for a field row
  with a help line select the same elements as before, written so Chrome
  no longer restyles the editor's whole layer on every key. A key typed
  into a 20 kB JSON or XML document now costs 1 to 2 ms at the median,
  script, style and layout together, where it cost 5 to 8 ms, so typing an
  openEHR composition into the editor no longer slows with its length.
