- `i18n::Word`: a library word with whether it fell back to English on a
  page in another language. Drawn as a text node it is wrapped in
  `<span lang="en">` when it fell back (WCAG 2.2 SC 3.1.2); written into an
  attribute it takes the page's language; in an island's props it travels as
  a plain string, or as `{"en": text}` when it fell back. `Word::lang` gives
  the `lang` of an element whose whole text is the word, such as an
  `<option>`.
