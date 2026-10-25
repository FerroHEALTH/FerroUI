- The top bar's search answers under the field as you type, and its answer
  is never drawn in a page again. Once the page hydrates, the search box asks
  the console's answer route (`SearchBox::suggest`, from
  `SearchBox::suggest_from` characters) and opens a panel anchored under the
  field: the WAI-ARIA combobox with a listbox popup, its hits grouped by kind
  under headings, at most five of a kind, the words searched for emphasised,
  and a "See all results" option that submits to the find page; ArrowDown and
  ArrowUp walk every kind with the focus kept in the field, rows show while it
  waits, when nothing matched and when the route failed. Breaking:
  `search_box` takes the `SearchBox` alone, `search_answer` is gone (draw the
  find page with `find_results`), the `Search` island takes `SearchWords` in
  place of its children, `SearchAnswer` gains `query` and `unfinished` and
  crosses the wire as JSON, and the hit classes are `fui-hit`,
  `fui-hit-name` and `fui-hit-line`. The route's wire is in
  `docs/consoles.md`.
