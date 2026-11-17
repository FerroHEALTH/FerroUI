- `action_menu::Act::PostToNewWindow { action, fields }` posts a plain form
  whose answer opens in a new window (`target="_blank"`, `rel="noopener"`),
  with no script, so the page stays open behind it, as a SMART app's EHR
  launch needs. A menu item and an action button draw it with the
  external-link icon (`Glyph::EXTERNAL_LINK`) after their words, and say
  "(opens in a new window)" to assistive technology (WCAG 2.2 technique
  G201). The gallery's overlays page launches a stand-in app from its More
  menu.
