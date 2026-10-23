- Breaking: `widget::editor::Editor::editable` says whether the text may be
  edited at all, and an editable text area takes keys from the first paint.
  A console that passed a signal turning true as the page hydrated passes
  `Signal::stored(true)` for an editable text, and false only for a text
  the reader may only read (#356).
