- A long code (more than 24 characters, such as a versioned uid or a UUID)
  breaks after a colon, a dot, a hyphen, a slash or a vertical bar, and a
  canonical breaks inside a long piece between two slashes the same way, so
  a `urn:uuid:` or `urn:oid:` canonical and a versioned uid stay inside a
  320 px page; an ordinary code still never breaks.
- A concept tree keeps the width of a long name beside its code and count,
  where the name ran under them; every tree now scrolls sideways instead.
- An invariant's number in a clause issue (`vsd-3`) never breaks at its
  hyphen, and a long match in a search's list breaks inside its row.
