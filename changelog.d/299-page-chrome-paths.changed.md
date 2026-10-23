- Breaking: `view::shell::Footer`'s `version` and `about` are
  `Cow<'static, str>`, and `view::shell::lockup` takes its `href` as
  `impl Into<Cow<'static, str>>`, so a console passes a base path or a
  version it learns at run time without leaking a string; a literal takes
  `.into()`. `Footer::release` borrows the footer.
- Breaking: the phone's tab select (`widget::tabs::tab_select`, the
  `TabSelect` island) is named by the label around it and carries no id, so
  a page may hold several many-tab rows; a test or a console that found it
  by `#fui-tabs-select` finds it by `.fui-tabs-select select`.
- The top bar's search answers Escape as the WAI-ARIA Authoring Practices'
  combobox does: an open answer closes and keeps the words, a closed one
  clears them, and the focus stays in the field either way (WCAG 2.2
  SC 2.4.3); it left the field for the page before.
