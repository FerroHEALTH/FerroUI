- Breaking: E7's results pane draws its exports as one small Export button
  opening a menu of the formats (F13), CSV then JSON, as the FerroEHR
  design draws it; with no row Export is disabled. An `Export::Submit`
  item submits the console's form by `formaction`, so the pane still draws
  no form of its own inside the console's. The pane now holds a menu
  island, so it renders inside a page with the server's shared context, as
  every menu does. The catalog's `results-export-csv` and
  `results-export-json` read "CSV" and "JSON", and the new `results-export`
  reads "Export": a console that translates the library's catalog adds
  `results-export` and rewords the two. A console's own code needs no
  change.
- A menu item (F13) and a title row's action (F14) can submit the console's
  own form to another address: `widget::action_menu::Act::SubmitTo { action,
  form }` draws a submit button with `formaction`, of the form with the id
  `form` or of the form it sits in, so no nested `<form>` is dropped by the
  parser. Breaking: a console that matches on `Act` exhaustively adds a
  `SubmitTo` arm.
- A menu behind a small labelled button: `widget::action_menu::labelled_menu(glyph,
  words, menu)` and its trigger `widget::action_menu::Trigger::Labelled { glyph,
  words }`, such as E7's Export. Breaking: a console that matches on
  `Trigger` exhaustively adds a `Labelled` arm.
