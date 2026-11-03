// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! What a request says to the gallery: its own routes and cookies, and
//! the brand, the theme, the preferences, the paging and the rail its path
//! and its `Cookie` header name.

use ferrohealth_ui::style::Brand;
use ferrohealth_ui::table::model::Paging;
use ferrohealth_ui::theme::{Preferences, Theme, cookie};

/// The path of the route every preference posts to, its value under its
/// cookie's name: Your preferences, the user menu's theme and the
/// overview's density.
pub const PREFERENCE_ROUTE: &str = "/preference";

/// The path of the route Reset to defaults posts to.
pub const RESET_ROUTE: &str = "/preferences/reset";

/// What the gallery puts before each preference's cookie name
/// ([`ferrohealth_ui::theme::Preference::COOKIE`]), so `gallery-theme` keeps
/// the theme.
pub const COOKIE_PREFIX: &str = "gallery-";

/// The gallery's own preferences, shown as a console's own rows.
///
/// In FerroPIX's Appearance: how long a revealed identifier stays and
/// whether leaving the page hides it. In Language and region: FerroTERM's
/// language concepts are asked for in and the rows a listing shows.
pub const OWN_PREFERENCES: [&str; 4] = ["reveal", "hide", "concepts", "size"];

/// The path of the route a session listing's pager posts its page and size
/// to.
pub const PAGING_ROUTE: &str = "/paging";

/// The listings that keep their position in the gallery's session, each
/// under the cookie its key names after [`PAGING_COOKIE`].
pub const PAGED_LISTINGS: [&str; 2] = ["files", "ehrs"];

/// The prefix of the cookie a session listing's position is kept in, for
/// the browser's session alone and never in a URL.
pub const PAGING_COOKIE: &str = "gallery-paging-";

/// The path of the route the code editor's form posts its draft to, which
/// answers with the draft it received.
pub const DRAFT_ROUTE: &str = "/draft";

/// The path under which an answer riding its query's form is exported, as
/// `/export/csv` or `/export/json`.
pub const EXPORT_ROUTE: &str = "/export";

/// The path of the route the topology's Probe now posts to, which sends
/// the browser back where it came from.
pub const PROBE_ROUTE: &str = "/probe";

/// The path of the route a member's removal posts to, which returns to the
/// page naming the member removed.
pub const REMOVE_ROUTE: &str = "/remove-member";

/// The path of the route the sidebar's Collapse and Expand post to.
pub const SIDEBAR_ROUTE: &str = "/sidebar";

/// The name of the cookie the sidebar's state is kept in.
pub const SIDEBAR_COOKIE: &str = "gallery-sidebar";

/// The path under which a brand's Sign out posts.
pub const SIGN_OUT_ROUTE: &str = "/sign-out";

/// The brand a request path names: the first segment that is a brand's key,
/// FerroBRIDGE for a path that names none.
#[must_use]
pub fn brand_of(path: &str) -> Brand {
    path.split('/')
        .find_map(Brand::parse)
        .unwrap_or(Brand::Bridge)
}

/// The theme the `Cookie` header value `cookies` names, the system theme for
/// none.
#[must_use]
pub fn theme_of(cookies: &str) -> Theme {
    Preferences::read(cookies, COOKIE_PREFIX).theme
}

/// The value of the gallery's own preference `name`, one of
/// [`OWN_PREFERENCES`], that the `Cookie` header value `cookies` keeps.
#[must_use]
pub fn own_of<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    cookie(cookies, &format!("{COOKIE_PREFIX}{name}"))
}

/// The paging of the session listing `listing` that the `Cookie` header
/// value `cookies` keeps, as `<page>.<size>`, the first page for none.
#[must_use]
pub fn paging_of(cookies: &str, listing: &str) -> Paging {
    let kept = cookie(cookies, &format!("{PAGING_COOKIE}{listing}"))
        .and_then(|value| value.split_once('.'));
    Paging::read(kept.map(|(page, _)| page), kept.map(|(_, size)| size))
}

/// Whether the `Cookie` header value `cookies` asks for the sidebar's rail.
#[must_use]
pub fn rail_of(cookies: &str) -> bool {
    cookie(cookies, SIDEBAR_COOKIE) == Some("rail")
}

#[cfg(test)]
mod tests {
    use ferrohealth_ui::style::Brand;
    use ferrohealth_ui::theme::Theme;

    use super::{brand_of, theme_of};

    #[test]
    fn the_brand_and_the_theme_are_read_from_the_request() {
        assert_eq!(Brand::Ehr, brand_of("/ferroehr/tables"));
        assert_eq!(Brand::Term, brand_of("/sign-in/ferroterm/basic"));
        assert_eq!(Brand::Bridge, brand_of("/"));
        assert_eq!(Theme::Dark, theme_of("a=1; gallery-theme=dark"));
        assert_eq!(Theme::System, theme_of("gallery-theme=sepia"));
        assert_eq!(Theme::System, theme_of(""));
    }
}
