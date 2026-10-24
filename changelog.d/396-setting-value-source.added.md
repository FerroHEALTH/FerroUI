- T3: a setting row says where its value came from when that is not the
  page's own source, small and muted after the value with its glyph:
  `view::template::settings::ValueSource::Environment`, "From the
  environment", for an environment variable, which wins over the file and
  a revision; `ValueSource::Default`, "Default", when nothing sets it; and
  `ValueSource::File`, "From the file", for a section the file sets in a
  managed configuration. The page's own source is unmarked: the file in
  file mode, the running revision when managed. The gallery's file-mode
  settings page shows an environment and a default value.
