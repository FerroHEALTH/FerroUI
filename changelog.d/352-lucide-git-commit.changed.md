- Breaking: `Glyph::ALL` is a `[(&str, Glyph); 110]`, up from 103 in 0.3.3
  with the seven glyphs this release vendors (#352, #360, #363), so a
  console that names the array's length writes 110, or reads it as a slice.
