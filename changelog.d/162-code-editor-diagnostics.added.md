- The code editor (B2), `widget::editor::editor_panel` over a `Panel`: F8's
  toolbar with what the text is, its line count and the console's tools; the
  editor with its gutter marks; and the status line with the cursor's line and
  column, the console's facts and the unsaved dot (`Opened` says what the
  draft is measured against), its words as props (`EditorWords`). While the
  cursor is on a marked line, the line is current and a polite live region
  says so. `editor_split` puts it beside its diagnostics, stacked under 64rem,
  and `editor_diagnostics` draws them in step with the cursor: a row's line
  link moves the caret to its line and column and focuses the editor.
  `caret_at`, `offset_of`, `mark_kind` and `current_line` are the pure parts.
- The diagnostics list (B3), `view::diagnostics`: one row per `Finding` with
  its gravity in shape, colour and word, its code, its line and column as a
  link to the line, its message and its mapping, errors first;
  `diagnostics` for one file and `diagnostics_by_file` grouped by file under
  each file's name and counts, the files with an error first. The row of the
  current line takes `is-current` and `aria-current` (#163).
- Serde for `view::code::Gravity`, so an island can take findings as props.
