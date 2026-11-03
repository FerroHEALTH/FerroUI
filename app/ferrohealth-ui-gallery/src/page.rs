// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The page a route draws, and what every page names the same way: the
//! gallery's version, its footer, the document title, a brand's glyph and
//! the page for a path the gallery does not serve.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::empty::empty_state;
use ferrohealth_ui::view::shell::Footer;
use ferrohealth_ui::view::toast::Flash;

use crate::locale::Msg;
use crate::visit::Visit;

/// The gallery's version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// How a page is framed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Frame {
    /// In the brand's shell: the top bar, the navigation, the footer.
    Shell,
    /// Centred above the footer, as the sign-in and a fault are.
    Centred,
}

/// A page a handler drew: its title words, its frame and its content.
#[derive(Debug)]
pub struct Page {
    /// The page's own words in the document title; the brand and the
    /// gallery follow them.
    pub title: String,
    /// How the page is framed.
    pub frame: Frame,
    /// The page's content.
    pub content: Html,
    /// The answer's status.
    pub status: http::StatusCode,
    /// The toasts the page arrives with, such as the outcome of the action
    /// that led to it.
    pub flash: Vec<Flash>,
}

impl Page {
    /// A page in the brand's shell.
    #[must_use]
    pub fn shell(title: String, content: Html) -> Self {
        Self {
            title,
            frame: Frame::Shell,
            content,
            status: http::StatusCode::OK,
            flash: Vec::new(),
        }
    }

    /// A page centred above the footer.
    #[must_use]
    pub fn centred(title: String, content: Html) -> Self {
        Self {
            title,
            frame: Frame::Centred,
            content,
            status: http::StatusCode::OK,
            flash: Vec::new(),
        }
    }

    /// This page with the toasts `flash`.
    #[must_use]
    pub fn with_flash(mut self, flash: Vec<Flash>) -> Self {
        self.flash = flash;
        self
    }

    /// This page answered with `status`.
    #[must_use]
    pub fn with_status(mut self, status: http::StatusCode) -> Self {
        self.status = status;
        self
    }
}

/// The gallery's name, as its footer and titles name it.
fn product(visit: &Visit) -> String {
    visit.tr(Msg::Product)
}

/// The footer every gallery page carries.
#[must_use]
pub fn footer(visit: &Visit) -> Footer {
    Footer {
        product: product(visit).into(),
        version: VERSION.into(),
        about: "/".into(),
    }
}

/// The document title of the page `name` in the visit's brand, the
/// gallery's for a path under no brand.
#[must_use]
pub fn title(visit: &Visit, name: &str) -> String {
    let product = product(visit);
    if name.is_empty() {
        return product;
    }
    let under = if visit
        .path
        .split('/')
        .any(|part| Brand::parse(part).is_some())
    {
        visit.brand.product().to_owned()
    } else {
        visit.tr(Msg::Gallery)
    };
    format!("{name} · {under} · {product}")
}

/// The glyph a brand's mark draws in the gallery.
#[must_use]
pub const fn brand_glyph(brand: Brand) -> Glyph {
    match brand {
        Brand::Bridge => Glyph::LAYERS,
        Brand::Fed => Glyph::USERS,
        Brand::Ehr => Glyph::HEART_PULSE,
        Brand::Term => Glyph::LIST_CHECKS,
        Brand::Pix => Glyph::LINK,
    }
}

/// The page for a path the gallery does not serve: `404`, and the way back.
#[must_use]
pub fn not_found(visit: &Visit) -> Page {
    let content = empty_state(
        Glyph::COMPASS,
        visit.tr(Msg::NotFound),
        visit.tr(Msg::NotFoundBody),
        Html::raw(format!(
            "<a href=\"/\" class=\"fui-button fui-button-primary\">{}{}</a>",
            icon(Glyph::ARROW_LEFT, "16"),
            Html::text(visit.tr(Msg::BackGallery))
        )),
    );
    Page::centred(
        visit.tr(Msg::NotFound),
        Html::raw(format!("<div class=\"fui-card\">{content}</div>")),
    )
    .with_status(http::StatusCode::NOT_FOUND)
}
