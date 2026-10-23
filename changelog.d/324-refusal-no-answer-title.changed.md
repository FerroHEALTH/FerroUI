- Breaking: a refusal with no status shows its summary alone as its title, as
  the design draws it; "No answer:" moves into the title's visually hidden
  lead, so assistive technology still says no answer came.
  `view::refusal::refusal_title` returns the summary alone for such a refusal,
  the new `view::refusal::refusal_lead` returns the hidden lead, and the
  catalog's `refusal-no-answer` is the lead alone and takes no `$summary`. A
  console that draws the title itself draws `refusal_lead` before it, and a
  translated catalog drops `$summary` from `refusal-no-answer`.
