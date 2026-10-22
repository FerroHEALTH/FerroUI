- Breaking: `view::chart::tick_label` takes the axis step as well,
  `tick_label(value, step)`, and prints the step's decimals; a console that
  printed a figure with it calls the new `view::chart::value_label`, which
  keeps six significant digits. `view::chart::MARKS` names the four point
  marks a series' palette index also picks.
- Breaking: `view::fanout::Verdict` has a fourth variant, `Unasked`, for a
  fan-out with no node asked; a console that matches on the verdict adds an
  arm. A legend swatch is an SVG of the series' mark (`svg.fui-swatch`), no
  longer a `span`.
