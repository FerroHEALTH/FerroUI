// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's assets served by a console.
//!
//! The stylesheet, the brand themes and the script modules are served under
//! one path that names the library's version, so a browser caches each
//! file for good and a new release is a new path.
//!
//! A console mounts [`axum_router::router`] at [`PREFIX`] and hands the shell
//! [`Assets`], which writes the `<link>` and `<script>` tags. The files are
//! the crate's constants: nothing is read from disk. The paths carry no
//! user input and every answer is `Cache-Control: public, max-age=31536000,
//! immutable` (RFC 9111 §5.2.2). A console without axum reads the same
//! bytes from [`crate::style`] and [`crate::script`] and serves them
//! itself. No specification governs the layout: our own design.

use std::fmt;

use crate::style::Brand;

/// The library's version, which the asset paths carry.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Where a console mounts the asset router: the directory the shell's
/// tags point under.
pub const PREFIX: &str = "/ferrohealth-ui";

/// The paths the shell writes for the stylesheet, the theme and the loader.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assets {
    /// The directory the assets are served from, such as
    /// `/ferrohealth-ui/0.6.0`.
    pub base: String,
    /// The brand whose theme the page links.
    pub brand: Brand,
}

impl Assets {
    /// The assets of `brand` under [`PREFIX`] and the library's version.
    #[must_use]
    pub fn of(brand: Brand) -> Self {
        Self {
            base: format!("{PREFIX}/{VERSION}"),
            brand,
        }
    }

    /// The stylesheet's path.
    #[must_use]
    pub fn stylesheet(&self) -> String {
        format!("{}/ferrohealth-ui.css", self.base)
    }

    /// The brand's theme file's path.
    #[must_use]
    pub fn theme(&self) -> String {
        format!("{}/themes/{}.css", self.base, self.brand.key())
    }

    /// The loader's path, the one script a page loads.
    #[must_use]
    pub fn loader(&self) -> String {
        format!("{}/scripts/{}", self.base, crate::script::LOADER.file)
    }
}

impl fmt::Display for Assets {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.base)
    }
}

/// The axum router over the assets.
#[cfg(feature = "axum")]
pub mod axum_router {
    use axum::Router;
    use axum::extract::Path;
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use http::header::{CACHE_CONTROL, CONTENT_TYPE};
    use http::{HeaderValue, StatusCode};

    use super::VERSION;
    use crate::style::{Brand, STYLESHEET};

    /// A year, the longest a browser is asked to keep an asset.
    const IMMUTABLE: &str = "public, max-age=31536000, immutable";

    /// An asset's answer.
    fn asset(content_type: &'static str, text: &'static str) -> Response {
        (
            [
                (CONTENT_TYPE, HeaderValue::from_static(content_type)),
                (CACHE_CONTROL, HeaderValue::from_static(IMMUTABLE)),
            ],
            text,
        )
            .into_response()
    }

    /// Whether `version` is this library's; a path under another has
    /// nothing there.
    fn versioned(version: &str) -> bool {
        version == VERSION
    }

    async fn stylesheet(Path(version): Path<String>) -> Response {
        if !versioned(&version) {
            return StatusCode::NOT_FOUND.into_response();
        }
        asset("text/css; charset=utf-8", STYLESHEET)
    }

    async fn theme(Path((version, file)): Path<(String, String)>) -> Response {
        if !versioned(&version) {
            return StatusCode::NOT_FOUND.into_response();
        }
        match file.strip_suffix(".css").and_then(Brand::parse) {
            Some(brand) => asset("text/css; charset=utf-8", brand.theme()),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    async fn script(Path((version, file)): Path<(String, String)>) -> Response {
        if !versioned(&version) {
            return StatusCode::NOT_FOUND.into_response();
        }
        match crate::script::module(&file) {
            Some(module) => asset("text/javascript; charset=utf-8", module.text),
            None => StatusCode::NOT_FOUND.into_response(),
        }
    }

    /// The router serving the library's assets, to mount at
    /// [`super::PREFIX`]: `/{version}/ferrohealth-ui.css`,
    /// `/{version}/themes/{brand}.css` and `/{version}/scripts/{file}`.
    pub fn router<S: Clone + Send + Sync + 'static>() -> Router<S> {
        Router::new()
            .route("/{version}/ferrohealth-ui.css", get(stylesheet))
            .route("/{version}/themes/{file}", get(theme))
            .route("/{version}/scripts/{file}", get(script))
    }
}

#[cfg(test)]
mod tests {
    use super::{Assets, PREFIX, VERSION};
    use crate::style::Brand;

    #[test]
    fn the_paths_name_the_version_and_the_brand() {
        let assets = Assets::of(Brand::Ehr);
        assert_eq!(
            format!("{PREFIX}/{VERSION}/ferrohealth-ui.css"),
            assets.stylesheet()
        );
        assert_eq!(
            format!("{PREFIX}/{VERSION}/themes/ferroehr.css"),
            assets.theme()
        );
        assert_eq!(
            format!("{PREFIX}/{VERSION}/scripts/fui.js"),
            assets.loader()
        );
    }

    #[cfg(feature = "axum")]
    #[tokio::test]
    async fn the_router_serves_each_asset_under_this_version_alone() {
        use axum::body::Body;
        use http::{Request, StatusCode};
        use tower::ServiceExt as _;

        let app: axum::Router = axum::Router::new().nest(PREFIX, super::axum_router::router());
        for (path, status) in [
            (
                format!("{PREFIX}/{VERSION}/ferrohealth-ui.css"),
                StatusCode::OK,
            ),
            (
                format!("{PREFIX}/{VERSION}/themes/ferropix.css"),
                StatusCode::OK,
            ),
            (
                format!("{PREFIX}/{VERSION}/themes/none.css"),
                StatusCode::NOT_FOUND,
            ),
            (format!("{PREFIX}/{VERSION}/scripts/fui.js"), StatusCode::OK),
            (
                format!("{PREFIX}/{VERSION}/scripts/none.js"),
                StatusCode::NOT_FOUND,
            ),
            (
                format!("{PREFIX}/0.0.0/ferrohealth-ui.css"),
                StatusCode::NOT_FOUND,
            ),
        ] {
            let request = Request::get(&path).body(Body::empty()).expect("a request");
            let answer = app.clone().oneshot(request).await.expect("an answer");
            assert_eq!(status, answer.status(), "{path}");
        }
    }
}
