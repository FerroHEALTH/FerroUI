- `widget::prefetch::routes` loads a lazy route's code when a link to it is
  hovered or focused, and the main navigation's routes once the first page is
  idle, one after another, so a first visit costs no more than a return. The
  gallery hydrates whole with its 21 pages in five lazy routes: its entry
  WebAssembly is 474860 bytes brotli-compressed, under its budget, and its
  pages' code is 32 chunks.
