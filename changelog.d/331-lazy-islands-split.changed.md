- Breaking: `DatePicker`, `DateRangePicker` and `TreeView` are lazy islands
  only with the new `split` feature. A console that builds with
  `cargo leptos build --split` turns on `features = ["split"]` for
  `ferrohealth-ui` to keep their code in chunks of its own; a console that
  builds without `--split` changes nothing and now hydrates them.
