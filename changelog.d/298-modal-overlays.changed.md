- Breaking: the confirmation dialog (`confirm_dialog`, `danger_dialog`),
  the popover form (`popover_form`) and the side panel (`SidePanel`) are
  native modal `<dialog>` elements with `closedby="any"`, opened by their
  trigger's `command="show-modal" commandfor=<id>` and closed by
  `command="close"`, in place of `popover="auto"` and `popovertarget`. The
  page behind is inert while one shows, the focus moves in and returns to
  the trigger, Escape and a press outside still close it, and all of it
  works with no WebAssembly (WAI-ARIA Authoring Practices, the dialog
  (modal) pattern; WCAG 2.2 SC 2.4.3 and SC 2.4.11). A console that closes
  one of these from its own code calls `HTMLDialogElement.close()`, or
  gives its confirming button `command="close" commandfor=<id>`, in place
  of `hidePopover()`. `Act::Dialog`, the Reset to defaults button and every
  trigger the library draws use the command; a trigger a console draws
  itself does the same.
- Breaking: the user menu's popup holds who is signed in above its ARIA
  menu, which is now the inner element with the id `<id>-menu` that the
  button's `aria-controls` names; the popup keeps `<id>` for
  `popovertarget`. `widget::menu::menu_button` no longer draws
  `role="menu"`: its `items` carry the head and the menu, as `user_menu`
  draws them. The button carries no `aria-expanded` until its island runs.
