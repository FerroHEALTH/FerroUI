- The editor's work per keystroke follows the edit, not the length of the
  text: an edit re-highlights only the lines it touched (a grammar that reads
  across lines, JSON, XML, AQL and HL7 v2, is highlighted whole again once
  typing rests for 250 ms), the caret is read from an index of the lines
  instead of a scan of the text, and a key no longer reads the text area's
  text more than once. Typing into a 64 KB document takes the editor about
  0.4 ms of script per key, down from about 2 ms.
