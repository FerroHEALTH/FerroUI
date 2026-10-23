- T3's file line can open the informational dialog: `Source::File::how` and
  the second argument of `file_line` are a `view::template::settings::How`.
  `How::Page(url)` keeps the link, and `How::Dialog(id)` draws a button
  styled as the link (`fui-link-button`) that opens `info_dialog` by its id
  with the `show-modal` command, with no script. Breaking: a console wraps
  its URL in `How::Page(…)`, or passes `How::Dialog(…)` and drops a
  title-row button that opened the same dialog.
