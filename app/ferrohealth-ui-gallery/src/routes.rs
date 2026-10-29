// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's pages by the path that names each, and the handler that
//! draws the one a request names.
//!
//! A page is one function of its module, `page(&Visit) -> Page`, which reads
//! the request and draws the page once, whole; [`draw`] picks it by path.

use crate::app::Page as Drawn;
use crate::visit::Visit;

/// A gallery page, as its path names it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    /// `/sign-in/:brand/:variant`.
    SignIn,
    /// `/fault/:brand`.
    Fault,
    /// `/:brand/templates/:variant`.
    Template,
    /// `/:brand`.
    Overview,
    /// `/:brand/forms`.
    Forms,
    /// `/:brand/tables`.
    Tables,
    /// `/:brand/code`.
    Code,
    /// `/:brand/configuration`.
    Configuration,
    /// `/:brand/configuration/provisioned`.
    Provisioned,
    /// `/:brand/preferences` and its sections.
    Preferences,
    /// `/:brand/find`.
    Find,
    /// `/:brand/charts`.
    Charts,
    /// `/:brand/layouts`.
    Layouts,
    /// `/:brand/trees`.
    Trees,
    /// `/:brand/clinical`.
    Clinical,
    /// `/:brand/terminology`.
    Terminology,
    /// `/:brand/operations`.
    Operations,
    /// `/:brand/federation`.
    Federation,
    /// `/:brand/federation/:part`.
    FederationPart,
    /// `/:brand/overlays`.
    Overlays,
    /// `/:brand/overlays/access`.
    Access,
}

impl Page {
    /// The page the address path `path` names, if it names one.
    #[must_use]
    pub fn of(path: &str) -> Option<Self> {
        let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
        Some(match parts.as_slice() {
            ["sign-in", _, _] => Self::SignIn,
            ["fault", _] => Self::Fault,
            [_] => Self::Overview,
            [_, "templates", _] => Self::Template,
            [_, "forms"] => Self::Forms,
            [_, "tables"] => Self::Tables,
            [_, "code"] => Self::Code,
            [_, "configuration"] => Self::Configuration,
            [_, "configuration", "provisioned"] => Self::Provisioned,
            [_, "preferences"] | [_, "preferences", _] => Self::Preferences,
            [_, "find"] => Self::Find,
            [_, "charts"] => Self::Charts,
            [_, "layouts"] => Self::Layouts,
            [_, "trees"] => Self::Trees,
            [_, "clinical"] => Self::Clinical,
            [_, "terminology"] => Self::Terminology,
            [_, "operations"] => Self::Operations,
            [_, "federation"] => Self::Federation,
            [_, "federation", _] => Self::FederationPart,
            [_, "overlays"] => Self::Overlays,
            [_, "overlays", "access"] => Self::Access,
            _ => return None,
        })
    }
}

/// Draws the page `visit` names, or the gallery's not-found page.
#[must_use]
pub fn draw(visit: &Visit) -> Drawn {
    use crate::pages;
    let Some(page) = Page::of(&visit.path) else {
        return if visit.path == "/" {
            crate::app::index(visit)
        } else {
            crate::app::not_found(visit)
        };
    };
    if page != Page::SignIn
        && page != Page::Fault
        && visit
            .segment(0)
            .and_then(ferrohealth_ui::style::Brand::parse)
            .is_none()
    {
        return crate::app::not_found(visit);
    }
    match page {
        Page::SignIn => pages::sign_in::page(visit),
        Page::Layouts => pages::layouts::page(visit),
        Page::Clinical => pages::clinical::page(visit),
        Page::Operations => pages::operations::page(visit),
        Page::Trees => pages::trees::page(visit),
        Page::Configuration => pages::configuration::page(visit),
        Page::Provisioned => pages::configuration::provisioned(visit),
        Page::Overview => pages::overview::page(visit),
        Page::Tables => pages::tables::page(visit),
        Page::Forms => pages::forms::page(visit),
        Page::Code => pages::code::page(visit),
        Page::Preferences => pages::preferences::page(visit),
        Page::Find => pages::find::page(visit),
        Page::Fault => pages::templates::fault(visit),
        Page::Template => pages::templates::page(visit),
        _ => crate::app::not_found(visit),
    }
}

/// Draws the page `visit` names with the answer to the form body `body`
/// posted to it, for a page that takes a post; `None` for any other.
#[must_use]
pub fn posted(visit: &Visit, body: &str) -> Option<Drawn> {
    match Page::of(&visit.path)? {
        Page::Forms => Some(crate::pages::forms::posted(visit, body)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Page;

    #[test]
    fn every_path_names_its_page() {
        assert_eq!(Some(Page::Access), Page::of("/ferrofed/overlays/access"));
        assert_eq!(Some(Page::Overview), Page::of("/ferrofed/"));
        assert_eq!(Some(Page::SignIn), Page::of("/sign-in/ferrofed/basic"));
        assert_eq!(
            Some(Page::FederationPart),
            Page::of("/ferrofed/federation/node-a")
        );
        assert_eq!(None, Page::of("/ferrofed/no/such/page"));
        assert_eq!(None, Page::of("/"));
    }
}
