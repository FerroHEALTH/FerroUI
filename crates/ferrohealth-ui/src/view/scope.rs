// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The SMART scope grant (E8).
//!
//! One scope parsed: the raw scope in mono, never broken; the compartment
//! and the resource in words; the five operations as marks, a permitted
//! one ticked and an absent one struck through as `<del>` with "not
//! permitted" said after it, so the grant never rests on colour; a broad
//! grant carries the warn badge Broad access. A malformed scope is the bad
//! tone with what the grammar expected.
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
//! last dot, since a template id holds dots. Which resources a server
//! knows and which grants are broad are the console's. The look is our
//! own design.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
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

    /// The compartment in words, in the page's locale.
    #[must_use]
    pub fn words(self) -> String {
        tr(match self {
            Self::Patient => Text::ScopePatient,
            Self::User => Text::ScopeUser,
            Self::System => Text::ScopeSystem,
        })
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

    /// The operation in words, in the page's locale.
    #[must_use]
    pub fn words(self) -> String {
        tr(match self {
            Self::Create => Text::ScopeCreate,
            Self::Read => Text::ScopeRead,
            Self::Update => Text::ScopeUpdate,
            Self::Delete => Text::ScopeDelete,
            Self::Search => Text::ScopeSearch,
        })
    }
}

/// The operations a scope permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Operations([bool; 5]);

impl Operations {
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
    /// What the grammar expected, in the page's locale.
    #[must_use]
    pub fn expected(self) -> String {
        tr(match self {
            Self::Compartment => Text::ScopeExpectedCompartment,
            Self::Resource => Text::ScopeExpectedResource,
            Self::Permissions => Text::ScopeExpectedPermissions,
        })
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
    /// A scope the console refuses: why, in one sentence, such as what the
    /// grammar expected ([`ScopeError::expected`]) or what the server
    /// takes instead.
    Refused(String),
}

/// One scope as the grant list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeGrant {
    /// The scope as it was granted or typed.
    pub raw: String,
    /// How the console reads it.
    pub grant: Grant,
}

/// Returns one operation mark: ticked when permitted, struck through as
/// `<del>` with the hidden words "not permitted" when not.
fn operation(operation: Operation, permitted: bool) -> AnyView {
    let words = operation.words();
    if permitted {
        view! { <span class="fui-scope-op is-on">{words}</span> }.into_any()
    } else {
        view! {
            <del class="fui-scope-op">
                {words}
                <span class="fui-sr-only">{format!(", {}", tr(Text::ScopeNotPermitted))}</span>
            </del>
        }
        .into_any()
    }
}

/// Returns one scope of the list.
fn grant_view(grant: ScopeGrant) -> AnyView {
    let ScopeGrant { raw, grant } = grant;
    match grant {
        Grant::Granted {
            scope,
            resource,
            broad,
        } => {
            let marks = Operation::ALL
                .into_iter()
                .map(|each| operation(each, scope.operations.permits(each)))
                .collect_view();
            let reads = format!("{} · {resource}", scope.compartment.words());
            let badge = broad
                .then(|| view! { <span class=Tone::Warn.badge()>{tr(Text::ScopeBroad)}</span> });
            view! {
                <li class="fui-scope">
                    <code class="fui-scope-raw">{raw}</code>
                    <span class="fui-scope-reads">
                        <span>{reads}</span>
                        <span class="fui-scope-ops">{marks}</span>
                        {badge}
                    </span>
                </li>
            }
            .into_any()
        }
        Grant::Refused(why) => view! {
            <li class="fui-scope is-refused">
                <code class="fui-scope-raw">{raw}</code>
                <span class="fui-scope-reads">
                    <span class="fui-small fui-tone-bad">{icon(Glyph::CIRCLE_X, "14")}
                        " "
                        {why}</span>
                </span>
            </li>
        }
        .into_any(),
    }
}

/// Returns the scopes `grants` as one list named `label`, such as "Scopes
/// the CDR supports".
#[must_use]
pub fn scope_grants(label: impl Into<String>, grants: Vec<ScopeGrant>) -> AnyView {
    let label = label.into();
    let items = grants.into_iter().map(grant_view).collect_view();
    view! {
        <ul class="fui-scopes" aria-label=label>
            {items}
        </ul>
    }
    .into_any()
}
