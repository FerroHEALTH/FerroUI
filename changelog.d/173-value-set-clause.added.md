- The value set clause (M4, `view::clause`): an include or exclude of a
  value set as its mark, its source as the title, the sentence of what it
  selects, then its filter rows and its codes, each an M2 coding
  (`clause_codes`), with `clause_part` and `clause_fields` for a
  composer's parts and `clauses` to run them edge to edge in their
  section. Include is a solid accent edge and Exclude a dashed warn edge,
  each with its word, and an invariant the clause breaks is named by its
  number inline (`ClauseIssue`).
- The repeatable rows (F7, `widget::repeat::repeat` and the island
  `Repeater`): a list of the same fields, each row ending in its labelled
  Remove (or a lock that says why the row stays), one Add under the list
  and a quiet line for an empty list. Remove and Add are submit buttons of
  the console's form, so they post with no WebAssembly; once the page
  hydrates, Remove takes its row out and moves the focus to the next row,
  and Add inserts a copy of the blank row with a fresh key (`ROW`) and
  focuses it. On a phone Remove stays on the row's first line. M4's filter
  rows and M6's target rows are F7 rows (`Fields::Three`).
