- The search box's panel placement no longer panics when a navigation
  disposes the search between the panel opening and the next animation
  frame: the search cancels the pending frame when it is disposed, and the
  frame reads and writes nothing of a disposed search.
