- F7 repeatable rows take two more layouts: `widget::repeat::Fields::One`, a
  row of one field with its Remove, and `Fields::Card`, a sub-card whose
  heading (the new `RepeatRow::heading`) names its group with Remove beside
  it, then its fields, a list of its own among them when it has one. A press
  inside a nested list adds or removes only that list's rows. Breaking: a
  `RepeatRow` literal adds `heading: None`.
