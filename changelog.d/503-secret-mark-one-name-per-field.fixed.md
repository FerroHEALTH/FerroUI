- A form can mark any number of secret fields: `widget::action::secret_mark`
  names each mark after the field it marks (`ferroui_secret_` and the
  name's bytes in hexadecimal), so a hydrated `<ActionForm>` with two marks
  no longer fails on `serde_qs` refusing a name posted twice. The plain post
  still keeps every marked value out of the answered page, and a mark named
  `ferroui_secret` alone, as 0.5.7 drew it, is still read.
