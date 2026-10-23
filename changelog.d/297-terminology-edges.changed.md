- Breaking: `widget::tree` addresses an item by its path key, the keys from the top
  to it (`path_key`, read back with `path_keys`): `Tree::expanded`,
  `Tree::selected`, `TreeData::open`, `TreeData::selected` and the open
  fields a tree posts hold path keys, and an item's key need only be unique
  among its siblings. The same concept under two parents (a SNOMED CT
  polyhierarchy) is two items, each with its own id, fold, selection and
  place in the keyboard's walk. A top item's path key is its key; a console
  that opens or selects a deeper item builds its path key.
- `view::browser::match_list` takes the rows as `Vec<AnyView>`, and the
  `Match` struct is removed: the match the browser reads marks its link
  `aria-current="true"` (`Coding::current`), and its row takes the tint
  from that mark, where it took `aria-selected` on a plain list item,
  which WAI-ARIA 1.2 does not allow there.
- `view::coding::Coding` has a `current` field (`Coding::current`), which
  puts `aria-current="true"` on its link; a struct literal names it.
- `view::clause::Clause` takes `issues: Vec<ClauseIssue>` in place of
  `issue`: two or more broken invariants draw as a list, one issue each.
  `clause_codes` takes the more row (`Option<MoreCodes>`) after the codes,
  and `view::browser::PathNode` has a `more` field, which also sets the
  count in its value set's heading.
- `widget::filter::table_filter` shows its count at once and says it from
  a status region of its own once the typing pauses (`SETTLE`, 600 ms), so
  assistive technology hears one count for a burst of keys.
