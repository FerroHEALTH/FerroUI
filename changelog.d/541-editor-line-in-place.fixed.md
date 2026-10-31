- A key that changes a line of the code editor updates that line's
  element in place, so the browser no longer counts every line number
  again, and the caret's line and column are read without walking the
  text before the caret. A key typed at the end of a 20 kB XML document
  costs the same as one in a short document, and in a 20 kB JSON document
  in the operations page it costs about 1.8 ms, where it cost 2.6 to 3.8 ms
  on the same machine.
