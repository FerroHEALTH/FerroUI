- Breaking: the navigation drawer is a native modal `<dialog>` with
  `closedby="any"` around the `<nav>` landmark, and the top bar's Menu
  button opens it with `command="show-modal" commandfor=<id>` in place of
  `popover="auto"` and `popovertarget`. The page behind is inert while it
  shows, the focus moves in and returns to the button, and Escape and a
  press outside close it, all with no WebAssembly (WAI-ARIA Authoring
  Practices, the dialog (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11).
  From 48 rem up the closed dialog is the column beside the page. The id a
  console passes as `Sidebar::id` is now the dialog's; a stylesheet or
  script of a console's own that selects `nav.fui-sidebar` or
  `.fui-sidebar:popover-open` selects `.fui-sidebar` and
  `.fui-sidebar[open]`.
- `widget::nav::Nest` takes an `all` field, the words of the link
  to the parent's page ("All 300 nodes") that the section draws when it
  holds more than `NEST_SHOWN` (8) children.
- A collapsed rail entry carries its words in a drawn tooltip in place of a
  `title` attribute.
