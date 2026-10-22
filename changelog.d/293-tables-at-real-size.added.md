- `table::view::data_table_wide`: a data table whose box scrolls sideways
  at every width, for a column set no console bounds such as a query's
  result set; its header gives up sticking from 64rem. A table with a code
  header (`Column::mono`) is drawn this way by `data_table` and
  `data_table_unstacked` too, and a code header breaks after a slash, a dot
  or an opening bracket.
- `i18n::number`: `grouped` writes a count in the page's language
  ("312,000"), and `format_number` is the Fluent formatter the library sets
  on its catalogs, which a console sets in the `customise` of its own
  `static_loader!` (`bundle.set_formatter(Some(ferrohealth_ui::i18n::number::format_number))`).
  The marks are the catalog messages `number-group` and `number-decimal`,
  so a language brings its own.
