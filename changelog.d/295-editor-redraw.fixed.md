- The code editor redraws only the lines an edit changes. Each edit
  tokenizes the text once and patches the layer's lines in place, where it
  wrote the whole layer again; the lines after an added or removed line
  only change their ids, and a caret move or a new check keeps the
  highlighting and changes only the marks. A keystroke in a 10000-line
  draft went from about 170 ms to about 75 ms in Chrome, most of which is
  now the browser's own layout of the text area.
