- Breaking: `widget::nav::Entry::href` and `widget::nav::Nest::under` are
  `Cow<'static, str>`, so a console passes a path under a base it reads at
  run time (`format!("{base}/files").into()`) as well as a literal
  (`"/files".into()`). A struct literal that set them to a `&'static str`
  adds `.into()`.
