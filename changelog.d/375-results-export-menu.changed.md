- Breaking: E7's results pane draws its exports as one small Export button
  opening a menu of the formats (F13), CSV then JSON, as the FerroEHR
  design draws it; with no row Export is disabled. An `Export::Submit`
  item submits the console's form by `formaction`, so the pane still draws
  no form of its own inside the console's. `view::results::results_pane`
  now holds a menu island, so a console renders it inside a page with the
  server's shared context, as it does every menu. A console's own code
  needs no other change.
- Breaking: the library's catalog adds `results-export`, "Export", and
  rewords `results-export-csv` and `results-export-json` to "CSV" and
  "JSON", the menu's items under Export. A console that translates the
  library's catalog adds `results-export` and rewords the two.
- Breaking: `widget::action_menu::Act` gains the `SubmitTo` variant, so a
  console that matches on `Act` exhaustively adds a `SubmitTo` arm.
- Breaking: `widget::action_menu::Trigger` gains the `Labelled` variant, so
  a console that matches on `Trigger` exhaustively adds a `Labelled` arm.
