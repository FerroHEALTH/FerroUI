- Breaking: `island::CopyButton` takes `source: CopySource` in place of
  `text: String`. `CopySource::Text` carries a short text as before;
  `CopySource::Element` copies the text of an element the page draws, and
  `CopySource::Lines` the code lines of a viewer. The document pane copies
  its raw view and the code viewer its lines, so a large document reaches
  the page once per view and never as island data (a 0.5 MB bundle made a
  4.9 MB page). Line breaks are copied as the page holds them, a carriage
  return as a line feed. `widget::copy::copy_from` draws the button over a
  source; `copy_button` keeps its live text.
