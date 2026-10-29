- The combobox answers a `fui:choose` event on its root, which puts it on a
  listed value, or on none, without a change event. A pick-or-type field's
  back control uses it: back puts the picker on the listed option chosen
  before the another option, or on its first, for a select and a combobox
  picker alike.
