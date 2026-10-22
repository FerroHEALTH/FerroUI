- Breaking: `Tag::locked` takes the value's owner (F4), such as
  "Provisioned from mappings/observation.yml", and draws it as the tag's
  tooltip and after its word in its accessible name, so the lock and the
  dashed edge never stand alone. A console that called `.locked()` passes
  the owner it set with `.title(...)` to `.locked(...)` instead.
