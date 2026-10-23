- Breaking: the library's own words are `i18n::Word` in place of `String`.
  Every `*Words` struct holds `Word` fields; `Phrase::new` takes
  `impl Into<Word>` and `Phrase::fill` returns a `Word`. The methods that
  return a library word return a `Word`: `label` of `Theme`, `Density`,
  `Motion`, `Hints` and `Step`, `Variant::name`, `FlowState::word`,
  `Severity::word`, `Relationship::words`, the scope's `Compartment::words`,
  `Operation::words`, `Request::words` and `ScopeError::expected`,
  `Status::line`, `refusal_title`, `at_time_label`, `Noun::word`, `empty`
  and `opening`, `range_label` and `range_of` of the table windows,
  `partial_note`, `SizeWords::size` and `Refused::sentence`.
- Breaking: the fields a library word can fill are `Word`:
  `Header::title` and `subtitle`, `Tab::label`, `TabChoice::words`,
  `Column::label`, `PreferenceRow::label` and `description`, the words of
  `SelectPreference::choices` and `SegmentedForm::choices`,
  `Confirmation::title` and `message`, `Modal::title` (a `Signal<Word>`),
  `action_menu::Choice::label` and `note`, `Listbox::value`,
  `Trigger::More` and `Trigger::Select`, `ViewChoice::words`, the `words`
  and `link` of `MoreCodes` and the tree's `More`, `RangePreset::words` and
  `Grant::Refused`. `From<String>`, `From<&str>` and
  `From<Cow<'static, str>>` make a console's own words a `Word` in the
  page's language.
- Breaking: these take `impl Into<Word>`: `Tab::new`, `Column::text`,
  `button::label`, `status::badge`, `kbd::shortcut_keys`,
  `surface::definitions`, `surface::page_header`, the form's `Field::new`,
  `help`, `help_if`, `field`, `keyed_field`, `check`, `keyed_check`,
  `check_field` and `keyed_check_field`, and `copy_button` and `copy_from`.
  The islands `CopyButton` (`label`), `NavDrawer` (`close`),
  `UserMenuButton` and `MenuButton` (`cancel`), `TabSelect` (`go`) and
  `PreferenceForm` (`save`) take a `Word`, `ChartLegend`'s `gap` is an
  `Option<Word>`, and so are the matching parameters of `nav::drawer`,
  `chart::legend`, the two `menu_button`s, `tab_select` and
  `preference_form`.
