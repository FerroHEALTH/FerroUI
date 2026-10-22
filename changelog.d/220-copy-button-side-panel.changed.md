- The copy button and a setting's key hint are hidden until the page
  hydrates, in place of disabled; with no WebAssembly the text they copy
  stays on the page as selectable text, and the key hint shows its key as
  text. A console that built a `CodeHead` with a struct literal adds
  `copy: None` or the id of its Copy button.
- `widget::side_panel::SidePanel` takes `id` and `title` as `String` and an
  `opener: Opener`, and draws its opener itself; its scrolling body's id is
  `id` and `-body`. Below 40 rem the panel is a sheet over the whole layout
  viewport, scrollbar excluded, with its body filling it.
