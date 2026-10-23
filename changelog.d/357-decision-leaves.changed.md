- Breaking: `view::dock::Decision` gains the public field `leaves`, so a
  console that writes a `Decision` literal without `..Decision::default()`
  adds `leaves: false`, and sets `leaves: true` on its Discard (#357).
