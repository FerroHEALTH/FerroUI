- The code editor (B2), the YAML editor and the upload field keep every key
  typed while the console's text comes back late: the text area is the
  source of the live text, a text the editor handed to `on_input` coming
  back is an echo and never written, and only an outside change that differs
  from the field is written into it. Before, a key typed before the echo of
  an earlier one was overwritten and the caret jumped to the end.
