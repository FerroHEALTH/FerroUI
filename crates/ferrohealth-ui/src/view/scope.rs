// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The SMART scope grant (E8).
//!
//! One scope parsed: the raw scope in mono, breaking only after a slash,
//! a dot, a question mark, an ampersand, an equals sign or a bar, so a
//! long one never widens the page (WCAG 2.2 SC 1.4.10); the compartment
//! and the resource in words; the five operations as marks, a permitted
//! one ticked and an absent one struck through as `<del>` with "not
//! permitted" said after it, so the grant never rests on colour; the
//! search constraint that narrows it, so the grant never reads wider than
//! it is; a broad grant carries the warn badge Broad access. A scope that
//! asks for no resource ([`Request`]: identity, launch context or a
//! refresh token) says what it asks for. A malformed scope is the bad tone
//! with what the grammar expected.
//!
//! [`Scope::parse`] reads the grammar of SMART App Launch 2.2.0, "Scopes
//! and Launch Context", § SMART's scopes are used to grant access: a
//! compartment (`patient`, `user` or `system`), a slash, a resource, a dot,
//! and the permissions, a subset of the in-order string `cruds`, with an
//! optional `?param=value` constraint; the SMART 1 suffixes `read`, `write`
//! and `*` read as `rs`, `cud` and `cruds` (§ Scope equivalence with SMART
//! v1). The resource is any text without a slash, so a FHIR resource type
//! and a SMART on openEHR resource (`composition-<template>`,
//! `template-<template>`, `aql-<query>`, openEHR ITS-REST, SMART on
//! openEHR, § 8.1 Resource scopes) both parse; the permissions follow the
//! last dot, since a template id holds dots. [`Request::parse`] reads the
//! scopes of the same chapter that ask for no resource: `openid`,
//! `profile` and `fhirUser` (§ Scopes for requesting identity data;
//! `profile` is OpenID Connect Core 1.0, § 5.4), `launch` and
//! `launch/<resource>` with its optional `?role=` (§ Scopes for requesting
//! context data), and `online_access` and `offline_access` (§ Scopes for
//! requesting a refresh token). [`ScopeGrant::read`] reads a raw scope
//! either way. Which resources a server knows and which grants are broad
//! are the console's. The look is our own design.

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// Whose data a scope reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compartment {
    /// The patient in context.
    Patient,
    /// What the signed-in user may reach.
    User,
    /// A backend service's own access, with no user.
    System,
}

impl Compartment {
    /// The compartment's word in a raw scope.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::Patient => "patient",
            Self::User => "user",
            Self::System => "system",
        }
    }

    /// The compartment in words, in `locale`.
    #[must_use]
    pub fn words(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Patient => Text::ScopePatient,
                Self::User => Text::ScopeUser,
                Self::System => Text::ScopeSystem,
            },
        )
    }
}

/// One of the five operations a scope may permit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    /// `c`: create.
    Create,
    /// `r`: read.
    Read,
    /// `u`: update.
    Update,
    /// `d`: delete.
    Delete,
    /// `s`: search, or run a query.
    Search,
}

impl Operation {
    /// The five, in the order of the string `cruds`.
    pub const ALL: [Self; 5] = [
        Self::Create,
        Self::Read,
        Self::Update,
        Self::Delete,
        Self::Search,
    ];

    /// The operation's letter in a raw scope.
    #[must_use]
    pub const fn letter(self) -> char {
        match self {
            Self::Create => 'c',
            Self::Read => 'r',
            Self::Update => 'u',
            Self::Delete => 'd',
            Self::Search => 's',
        }
    }

    /// The operation in words, in `locale`.
    #[must_use]
    pub fn words(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Create => Text::ScopeCreate,
                Self::Read => Text::ScopeRead,
                Self::Update => Text::ScopeUpdate,
                Self::Delete => Text::ScopeDelete,
                Self::Search => Text::ScopeSearch,
            },
        )
    }
}

/// The operations a scope permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Operations([bool; 5]);

impl Operations {
    /// The operations `permitted`, for a console that parses scopes with
    /// its own grammar and draws them with E8.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrohealth_ui::view::scope::{Operation, Operations};
    ///
    /// let read = Operations::of(&[Operation::Read, Operation::Search]);
    /// assert!(read.permits(Operation::Search));
    /// assert!(!read.permits(Operation::Create));
    /// ```
    #[must_use]
    pub fn of(permitted: &[Operation]) -> Self {
        permitted.iter().copied().collect()
    }

    /// Whether `operation` is permitted.
    #[must_use]
    pub const fn permits(self, operation: Operation) -> bool {
        let [create, read, update, delete, search] = self.0;
        match operation {
            Operation::Create => create,
            Operation::Read => read,
            Operation::Update => update,
            Operation::Delete => delete,
            Operation::Search => search,
        }
    }

    /// The permissions `letters`, a non-empty subset of `cruds` in its
    /// order, or a SMART 1 suffix; `None` for anything else.
    fn parse(letters: &str) -> Option<Self> {
        let letters = match letters {
            "read" => "rs",
            "write" => "cud",
            "*" => "cruds",
            other => other,
        };
        let mut flags = [false; 5];
        let mut rest = letters;
        let mut any = false;
        for (index, operation) in Operation::ALL.into_iter().enumerate() {
            if let Some(after) = rest.strip_prefix(operation.letter()) {
                if let Some(flag) = flags.get_mut(index) {
                    *flag = true;
                }
                rest = after;
                any = true;
            }
        }
        (any && rest.is_empty()).then_some(Self(flags))
    }
}

impl FromIterator<Operation> for Operations {
    fn from_iter<I: IntoIterator<Item = Operation>>(permitted: I) -> Self {
        let mut flags = [false; 5];
        for operation in permitted {
            if let Some(at) = Operation::ALL.iter().position(|each| *each == operation)
                && let Some(flag) = flags.get_mut(at)
            {
                *flag = true;
            }
        }
        Self(flags)
    }
}

/// What a scope that does not parse lacks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    /// It does not open with `patient/`, `user/` or `system/`.
    Compartment,
    /// Nothing stands between the slash and the last dot, the resource
    /// holds a slash, or there is no dot.
    Resource,
    /// The text after the last dot is no set of permissions.
    Permissions,
}

impl ScopeError {
    /// What the grammar expected, in `locale`.
    #[must_use]
    pub fn expected(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::Compartment => Text::ScopeExpectedCompartment,
                Self::Resource => Text::ScopeExpectedResource,
                Self::Permissions => Text::ScopeExpectedPermissions,
            },
        )
    }
}

impl std::fmt::Display for ScopeError {
    /// Names the part the scope lacks, as a log reads it; a reader sees
    /// [`ScopeError::expected`].
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Compartment => "compartment",
            Self::Resource => "resource",
            Self::Permissions => "permissions",
        })
    }
}

impl std::error::Error for ScopeError {}

/// A SMART clinical data scope, parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    /// Whose data it reaches.
    pub compartment: Compartment,
    /// The resource, as the scope writes it: a FHIR resource type, `*`, or
    /// a SMART on openEHR resource such as `composition-*`.
    pub resource: String,
    /// The operations it permits.
    pub operations: Operations,
    /// The `param=value` constraint after the `?`, when it has one.
    pub constraint: Option<String>,
}

impl Scope {
    /// Parses the scope `raw`.
    ///
    /// # Errors
    ///
    /// Returns what the scope lacks: its compartment, its resource or its
    /// permissions ([`ScopeError`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrohealth_ui::view::scope::{Compartment, Operation, Scope, ScopeError};
    ///
    /// let scope = Scope::parse("patient/Observation.rs?category=vital-signs").unwrap();
    /// assert_eq!(Compartment::Patient, scope.compartment);
    /// assert!(scope.operations.permits(Operation::Search));
    /// assert!(!scope.operations.permits(Operation::Delete));
    /// assert_eq!(Err(ScopeError::Permissions), Scope::parse("user/*.dus"));
    /// ```
    pub fn parse(raw: &str) -> Result<Self, ScopeError> {
        let (body, constraint) = match raw.split_once('?') {
            Some((body, constraint)) => (body, Some(constraint.to_owned())),
            None => (raw, None),
        };
        let (compartment, rest) = body.split_once('/').ok_or(ScopeError::Compartment)?;
        let compartment = match compartment {
            "patient" => Compartment::Patient,
            "user" => Compartment::User,
            "system" => Compartment::System,
            _ => return Err(ScopeError::Compartment),
        };
        let (resource, letters) = rest.rsplit_once('.').ok_or(ScopeError::Resource)?;
        if resource.is_empty() || resource.contains('/') {
            return Err(ScopeError::Resource);
        }
        let operations = Operations::parse(letters).ok_or(ScopeError::Permissions)?;
        Ok(Self {
            compartment,
            resource: resource.to_owned(),
            operations,
            constraint,
        })
    }
}

/// A SMART scope that asks for no resource: an identity, a launch
/// context or a refresh token (SMART App Launch 2.2.0, Scopes and Launch
/// Context).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// `openid`: who the user is, by OpenID Connect.
    OpenId,
    /// `profile`: the user's default profile claims, such as their name
    /// and picture (OpenID Connect Core 1.0, § 5.4).
    Profile,
    /// `fhirUser`: the user as a FHIR resource.
    FhirUser,
    /// `launch`: the context of the EHR that launched the app.
    Launch,
    /// `launch/<resource>`: a resource in context at launch, with the
    /// role it plays when the scope names one.
    LaunchContext {
        /// The resource type as the scope writes it, in lower case
        /// (`patient`, `encounter`).
        resource: String,
        /// The role after `?role=`, when the scope names one.
        role: Option<String>,
    },
    /// `online_access`: a refresh token while the user stays online.
    OnlineAccess,
    /// `offline_access`: a refresh token that outlasts the session.
    OfflineAccess,
}

impl Request {
    /// Reads `raw` as a scope that asks for no resource; `None` for any
    /// other scope.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrohealth_ui::view::scope::Request;
    ///
    /// assert_eq!(Some(Request::OpenId), Request::parse("openid"));
    /// let patient = Request::LaunchContext { resource: "patient".into(), role: None };
    /// assert_eq!(Some(patient), Request::parse("launch/patient"));
    /// assert_eq!(None, Request::parse("patient/Observation.rs"));
    /// ```
    #[must_use]
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "openid" => return Some(Self::OpenId),
            "profile" => return Some(Self::Profile),
            "fhirUser" => return Some(Self::FhirUser),
            "launch" => return Some(Self::Launch),
            "online_access" => return Some(Self::OnlineAccess),
            "offline_access" => return Some(Self::OfflineAccess),
            _ => {}
        }
        let context = raw.strip_prefix("launch/")?;
        let (resource, role) = match context.split_once('?') {
            Some((resource, query)) => (resource, Some(query.strip_prefix("role=")?)),
            None => (context, None),
        };
        let plain = |text: &str| !text.is_empty() && !text.contains(['/', '?', '&', '=']);
        (plain(resource) && role.is_none_or(plain)).then(|| Self::LaunchContext {
            resource: resource.to_owned(),
            role: role.map(str::to_owned),
        })
    }

    /// What the scope asks for, in `locale`.
    #[must_use]
    pub fn words(&self, locale: &Locale) -> Word {
        match self {
            Self::OpenId => tr(locale, Text::ScopeOpenid),
            Self::Profile => tr(locale, Text::ScopeProfile),
            Self::FhirUser => tr(locale, Text::ScopeFhirUser),
            Self::Launch => tr(locale, Text::ScopeLaunch),
            Self::LaunchContext {
                resource,
                role: None,
            } => tr_with(
                locale,
                Text::ScopeLaunchContext,
                &[("what", resource.clone().into())],
            ),
            Self::LaunchContext {
                resource,
                role: Some(role),
            } => tr_with(
                locale,
                Text::ScopeLaunchRole,
                &[
                    ("what", resource.clone().into()),
                    ("label", role.clone().into()),
                ],
            ),
            Self::OnlineAccess => tr(locale, Text::ScopeOnlineAccess),
            Self::OfflineAccess => tr(locale, Text::ScopeOfflineAccess),
        }
    }
}

/// How the console reads one scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Grant {
    /// A scope the console reads.
    Granted {
        /// The scope.
        scope: Scope,
        /// Its resource in words, such as "Compositions of any template".
        resource: String,
        /// Whether the grant is broad, which the warn badge says.
        broad: bool,
    },
    /// A scope that asks for no resource, which says what it asks for.
    Request(Request),
    /// A scope the console refuses: why, in one sentence, such as what the
    /// grammar expected ([`ScopeError::expected`]) or what the server
    /// takes instead.
    Refused(Word),
}

/// One scope as the grant list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeGrant {
    /// The scope as it was granted or typed.
    pub raw: String,
    /// How the console reads it.
    pub grant: Grant,
}

impl ScopeGrant {
    /// Reads the scope `raw`: a [`Request`] as itself, a clinical data
    /// scope as granted with the resource words and the breadth
    /// `describe` gives it, and anything else as refused with what the
    /// grammar expected, in `locale`.
    ///
    /// # Examples
    ///
    /// ```
    /// use ferrohealth_ui::i18n::Locale;
    /// use ferrohealth_ui::view::scope::{Grant, ScopeGrant};
    ///
    /// let en = Locale::english();
    /// let read = ScopeGrant::read(&en, "fhirUser", |_| (String::new(), false));
    /// assert!(matches!(read.grant, Grant::Request(_)));
    /// let read = ScopeGrant::read(&en, "patient/Observation.rs", |_| ("Observations".into(), false));
    /// assert!(matches!(read.grant, Grant::Granted { .. }));
    /// ```
    #[must_use]
    pub fn read(
        locale: &Locale,
        raw: impl Into<String>,
        describe: impl FnOnce(&Scope) -> (String, bool),
    ) -> Self {
        let raw = raw.into();
        let grant = match Request::parse(&raw) {
            Some(request) => Grant::Request(request),
            None => match Scope::parse(&raw) {
                Ok(scope) => {
                    let (resource, broad) = describe(&scope);
                    Grant::Granted {
                        scope,
                        resource,
                        broad,
                    }
                }
                Err(error) => Grant::Refused(error.expected(locale)),
            },
        };
        Self { raw, grant }
    }
}

/// Returns `text` with a break opportunity after each slash, dot, question
/// mark, ampersand, equals sign and bar, so a long scope breaks there.
fn breakable(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        let mut one = [0_u8; 4];
        out.push_str(&Html::text(c.encode_utf8(&mut one)).to_string());
        if matches!(c, '/' | '.' | '?' | '&' | '=' | '|') {
            out.push_str("<wbr>");
        }
    }
    out
}

/// Returns the raw scope `raw` in the code face, breakable.
fn raw_view(raw: &str) -> String {
    format!("<code class=\"fui-scope-raw\">{}</code>", breakable(raw))
}

/// Returns one operation mark in `locale`: ticked when permitted, struck
/// through as `<del>` with the hidden words "not permitted" when not.
fn operation(locale: &Locale, operation: Operation, permitted: bool) -> String {
    let words = operation.words(locale);
    if permitted {
        format!("<span class=\"fui-scope-op is-on\">{words}</span>")
    } else {
        format!(
            "<del class=\"fui-scope-op\">{words}<span class=\"fui-sr-only\">{}</span></del>",
            tr(locale, Text::ScopeNotPermitted).map(|words| format!(", {words}"))
        )
    }
}

/// Returns one scope of the list, in `locale`.
fn grant_view(locale: &Locale, grant: &ScopeGrant) -> String {
    let raw = raw_view(&grant.raw);
    match &grant.grant {
        Grant::Granted {
            scope,
            resource,
            broad,
        } => {
            let marks: String = Operation::ALL
                .into_iter()
                .map(|each| operation(locale, each, scope.operations.permits(each)))
                .collect();
            let reads = scope
                .compartment
                .words(locale)
                .map(|words| format!("{words} · {resource}"));
            let constraint = scope
                .constraint
                .as_deref()
                .map_or_else(String::new, |constraint| {
                    format!(
                        "<span class=\"fui-scope-constraint\">{} <code>{}</code></span>",
                        tr(locale, Text::ScopeOnlyWhere),
                        breakable(constraint)
                    )
                });
            let badge = if *broad {
                format!(
                    "<span class=\"{}\">{}</span>",
                    Tone::Warn.badge(),
                    tr(locale, Text::ScopeBroad)
                )
            } else {
                String::new()
            };
            format!(
                "<li class=\"fui-scope\">{raw}<span class=\"fui-scope-reads\"><span>{reads}</span><span class=\"fui-scope-ops\">{marks}</span>{constraint}{badge}</span></li>"
            )
        }
        Grant::Request(request) => format!(
            "<li class=\"fui-scope\">{raw}<span class=\"fui-scope-reads\"><span>{}</span></span></li>",
            request.words(locale)
        ),
        Grant::Refused(why) => format!(
            "<li class=\"fui-scope is-refused\">{raw}<span class=\"fui-scope-reads\"><span class=\"fui-small fui-tone-bad\">{} {why}</span></span></li>",
            icon(Glyph::CIRCLE_X, "14")
        ),
    }
}

/// Returns the scopes `grants` as one list named `label`, such as "Scopes
/// the CDR supports", in `locale`.
#[must_use]
pub fn scope_grants(locale: &Locale, label: &str, grants: Vec<ScopeGrant>) -> Html {
    let items: String = grants
        .into_iter()
        .map(|grant| grant_view(locale, &grant))
        .collect();
    Html::raw(format!(
        "<ul aria-label=\"{}\" class=\"fui-scopes\">{items}</ul>",
        Html::text(label)
    ))
}
