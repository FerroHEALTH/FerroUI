- Breaking: `view::diagnostics::FileFindings` takes `all`, the console's
  page of every finding in the file, and `DiagnosticsWords` takes `more`,
  `more_all` and `none`. A console that builds either by struct literal
  names the new fields; `DiagnosticsWords::of_page()` fills the words.
