- Breaking: the tree keeps its state as data the server can read. Each
  toggle is a link to `TreeNode::toggle_href` when the console gives one,
  so a tree folds with no WebAssembly, and `Tree::open_field` posts the
  open items as hidden fields of the form around it; a console that keeps
  them in its address watches `Tree::expanded` with `Effect::watch`.
  `TreeNode::children` is now `Children` (`None`, `Unread`, `Read`), the
  builder `expandable()` is `unread()`, `Tree::id` and `Tree::label` are
  `String`s, and `Tree` takes `kind`, `editing`, `open_field` and `words`.
  `TreeNode` gains the public fields `count`, `edits`, `inactive`, `mark`,
  `removed`, `rename`, `tag` and `toggle_href`; a console that builds a
  node by struct literal names them, or starts from `TreeNode::new`. The
  tree draws inside its `fui-concept-tree` box.
