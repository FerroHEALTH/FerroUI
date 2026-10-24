- Breaking: E5's node inspector draws Build a query on this path only
  where the node has a query: `view::browser::PathNode::query` is an
  `Option<String>`, and a node with `None`, such as one of a demographic
  template (a PERSON's or a ROLE's), where no query can be built, draws no
  button. A console wraps its link in `Some(..)` to keep today's drawing,
  and passes `None` for a template no query can be built on.
