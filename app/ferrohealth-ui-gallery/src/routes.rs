// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's pages as lazy routes, as a console splits its code.
//!
//! Each page's code loads from chunks of its own under `cargo leptos build
//! --split` (the Leptos book, "Code Splitting"), and [`ROUTES`] tells the
//! library which link loads which route ahead of a visit
//! ([`ferrohealth_ui::widget::prefetch`]).

use ferrohealth_ui::widget::prefetch::Route;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

/// Declares the lazy route `$name` that draws the page component `$page`.
macro_rules! lazy_page {
    ($(#[$doc:meta])* $name:ident, $page:path) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        #[lazy_route]
        impl LazyRoute for $name {
            fn data() -> Self {
                Self
            }

            fn view(_this: Self) -> AnyView {
                view! { <$page /> }.into_any()
            }
        }
    };
}

lazy_page!(
    /// The lazy route of [`crate::pages::sign_in::SignInPage`].
    LSignInPage,
    crate::pages::sign_in::SignInPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::templates::FaultPage`].
    LFaultPage,
    crate::pages::templates::FaultPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::overview::OverviewPage`].
    LOverviewPage,
    crate::pages::overview::OverviewPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::forms::FormsPage`].
    LFormsPage,
    crate::pages::forms::FormsPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::tables::TablesPage`].
    LTablesPage,
    crate::pages::tables::TablesPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::code::CodePage`].
    LCodePage,
    crate::pages::code::CodePage
);

lazy_page!(
    /// The lazy route of [`crate::pages::configuration::ConfigurationPage`].
    LConfigurationPage,
    crate::pages::configuration::ConfigurationPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::configuration::ProvisionedPage`].
    LProvisionedPage,
    crate::pages::configuration::ProvisionedPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::overlays::OverlaysPage`].
    LOverlaysPage,
    crate::pages::overlays::OverlaysPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::overlays::AccessPage`].
    LAccessPage,
    crate::pages::overlays::AccessPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::charts::ChartsPage`].
    LChartsPage,
    crate::pages::charts::ChartsPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::layouts::LayoutsPage`].
    LLayoutsPage,
    crate::pages::layouts::LayoutsPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::operations::OperationsPage`].
    LOperationsPage,
    crate::pages::operations::OperationsPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::clinical::ClinicalPage`].
    LClinicalPage,
    crate::pages::clinical::ClinicalPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::terminology::TerminologyPage`].
    LTerminologyPage,
    crate::pages::terminology::TerminologyPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::federation::FederationPage`].
    LFederationPage,
    crate::pages::federation::FederationPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::find::FindPage`].
    LFindPage,
    crate::pages::find::FindPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::federation::FederationPart`].
    LFederationPart,
    crate::pages::federation::FederationPart
);

lazy_page!(
    /// The lazy route of [`crate::pages::preferences::PreferencesPage`].
    LPreferencesPage,
    crate::pages::preferences::PreferencesPage
);

lazy_page!(
    /// The lazy route of [`crate::pages::templates::TemplatePage`].
    LTemplatePage,
    crate::pages::templates::TemplatePage
);

lazy_page!(
    /// The lazy route of [`crate::pages::trees::TreesPage`].
    LTreesPage,
    crate::pages::trees::TreesPage
);

/// Every lazy route by its path; the brand's pages load once the first page
/// is idle, as a console's main navigation does.
pub static ROUTES: &[Route] = &[
    Route {
        path: "/sign-in/:brand/:variant",
        preload: || Box::pin(LSignInPage::preload()),
        idle: false,
    },
    Route {
        path: "/fault/:brand",
        preload: || Box::pin(LFaultPage::preload()),
        idle: false,
    },
    Route {
        path: "/:brand",
        preload: || Box::pin(LOverviewPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/forms",
        preload: || Box::pin(LFormsPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/tables",
        preload: || Box::pin(LTablesPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/code",
        preload: || Box::pin(LCodePage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/configuration",
        preload: || Box::pin(LConfigurationPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/configuration/provisioned",
        preload: || Box::pin(LProvisionedPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/overlays",
        preload: || Box::pin(LOverlaysPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/overlays/access",
        preload: || Box::pin(LAccessPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/charts",
        preload: || Box::pin(LChartsPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/layouts",
        preload: || Box::pin(LLayoutsPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/operations",
        preload: || Box::pin(LOperationsPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/clinical",
        preload: || Box::pin(LClinicalPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/terminology",
        preload: || Box::pin(LTerminologyPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/federation",
        preload: || Box::pin(LFederationPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/find",
        preload: || Box::pin(LFindPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/federation/:part",
        preload: || Box::pin(LFederationPart::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/preferences",
        preload: || Box::pin(LPreferencesPage::preload()),
        idle: true,
    },
    Route {
        path: "/:brand/preferences/:section",
        preload: || Box::pin(LPreferencesPage::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/templates/:variant",
        preload: || Box::pin(LTemplatePage::preload()),
        idle: false,
    },
    Route {
        path: "/:brand/trees",
        preload: || Box::pin(LTreesPage::preload()),
        idle: true,
    },
];
