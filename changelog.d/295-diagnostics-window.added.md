- The diagnostics list windows a long file and says when it is empty. With
  the new `FileFindings::all` link, a file of more than
  `view::diagnostics::WINDOW` (100) findings draws the first 100 and one
  quiet row, "100 of 2000 findings shown. Show all 2000", linking to the
  console's page of them all (`fui-diags-more`); a list or a file with no
  finding says "No errors or warnings." (`fui-diags-empty`). Breaking:
  `FileFindings` takes `all`, and `DiagnosticsWords` takes `more`,
  `more_all` and `none`.
