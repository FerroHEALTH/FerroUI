// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! What a handler knows of the request it draws a page for: the path, the
//! query, the cookies with the preferences they keep, the locale they name
//! and the brand the path names.
//!
//! Every page is drawn from a [`Visit`], once, whole, with the state the
//! server read: the theme, the listing's position, the open tree nodes. No
//! specification governs the gallery's request model: our own design.

use ferrohealth_ui::i18n::{Arg, Locale};
use ferrohealth_ui::query::Query;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::theme::Preferences;

use crate::locale::{Msg, gallery_locale, tr_in};

/// One request, as a page reads it.
#[derive(Debug, Clone)]
pub struct Visit {
    /// The address path, such as `/ferrofed/tables`.
    pub path: String,
    /// The address query.
    pub query: Query,
    /// The `Cookie` header's value, empty for none.
    pub cookies: String,
    /// The preferences the cookies keep.
    pub preferences: Preferences,
    /// The locale the language preference names.
    pub locale: Locale,
    /// The brand the path names, FerroBRIDGE for a path that names none.
    pub brand: Brand,
}

impl Visit {
    /// The visit of the path `path`, the query text `query` (after the
    /// `?`, possibly empty) and the `Cookie` header value `cookies`.
    #[must_use]
    pub fn new(path: &str, query: &str, cookies: &str) -> Self {
        let preferences = crate::app::preferences_of(cookies);
        let locale = gallery_locale(preferences.language.tag().unwrap_or_default());
        Self {
            path: path.to_owned(),
            query: Query::parse(query),
            cookies: cookies.to_owned(),
            brand: crate::app::brand_of(path),
            preferences,
            locale,
        }
    }

    /// The visit of the request `parts`.
    #[must_use]
    pub fn of(parts: &http::request::Parts) -> Self {
        let cookies = parts
            .headers
            .get(http::header::COOKIE)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        Self::new(
            parts.uri.path(),
            parts.uri.query().unwrap_or_default(),
            cookies,
        )
    }

    /// The first value of the query parameter `name`.
    #[must_use]
    pub fn param(&self, name: &str) -> Option<&str> {
        self.query.get_str(name)
    }

    /// The path segment at `index`, counted from the first after the root.
    #[must_use]
    pub fn segment(&self, index: usize) -> Option<&str> {
        self.path
            .split('/')
            .filter(|part| !part.is_empty())
            .nth(index)
    }

    /// The path with its query, as a link back to this very page.
    #[must_use]
    pub fn here(&self) -> String {
        format!("{}{}", self.path, self.query.to_query_string())
    }

    /// The gallery's message `msg` in the visit's locale.
    #[must_use]
    pub fn tr(&self, msg: Msg) -> String {
        tr_in(&self.locale, msg, &[])
    }

    /// The gallery's message `msg` in the visit's locale, with the
    /// placeables `args` filled in.
    #[must_use]
    pub fn tr_with(&self, msg: Msg, args: &[(&'static str, Arg)]) -> String {
        tr_in(&self.locale, msg, args)
    }
}

#[cfg(test)]
mod tests {
    use ferrohealth_ui::style::Brand;

    use super::Visit;

    #[test]
    fn a_visit_reads_its_brand_query_and_locale() {
        let visit = Visit::new("/ferroehr/tables", "page=2&size=50", "gallery-language=qps");
        assert_eq!(Brand::Ehr, visit.brand);
        assert_eq!(Some("2"), visit.param("page"));
        assert_eq!(Some("tables"), visit.segment(1));
        assert_eq!("qps", visit.locale.tag());
        assert_eq!("/ferroehr/tables?page=2&size=50", visit.here());
        assert_eq!("/ferroehr", Visit::new("/ferroehr", "", "").here());
    }
}
