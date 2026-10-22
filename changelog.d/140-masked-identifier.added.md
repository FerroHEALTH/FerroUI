- The masked identifier (`widget::identifier`), for any national or local
  identifier system: the domain is data (`Domain`, a label and the system's
  URI), and `Masked::new` masks on the server with no assumption of length,
  characters, format or check digit, keeping at most `KEEP` trailing
  characters and never more than half, behind a fixed run of dots that never
  tells the length. `identifier` draws it masked with Reveal, which runs the
  console's audited round trip; the console answers in the widget's `Reveal`
  signal, and the widget counts the seconds down, drops the value at zero or
  on Hide, and offers Copy only while it shows. The masked, revealed and busy
  faces share one reserved slot, so nothing around it moves; a refusal is a
  line in words on its row; the countdown speaks through a polite live region
  at 10 and 0 seconds. A revealed value is a `Revealed`, which never
  serializes and whose `Debug` withholds it. Before hydration Reveal is a link
  to the console's own reveal page. `identifier_static` and `identifier_list`
  are the listing forms.
- The Lucide icons `equal-not`, `eye-off` and `minus` (`Glyph::EQUAL_NOT`,
  `Glyph::EYE_OFF`, `Glyph::MINUS`).
