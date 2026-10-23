- `view::popover::PopoverForm::id` and `view::dialog::Confirmation::id` take a
  `Cow<'static, str>`, so each table row's popover form or confirmation can
  carry an id of its own, such as `remove-node-c`. Breaking: a static id
  becomes `"save-query".into()`.
