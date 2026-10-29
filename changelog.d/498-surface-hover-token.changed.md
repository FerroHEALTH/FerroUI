- Every menu item that is not destructive or disabled draws a visible bar under
  the pointer and the keyboard focus, in the new `--surface-hover` token: a
  neutral tint a little darker than the panel in light and a little lighter in
  dark (1.37:1 and 1.53:1 against it, the most muted text on it allows at
  4.5:1). Danger items keep `--bad-ground`, a pressed or current item and a
  chosen listbox option keep `--accent-tint` under the pointer, and a disabled
  item draws no bar. The combobox and listbox options and the tree rows hover
  with the same token.
