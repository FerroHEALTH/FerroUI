- The Lucide 1.51.0 glyph `Glyph::SCROLL_TEXT` for the FerroEHR design's
  Stored queries, which a console draws in place of its stand-in
  `SEARCH_CODE`. Breaking: `Glyph::ALL` is a `[(&str, Glyph); 111]`, up
  from 110 in 0.3.4, so a console that names the array's length writes 111,
  or reads it as a slice.
