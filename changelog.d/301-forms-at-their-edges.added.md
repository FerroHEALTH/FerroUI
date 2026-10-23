- `widget::combobox::searched_combobox` and `Remote`: a combobox over a
  console's own search, for a list too long to send to the browser whole;
  the combobox writes the words typed and shows the options the search
  answers.
- The date field takes a date typed in the order the page's language writes
  one, from the catalog's new `date-order` message (`ymd`, `dmy` or `mdy`;
  English `ymd`), through `calendar::DateOrder`, `Day::parse_in` and
  `Moment::parse_in`, which a console uses to read the posted text.
- `view::refusal::split_issues`, `ISSUES_OPEN` and `ISSUES_DRAWN`, and
  `widget::upload::decode`.
