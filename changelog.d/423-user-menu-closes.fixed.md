- The user menu closes once one of its items is chosen: since the router
  draws the next page in the document, a link or a theme choice in the menu
  left it open over the new page. Its handlers also read nothing the router
  has disposed when a toggle or a focus change arrives after a navigation, as
  the action menu's do.
