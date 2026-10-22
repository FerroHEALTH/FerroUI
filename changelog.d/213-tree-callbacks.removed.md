- Breaking: `widget::tree::tree_view_collapsing` and the tree's
  `on_expand` and `actions` callbacks. No callback crosses an island: a
  level is read by the server behind the toggle's link, and a row's
  actions are `TreeNode::edits`.
