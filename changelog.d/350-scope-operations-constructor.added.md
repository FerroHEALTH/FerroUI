- E8's operations can be built by a console that parses scopes with its own
  grammar: `view::scope::Operations::of(&[Operation])` and
  `Operations: FromIterator<Operation>`, so a `Scope` literal draws the
  console's own parse in `scope_grants`.
