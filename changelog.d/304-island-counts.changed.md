- Breaking: `EditorWords`, `SizeWords` and `IdentifierWords` carry
  `grouping`, the page's `i18n::number::Grouping`. A console that builds
  one by struct literal sets it with `Grouping::of_page()`; each type's
  `of_page()` does so.
