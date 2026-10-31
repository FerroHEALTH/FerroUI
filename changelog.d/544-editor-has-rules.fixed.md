- The stylesheet no longer looks for a decision bar, a toast, a work-list
  foot or an open menu sheet anywhere in the page with `:has()`, which made
  Chrome weigh the whole page again on every key typed into the code
  editor. Each rule now looks only where the shell draws what it looks
  for: the decision bar as the last part of the main area or of its
  column, the work-list's card in the column at most two levels down, the
  toast region beside the body, the menu sheets in the top bar and in a
  transform pane's head, and the editor as a transform pane's own child.
  A code editor with an error mark carries `is-invalid`, which draws its
  red edge. A key typed at the end of a 20 kB XML document costs 0.2 ms at
  the median where it cost 0.7 to 1.4 ms, and in a 20 kB JSON document 0.5
  to 0.6 ms where it cost 0.9 to 1.0 ms, on the same machine.
