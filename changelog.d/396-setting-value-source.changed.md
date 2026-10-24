- Breaking: `view::template::settings::Setting` gains the public field
  `source: Option<ValueSource>`, so a console sets `source: None` on each
  setting, or the source its running configuration names, in place of its
  own marker. The library's catalog adds `setting-source-default`,
  "Default", `setting-source-environment`, "From the environment", and
  `setting-source-file`, "From the file", so a console that translates the
  library's catalog adds the three.
