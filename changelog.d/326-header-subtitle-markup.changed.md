- Breaking: `view::surface::Header` gains the public field `subtitle_view`,
  so a console that writes a `Header` literal without `..Header::default()`
  adds `subtitle_view: None`.
