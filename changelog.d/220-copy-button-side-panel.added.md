- `island::CopyButton` and `island::SidePanel`, the ready islands of the copy
  button and the side panel, with their words as props (`CopyWords`,
  `SidePanelWords`). The side panel's opener (`widget::side_panel::Opener`)
  is a link to the content's own route until the page hydrates, then the
  button that opens the panel; the panel moves the focus to its Close button
  as it opens and returns it to the opener as it closes.
- Copy in the code viewer's head (`CodeHead::copy`, the id of its button) and
  in the document pane, which copies the document in the format in force.
- `Glyph::name` and `Glyph::named`, and serde for `Glyph` (as its Lucide
  name) and for `view::button::Variant`, so an island can take either as a
  prop.
- The crate docs name the widgets that share state with a console's code
  (editor, filter, pager, dialog, tree, upload, help) as island parts.
