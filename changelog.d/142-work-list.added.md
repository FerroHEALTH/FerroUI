- The work-list (`widget::worklist`, the ready island `island::WorkList`): a
  data table a steward walks with the keyboard. The server draws its rows
  with `worklist::row`, the first focused, and the foot with
  `worklist_foot`, which stays in view below the rows inside the card. Once
  hydrated the table is a grid with one tab stop on the focused row, which
  carries `aria-current`: `j` and Down, `k` and Up, Home and End move it and
  scroll it clear of the sticky header and the foot, and Enter opens it (the
  ready island follows the row's link; `worklist` takes the console's
  `on_open`). One focus outline goes round the whole row, its row menu
  included. With no WebAssembly the table stays a table whose rows open by
  their links.
