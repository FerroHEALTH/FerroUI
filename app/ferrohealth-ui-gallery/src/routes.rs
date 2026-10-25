// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's pages as lazy routes, as a console splits its code.
//!
//! The pages load in five groups, each from chunks of its own under `cargo
//! leptos build --split` (the Leptos book, "Code Splitting"): one lazy route
//! a page made some 800 chunks, more than a browser fetches at once, and a
//! group of pages read together loads as one. Every route of a group draws
//! the page its path names. [`ROUTES`] tells the library which link loads
//! which group ahead of a visit ([`ferrohealth_ui::widget::prefetch`]).

use ferrohealth_ui::widget::prefetch::Route;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

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

/// Declares the lazy route `$name` of a group of pages, drawing each page
/// `$page` by its component `$component`.
///
/// Each group matches its own pages alone, so its chunks hold their code
/// and no other group's.
macro_rules! group {
    ($(#[$doc:meta])* $name:ident { $($page:ident => $component:path),+ $(,)? }) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name(Option<Page>);

        #[lazy_route]
        impl LazyRoute for $name {
            fn data() -> Self {
                Self(Page::of(&leptos_router::hooks::use_location().pathname.get_untracked()))
            }

            fn view(this: Self) -> AnyView {
                match this.0 {
                    $(Some(Page::$page) => view! { <$component /> }.into_any(),)+
                    _ => ().into_any(),
                }
            }
        }
    };
}

group!(
    /// The pages no navigation leads to: sign-in, the fault page and the
    /// templates.
    LAside {
        SignIn => crate::pages::sign_in::SignInPage,
        Fault => crate::pages::templates::FaultPage,
        Template => crate::pages::templates::TemplatePage,
    }
);

group!(
    /// The overview, forms, tables, code, configuration, preferences and
    /// find pages.
    LCore {
        Overview => crate::pages::overview::OverviewPage,
        Forms => crate::pages::forms::FormsPage,
        Tables => crate::pages::tables::TablesPage,
        Code => crate::pages::code::CodePage,
        Configuration => crate::pages::configuration::ConfigurationPage,
        Provisioned => crate::pages::configuration::ProvisionedPage,
        Preferences => crate::pages::preferences::PreferencesPage,
        Find => crate::pages::find::FindPage,
    }
);

group!(
    /// The charts, layouts and trees pages.
    LVisual {
        Charts => crate::pages::charts::ChartsPage,
        Layouts => crate::pages::layouts::LayoutsPage,
        Trees => crate::pages::trees::TreesPage,
    }
);

group!(
    /// The clinical, terminology and operations pages.
    LRecords {
        Clinical => crate::pages::clinical::ClinicalPage,
        Terminology => crate::pages::terminology::TerminologyPage,
        Operations => crate::pages::operations::OperationsPage,
    }
);

group!(
    /// The federation and overlays pages.
    LFederation {
        Federation => crate::pages::federation::FederationPage,
        FederationPart => crate::pages::federation::FederationPart,
        Overlays => crate::pages::overlays::OverlaysPage,
        Access => crate::pages::overlays::AccessPage,
    }
);

/// Every route by its path and the group that draws it.
///
/// The four groups of the main navigation load once the first page is idle,
/// each once, through the first of its routes; the pages aside load on a
/// hover or a focus.
pub static ROUTES: &[Route] = &[
    Route {
        path: "/sign-in/:brand/:variant",
        preload: || Box::pin(LAside::preload()),
        idle: false,
    },
    Route {
        path: "/fault/:brand",
        preload: || Box::pin(LAside::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/templates/:variant",
        preload: || Box::pin(LAside::preload()),
        idle: false,
    },
    Route {
        path: "/:brand",
        preload: || Box::pin(LCore::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/forms",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/tables",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/code",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/configuration",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/configuration/provisioned",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/preferences",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/preferences/:section",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/find",
        preload: || Box::pin(LCore::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/charts",
        preload: || Box::pin(LVisual::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/layouts",
        preload: || Box::pin(LVisual::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/trees",
        preload: || Box::pin(LVisual::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/clinical",
        preload: || Box::pin(LRecords::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/terminology",
        preload: || Box::pin(LRecords::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/operations",
        preload: || Box::pin(LRecords::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/federation",
        preload: || Box::pin(LFederation::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/federation/:part",
        preload: || Box::pin(LFederation::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/overlays",
        preload: || Box::pin(LFederation::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/overlays/access",
        preload: || Box::pin(LFederation::preload()),
        idle: false,
    },
];

#[cfg(test)]
mod tests {
    use super::{Page, ROUTES};

    #[test]
    fn every_route_names_a_page() {
        for route in ROUTES {
            let path = route
                .path
                .replace(":brand", "ferrobridge")
                .replace(":variant", "a")
                .replace(":section", "b")
                .replace(":part", "c");
            assert!(Page::of(&path).is_some(), "{}", route.path);
        }
        assert_eq!(Some(Page::Access), Page::of("/ferrofed/overlays/access"));
        assert_eq!(Some(Page::Overview), Page::of("/ferrofed/"));
        assert_eq!(None, Page::of("/ferrofed/no/such/page"));
    }
}
