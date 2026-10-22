- `DatePicker` and `DateRangePicker` are lazy islands (`#[island(lazy)]`): a
  console that builds with `cargo leptos build --split` loads their code
  from chunks of its own on a page that draws them, which keeps 72 KB of
  brotli-compressed WebAssembly off every other page. Without `--split`
  nothing changes. The gallery builds with `--split`, makes its mapping
  editors, masked identifiers and concept tree lazy too, and holds its
  entry WebAssembly to the budget, with each lazy island's chunks reported
  beside it.
- Breaking: `widget::editor::editor_diagnostics` takes the list's words as
  a `view::diagnostics::DiagnosticsWords` (`DiagnosticsWords::of_page()`
  on the server), and `widget::identifier::IdentifierWords` gains `masked`,
  the words a mask is heard as.
