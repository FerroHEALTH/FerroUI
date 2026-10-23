- E8 reads the OpenID Connect `profile` scope as the request it is: the new
  `view::scope::Request::Profile` is read from `profile` by
  `Request::parse`, so `ScopeGrant::read` no longer draws it refused, and it
  says "Your profile: your name, picture and the like" (OpenID Connect Core
  1.0, § 5.4).
