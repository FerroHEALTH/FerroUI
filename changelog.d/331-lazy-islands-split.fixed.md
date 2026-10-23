- The library's lazy islands (`DatePicker`, `DateRangePicker`, `TreeView`) are
  lazy only with the new `split` feature, so a console built without
  `cargo leptos build --split` no longer gets a glue that imports
  `__wasm_split_placeholder__` and hydrates no island. A console that builds
  with `--split` turns the feature on to keep their code in chunks of its own.
  `docs/consoles.md` says how a console builds, and the gallery's site build
  fails on a glue that imports the placeholder.
