- A pick-or-type field's back control puts its picker on the listed option
  chosen before the another option, or on its first, for a select and a
  combobox picker alike, so the picker no longer reads "another" while no
  text input shows and choosing it again shows the input. The combobox takes
  a `fui:choose` event on its root that puts it on a listed value without a
  change.
