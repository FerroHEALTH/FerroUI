- The `keys` module binds every key a control announces in
  `aria-keyshortcuts`, from 40 rem up: a chord such as Ctrl Enter
  (`view::transform::Run::shortcut`) or Ctrl S (`view::dock::Decision::shortcut`)
  presses its control from inside a field too, the one in the focused
  field's form first, and a single key such as `l` or `]` presses its
  control, or focuses a field such as the dock's reason
  (`view::dock::DockField::shortcut`), outside a field alone. The dock's
  and Run's keys load it themselves, and so does the shortcut legend; a
  console's own control writes `data-fui="keys"` (`view::shortcuts::KEYS_JOB`)
  on a page with no legend.
- `view::shortcuts::shortcut_legend_with_go_to` lists a console's go-to
  pairs (`GoTo` with its first key and each `GoToPair` of a letter and a
  link of the navigation) in their own group, each as the link the `keys`
  module follows on the pair's keys; `GoTo::group` gives them as a
  `ShortcutGroup` for a console's own shortcuts page.
