- E7's results pane rides the console's own form for a run that is a form
  post: `Switch::Form { form }` draws Table and Chart as submits of that
  form posting `shown`, and each export is a `view::results::Export`, a
  link (`Export::Link`) or a submit of that form with `formaction`
  (`Export::Submit`), so no form of the pane's own sits inside the
  console's.
