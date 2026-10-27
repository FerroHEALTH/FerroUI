- `widget::editor::code_editor_by_edit` and
  `widget::editor::editor_panel_by_edit` draw the editor and the code
  editor panel as `code_editor` and `editor_panel` do, and hand the console
  each edit the reader makes as a `widget::editor::TextEdit` (the replaced
  byte range and its replacement) instead of the whole text, so a console
  keeps its own copy in step with `TextEdit::apply` at the cost of the
  edit. The editor's `text` signal then carries the opening text and
  outside changes only; the console does not write each edited text back
  into it. `code_editor` and `editor_panel` are unchanged.
