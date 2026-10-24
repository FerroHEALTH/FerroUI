- The FerroFED configuration design has an Authentication tab after FHIR
  face, in the managed and the off states, which a console draws with
  `view::template::settings::settings_panel` and `setting_row`: who may
  call, read-only with no Edit in both states, since only `ferrofed.toml`
  sets it. It holds the audience, the operator scope, each token issuer
  with its key set file, and each trusted issuer with its introspection
  secret named by its file and, for a national contact point, its claim
  names, its correlation header and its added countries.
