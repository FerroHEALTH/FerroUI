- Breaking: `view::chart::ActivityChart` takes its axis labels as
  `ticks: Vec<(u32, String)>`, `(day index, words)`, in place of `ends`,
  so a console labels the days the FerroEHR mockups label (22 Sep, 1 Oct,
  11 Oct and 21 Oct). A console that set `ends: (first, last)` writes
  `ticks: vec![(0, first), (last_day, last)]` for the same drawing.
