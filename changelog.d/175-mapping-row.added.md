- The mapping row (M6, `view::mapping`): a concept map's mapping as a table
  row of its source coding, the relationship as its words with an arrow,
  the target coding and the comment (`mapping_columns`, `mapping_row`).
  `Relationship` reads the R4 `ConceptMapEquivalence` and the R5
  `ConceptMapRelationship` codes, each in the words its version reads, and
  a mapping to nothing says so with its own glyph and a named dash for its
  target. In an editor each source code is one block (`map_element`) over
  its target rows, F7 rows that name a missing relationship under the row.
