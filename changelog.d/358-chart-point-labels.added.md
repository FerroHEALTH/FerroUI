- The line chart's figures table names each point by its own label:
  `LineChart::x_labels`, each an X value and its label, such as a point's
  time on a time axis, which the table under the plot reads before a tick's
  label and the number; the axis draws `x_ticks` alone. Breaking: a
  `LineChart` literal adds `x_labels: Vec::new()`.
