- The script modules answer as the library's Rust does where both decide:
  they trim white space as Rust's `str::trim` does (a typed date, the
  editor's grammar and empty state, the typeahead's words, a menu's typed
  letter, the filter), a date picker at the calendar's first or last month
  keeps its weeks to the calendar's days, the session watch reads a count
  between tabs as a `u32`, and the upload field compares a file's extension
  in ASCII case alone. A parity test now feeds each rule written in both
  languages the same inputs and fails on any difference; `docs/consoles.md`
  lists every pair.
