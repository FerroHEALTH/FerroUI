- The record compare (`view::compare`): several records side by side, one
  column per record and one row per field, each value marked agree, differ
  or absent against the leading record as a glyph and a word, never colour
  alone. `compare` draws the table at every width with fixed columns, so a
  revealed value moves nothing, scrolling sideways inside its card with the
  field and the leading record held at the start; `compare_stacked` reads one
  record at a time under a radio choice that works with no WebAssembly, every
  mark at the end of its line; `compare_pair` switches between them at 64 rem.
  The differing rows come first when asked, and a caption whose fields all
  agree says so. A cell's value is a `ViewFn`, so it can be an island such as
  a masked identifier, or a record's identifiers in any number and order.
