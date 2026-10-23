- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 111]`, up from 110 in 0.3.4
  with the glyph this release vendors, so a console that names the array's
  length writes 111, or reads it as a slice.
