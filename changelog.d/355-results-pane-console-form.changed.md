- Breaking: `view::results::ResultsPane::csv` and `ResultsPane::json` take a
  `view::results::Export`, so an address becomes `Export::Link(address)`,
  and `view::results::Switch` gains the `Form` variant, so a console that
  matches on `Switch` exhaustively adds an arm (#355).
