- A dialog drawn inside a table's actions cell wraps its sentence: every
  dialog (`fui-dialog`) sets `white-space: normal` and `text-align: start`,
  so it no longer inherits the cell's `nowrap` and end alignment, which cut
  its sentence off at 390 px. The gallery's member removals sit in a table,
  each confirmation beside its row menu in the actions cell. A console
  changes nothing.
