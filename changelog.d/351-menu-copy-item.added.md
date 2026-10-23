- A copy item for the menus (F13): `widget::action_menu::Act::Copy(text)`
  puts the text on the clipboard with the copy button's clipboard code and
  says so in a toast, Copied to the clipboard or the browser's refusal. Only
  script can copy, so the item is hidden until the menu's island runs and
  the text stays where the page shows it. In a title row a copy action is
  the copy button. Breaking: a `match` over `Act` adds the `Copy` arm, and
  `widget::action_menu::menu_button` and `island::MenuButton` take the copy
  words (`copy`, `None` for a menu with no copy item).
