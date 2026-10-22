- Every menu, the phone's bottom sheet, the user menu and the listbox stay
  inside the viewport at 400% zoom or with many items: past its height each
  scrolls inside itself, and a sheet keeps its Cancel in sight at its foot
  (WCAG 2.2 SC 1.4.10). A menu item's words, a listbox option and the user
  menu's e-mail wrap past the popup's widest, so nothing is clipped. The
  confirmation dialog scrolls past the viewport's height and breaks a long
  word in its message.
- The keyboard reaches a disabled menu item, which is announced disabled and
  takes no press (WAI-ARIA Authoring Practices, the menu pattern).
- The user menu's button no longer says it is closed before its island
  runs, when with no WebAssembly the menu can be open.
- The popover form in a browser without CSS anchor positioning is centred
  in the viewport; it opened below the viewport's foot.
