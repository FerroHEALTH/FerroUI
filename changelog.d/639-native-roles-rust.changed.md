- The pager's range, the filter's count, the countdown's set points, the
  topology's zoom level, the busy spinner and skeleton lines and the date
  pickers' status lines are `<output>` elements; the data table's,
  compare's, line diff's and root matrix's scroll boxes are named
  `<section>` elements, and the reading pane's `<section>` drops its
  redundant role; the action menu's and the filter menu's separators are
  `<hr>` elements. Each keeps its role and draws as before. A console
  script or test that selected one by `[role="status"]`, `[role="region"]`
  or `[role="separator"]` selects it by its element, and
  `scripts/checks/markup-rules.sh` (M6) refuses a new such role unless a
  `NOTE:` beside it says why the element keeps it.
