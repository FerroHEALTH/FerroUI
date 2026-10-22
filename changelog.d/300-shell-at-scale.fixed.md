- The toast region shows three toasts at most (`widget::toast::SHOWN_MAX`),
  the oldest that is no refusal and has no action leaving first, stays above the page's
  decision bar, and scrolls past the viewport's height, so toasts never
  cover the decisions (WCAG 2.2 SC 2.4.11). A toast without an action stays
  while the pointer or the focus is on the region (SC 2.2.1), a long file
  name in a toast wraps, and with no WebAssembly a flash toast closes by its
  Dismiss, a native disclosure.
- The nested sidebar section draws eight children at most, the page in view
  always among them, and a link to all; a child's full label is its
  tooltip. A parent with 300 nodes ran about 9000 px and buried the groups
  under it.
- A long word in a sidebar entry wraps; it cut the label and scrolled the
  sidebar sideways (SC 1.4.10).
- A rail count past 99 shows as "99+" on its badge, which covered the glyph
  and left the window at four digits; the full sidebar still shows the
  number whole.
- The collapsed rail shows an entry's words beside it on keyboard focus as
  well as on hover, until Escape (SC 1.4.13), and keeps scrolling.
