- Breaking: `widget::editor::Editor` no longer implements `Copy`, since its
  id may be made at run time; it is `Clone`. An editor used twice, as by
  `editor_panel` and `editor_diagnostics`, is cloned for the first use
  (#353).
