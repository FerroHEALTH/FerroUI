- `view::secret`, the write-only secret field (F3): `secret_field` draws
  what is set now by reference (`SecretState`: set from a file, set and not
  shown, not set, or set and unreadable) and one radio group of Keep (Leave
  unset when not set), Set a value, Read from a file and Clear
  (`SecretChoice`), whose chosen control shows with no script; the value is
  a password input with `autocomplete="off"` the page never fills, and
  `SecretField::refuse_value` refuses a value where no secrets directory is
  set. `secret_state` draws the state alone for a read-only settings row.
