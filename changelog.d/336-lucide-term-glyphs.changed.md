- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 103]`, up from 94 in 0.3.2
  with the nine glyphs this release vendors, so a console that names the
  array's length writes 103, or reads it as a slice.
