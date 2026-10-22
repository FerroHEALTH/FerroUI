- The concept tree and the concept browser (M1), the folder tree of a
  directory (E4) and the path browser of a template (E5).
  `widget::tree` draws the tree as `TreeKind::Concept`, `Folder` or `Path`:
  a level read one at a time (`Children::Unread` shows the loading row
  while the console's server reads it behind the toggle's link), a long
  level ending in the more row (`More::of_page`), the selected concept's
  path in the heavier weight, M2's inactive mark, a folder's glyph and
  count, a reference's short uid, a path's RM type, and in an edit each
  row's named icon actions (`Edit`, submit buttons of the console's form)
  and the inline rename field. The ready island `island::TreeView` takes
  the tree as plain data (`TreeData`) with its words (`TreeWords`), and is
  lazy. `view::browser` lays out the panes (`Panes::Three`, `Flat`, `Two`),
  the pane's find field, the search's matches and the path inspector
  (`path_node`); `view::path` draws an archetype id that breaks only after
  a dot and a path that breaks only after a slash, a dot or an opening
  bracket. The gallery's Trees page shows each in every brand.
