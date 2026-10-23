- Breaking: `view::chart::LineChart` gains the public field `x_labels`, so a
  console that writes a `LineChart` literal adds `x_labels: Vec::new()`, or
  each point's own label (#358).
