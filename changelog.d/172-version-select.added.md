- The FHIR version select (M3, `widget::version::version_select`): the
  version a console reads through as one compact control in the top bar,
  the version alone with its label and release as its tooltip and
  accessible name, over a menu of `menuitemradio` links that checks the
  root in view and marks the default beside each release. Below 40 rem it
  leaves the top bar, and `VersionSelect::choices` hands the user menu the
  same roots as its segmented group. The menu island's keys walk
  `menuitemradio` items, and its trigger `Trigger::Version` draws the
  control.
