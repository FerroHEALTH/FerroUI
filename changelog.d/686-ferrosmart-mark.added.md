- `view::shell::ferrosmart_mark()` draws FerroSMART's mark, the shield with
  the keyhole of FerroHEALTH/FerroSMART `assets/brand/ferrosmart-icon.svg`,
  for `lockup`, in place of `mark(Glyph::KEY_ROUND)`: the shield in the
  accent and the keyhole in the yellow, amber 100 on a dark ground.
  `Glyph::SHIELD` (Lucide `shield`) stands for FerroSMART in a list.
- A theme file may set `--accent-fill`, `--accent-fill-hover` and
  `--accent-fill-ink` when its accent cannot be a fill on a light ground;
  the primary action, the split button and the chosen day paint with them,
  and a theme that leaves them out fills with its accent as before.
