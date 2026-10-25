- `widget::prefetch::routes` loads a lazy route's code when a link to it is
  hovered or focused, and the main navigation's routes once the first page is
  idle, so a first visit costs no more than a return. The gallery hydrates
  whole with every page a lazy route: its entry WebAssembly stays at 465338
  bytes brotli-compressed, under its budget.
