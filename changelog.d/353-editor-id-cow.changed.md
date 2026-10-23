- `widget::editor::Editor::id` takes a `Cow<'static, str>`, so a page with a
  number of editors known only at run time gives each its own id and field
  name, such as `staged-3`. `Editor` is `Clone` and no longer `Copy`.
  Breaking: a static id becomes `"draft".into()`, and an editor used twice,
  as by `editor_panel` and `editor_diagnostics`, is cloned for the first.
