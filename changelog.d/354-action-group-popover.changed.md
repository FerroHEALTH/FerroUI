- Breaking: `view::action::ActionGroup` gains the public field `popover`,
  so a console that writes an `ActionGroup` literal adds `popover: None`, or
  builds the group with `ActionGroup::popover(form)` (#354).
