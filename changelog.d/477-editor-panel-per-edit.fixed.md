- The code editor panel (`widget::editor::editor_panel`) keeps its
  toolbar's line count and its status line's changed state by each edit:
  the count comes from the editor's index of the lines, and the changed
  state compares only the bytes between the parts the draft still shares
  with the opened text, so a key no longer counts the lines of the whole
  text or compares the whole draft with the opened one.
