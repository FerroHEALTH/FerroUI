- Breaking: `view::action::ActionGroup` no longer implements `Clone`,
  `PartialEq` or `Eq`, since its popover form holds a view. A console that
  cloned or compared a group builds it again where it needs a second one
  (#354).
