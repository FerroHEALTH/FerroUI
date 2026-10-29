- `view::dialog::danger_action_dialog` draws a danger confirmation with fields
  inside the `<ActionForm>` a console draws for its server action (class
  `view::dialog::DIALOG_FORM`): once the page has hydrated, its confirm runs the
  action in the page and the dialog closes when the console's `done` signal turns
  true; with no WebAssembly the form posts to the server function's address as
  a plain form. `danger_form_dialog` keeps posting to a plain route.
