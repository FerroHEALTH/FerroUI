- `view::surface::Header::subtitle_view` gives a page header a subtitle with
  inline markup, such as a bold word and a code link to another page, in the
  `fui-page-subtitle` style, drawn in place of the text `subtitle` when set.
  The text form works as before; a console that writes a `Header` literal
  without `..Header::default()` adds `subtitle_view: None`.
