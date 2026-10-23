- A menu item (F13) and a title row's action (F14) can submit the console's
  own form to another address: `widget::action_menu::Act::SubmitTo { action,
  form }` draws a submit button with `formaction`, of the form with the id
  `form` or of the form it sits in, so no nested `<form>` is dropped by the
  parser.
- A menu behind a small labelled button: `widget::action_menu::labelled_menu(glyph,
  words, menu)` and its trigger `widget::action_menu::Trigger::Labelled { glyph,
  words }`, such as E7's Export.
