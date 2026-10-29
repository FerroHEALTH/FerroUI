// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The chrome around every page, and the document around the chrome.
//!
//! [`document`] writes the whole HTML document: the head with the
//! stylesheet, the brand's theme, the loader of the script modules and the
//! console's own head parts, and the body. [`shell`] holds the skip link,
//! the top bar with the drawer button, the lockup, the search box, the
//! status pill and the user menu, the body with the navigation beside the
//! page and the dock at the foot of the main area, the toast region, and
//! the footer that names the product, its version and its maker.
//! [`centred`] is the page before sign-in and the error page of a visitor.
//!
//! The navigation itself is [`crate::view::nav`]; the user menu is
//! [`crate::view::menu`]; the toasts are [`crate::view::toast`].

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, tr};
use crate::icon::{Glyph, icon};
use crate::serve::Assets;
use crate::theme::Preferences;

/// The maker every FerroHEALTH console's footer names.
pub const MAKER: &str = "Cadasto B.V.";

/// The id of the page's main region, which the skip link leads to.
pub const MAIN_ID: &str = "main";

/// What the footer names.
#[derive(Debug, Clone)]
pub struct Footer {
    /// The product, such as "FerroBRIDGE Console".
    pub product: Cow<'static, str>,
    /// Its version.
    pub version: Cow<'static, str>,
    /// The About page every footer links to, under the console's base path.
    pub about: Cow<'static, str>,
}

impl Footer {
    /// The product and its version in the one form the footer and the
    /// About page both write, such as "FerroBRIDGE Console 0.0.12".
    #[must_use]
    pub fn release(&self) -> String {
        // NOTE: no specification governs this: our own design; the product's
        // name and its version are data, drawn as they stand in every locale.
        format!("{} {}", self.product, self.version)
    }

    /// The footer's line before its About link.
    fn line(&self) -> String {
        format!("{} · {MAKER} · ", self.release())
    }
}

/// The whole document around a page.
#[derive(Debug, Clone)]
pub struct Document {
    /// The locale the page is drawn in, written as `<html lang>` (WCAG 2.2
    /// SC 3.1.1).
    pub locale: Locale,
    /// The reader's preferences, written on `<html>` as `data-theme`,
    /// `data-density`, `data-motion` and `data-hints`, so the first paint
    /// is right (family rule 9).
    pub preferences: Preferences,
    /// The document's title.
    pub title: String,
    /// Where the library's assets are served ([`Assets::of`]).
    pub assets: Assets,
    /// The nonce a console's Content-Security-Policy names, written on the
    /// loader's tag; `None` for a policy of `script-src 'self'`, which the
    /// library needs no more than.
    pub nonce: Option<String>,
    /// The console's own head parts: its icons, its own stylesheet after
    /// the library's, a `<meta>` of its own.
    pub head: Html,
    /// The body: [`shell`] or [`centred`].
    pub body: Html,
}

#[derive(Template)]
#[template(path = "shell/document.html")]
struct DocumentView {
    lang: String,
    theme: &'static str,
    density: &'static str,
    motion: &'static str,
    hints: &'static str,
    title: String,
    stylesheet: String,
    theme_file: String,
    head: Html,
    loader: String,
    nonce: Option<String>,
    body: Html,
}

/// Returns the whole HTML document `document`.
#[must_use]
pub fn document(document: Document) -> Html {
    let Document {
        locale,
        preferences,
        title,
        assets,
        nonce,
        head,
        body,
    } = document;
    DocumentView {
        lang: locale.tag(),
        theme: preferences.theme.key(),
        density: preferences.density.key(),
        motion: preferences.motion.key(),
        hints: preferences.hints.key(),
        title,
        stylesheet: assets.stylesheet(),
        theme_file: assets.theme(),
        head,
        loader: assets.loader(),
        nonce,
        body,
    }
    .html()
}

/// The parts of a signed-in page.
#[derive(Debug)]
pub struct Shell {
    /// The top bar, from [`topbar`].
    pub topbar: Html,
    /// A banner under the top bar, such as the sandbox notice, if any.
    pub banner: Option<Html>,
    /// The navigation, from [`crate::view::nav::sidebar`].
    pub sidebar: Html,
    /// The page.
    pub content: Html,
    /// The docked bar at the foot of the main area (R4), if the page has
    /// one.
    pub dock: Option<Html>,
    /// The toast region, from [`crate::view::toast::toast_region`].
    pub toasts: Html,
    /// What the footer names.
    pub footer: Footer,
}

#[derive(Template)]
#[template(path = "shell/page.html")]
struct PageView {
    main_id: &'static str,
    skip: crate::i18n::Word,
    topbar: Html,
    banner: Option<Html>,
    sidebar: Html,
    content: Html,
    dock: Option<Html>,
    toasts: Html,
    footer_line: String,
    about: Cow<'static, str>,
    about_words: crate::i18n::Word,
}

/// Returns a signed-in page: the skip link (WCAG 2.2 SC 2.4.1), the top
/// bar, the banner, the navigation beside the page in its column, the
/// toasts and the footer.
#[must_use]
pub fn shell(locale: &Locale, shell: Shell) -> Html {
    let Shell {
        topbar,
        banner,
        sidebar,
        content,
        dock,
        toasts,
        footer,
    } = shell;
    PageView {
        main_id: MAIN_ID,
        skip: tr(locale, Text::SkipToPage),
        topbar,
        banner,
        sidebar,
        content,
        dock,
        toasts,
        footer_line: footer.line(),
        about: footer.about,
        about_words: tr(locale, Text::FooterAbout),
    }
    .html()
}

#[derive(Template)]
#[template(path = "shell/centred.html")]
struct CentredView {
    main_id: &'static str,
    content: Html,
    toasts: Html,
    footer_line: String,
    about: Cow<'static, str>,
    about_words: crate::i18n::Word,
}

/// Returns a page that centres `content` (the sign-in, a page the console
/// does not serve, a fault) above the footer.
#[must_use]
pub fn centred(locale: &Locale, content: Html, toasts: Html, footer: Footer) -> Html {
    CentredView {
        main_id: MAIN_ID,
        content,
        toasts,
        footer_line: footer.line(),
        about: footer.about,
        about_words: tr(locale, Text::FooterAbout),
    }
    .html()
}

/// Returns a mark for a product that has no drawn one: `glyph` in the
/// accent. A console with a drawn mark passes [`brand_mark`] to [`lockup`].
#[must_use]
pub fn mark(glyph: Glyph) -> Html {
    Html::raw(format!(
        "<span class=\"fui-mark\" aria-hidden=\"true\">{}</span>",
        icon(glyph, "22")
    ))
}

/// Returns a console's own drawn mark, light and dark: the image at
/// `light`, and the one at `dark` under the dark theme, both served from
/// the console's own origin and chosen by the stylesheet.
#[must_use]
pub fn brand_mark(light: &str, dark: &str) -> Html {
    Html::raw(format!(
        "<span class=\"fui-brand-mark\" aria-hidden=\"true\"><img class=\"fui-brand-light\" src=\"{}\" alt=\"\" width=\"24\" height=\"24\"><img class=\"fui-brand-dark\" src=\"{}\" alt=\"\" width=\"24\" height=\"24\"></span>",
        Html::text(light),
        Html::text(dark)
    ))
}

/// Returns the lockup that leads to `href`: the decorative `mark` beside
/// the product's `name` and its accented `word`, such as "FerroBRIDGE" and
/// "Console".
#[must_use]
pub fn lockup(
    href: impl AsRef<str>,
    mark: Html,
    name: impl AsRef<str>,
    word: impl AsRef<str>,
) -> Html {
    let mark = String::from(mark);
    Html::raw(format!(
        "<a href=\"{}\" class=\"fui-lockup\">{mark}<span class=\"fui-lockup-name\">{} <span class=\"fui-lockup-accent\">{}</span></span></a>",
        Html::text(href),
        Html::text(name),
        Html::text(word)
    ))
}

/// Returns the lockup [`lockup`] draws as plain text that leads nowhere,
/// for a page shown before sign-in.
#[must_use]
pub fn lockup_plain(mark: Html, name: impl AsRef<str>, word: impl AsRef<str>) -> Html {
    let mark = String::from(mark);
    Html::raw(format!(
        "<span class=\"fui-lockup\">{mark}<span class=\"fui-lockup-name\">{} <span class=\"fui-lockup-accent\">{}</span></span></span>",
        Html::text(name),
        Html::text(word)
    ))
}

/// What the top bar holds, start to end.
#[derive(Debug)]
pub struct TopBar {
    /// The id of the navigation the drawer button opens below 48 rem.
    pub sidebar: &'static str,
    /// The lockup, from [`lockup`].
    pub lockup: Html,
    /// The search box in the free width between the two ends, from
    /// [`crate::view::search::search_box`], if the console has one.
    pub search: Option<Html>,
    /// The end: the status pill, any control of the console's own, and the
    /// user menu, last.
    pub end: Html,
}

#[derive(Template)]
#[template(path = "shell/topbar.html")]
struct TopBarView {
    sidebar: &'static str,
    open_menu: String,
    menu_glyph: Html,
    lockup: Html,
    search: Option<Html>,
    end: Html,
}

/// Returns the top bar: the drawer button, the lockup, the search box and,
/// at the far end, the status pill and the user menu.
///
/// It carries no theme
/// control (family rule 5). Below 48 rem the search wraps onto its own row
/// under the lockup (WCAG 2.2 SC 1.4.10).
#[must_use]
pub fn topbar(locale: &Locale, bar: TopBar) -> Html {
    TopBarView {
        sidebar: bar.sidebar,
        open_menu: tr(locale, Text::OpenMenu).text().to_owned(),
        menu_glyph: icon(Glyph::MENU, "18"),
        lockup: bar.lockup,
        search: bar.search,
        end: bar.end,
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{
        Document, Footer, MAIN_ID, Shell, TopBar, centred, document, lockup, mark, shell, topbar,
    };
    use crate::html::Html;
    use crate::i18n::Locale;
    use crate::icon::Glyph;
    use crate::serve::Assets;
    use crate::style::Brand;
    use crate::theme::Preferences;

    fn footer() -> Footer {
        Footer {
            product: "FerroTEST Console".into(),
            version: "0.6.0".into(),
            about: "/about".into(),
        }
    }

    #[test]
    fn the_document_writes_the_locale_the_preferences_and_the_assets() {
        let en = Locale::english();
        let body = centred(&en, Html::text("hello"), Html::empty(), footer());
        let html = document(Document {
            locale: en,
            preferences: Preferences::default(),
            title: String::from("A <title>"),
            assets: Assets::of(Brand::Term),
            nonce: Some(String::from("n0nce")),
            head: Html::raw(String::from("<meta name=\"x\">")),
            body,
        });
        let html = html.as_str();
        assert!(
            html.starts_with("<!doctype html>\n<html lang=\"en\" data-theme=\"system\""),
            "{html}"
        );
        assert!(html.contains("<title>A &#60;title&#62;</title>"), "{html}");
        assert!(html.contains("/themes/ferroterm.css"), "{html}");
        assert!(
            html.contains("<script type=\"module\" src=\"")
                && html.contains("fui.js\" nonce=\"n0nce\">"),
            "{html}"
        );
        assert!(html.contains("<meta name=\"x\">"), "{html}");
        assert!(
            html.contains(&format!(
                "id=\"{MAIN_ID}\" class=\"fui-centred\">hello</main>"
            )),
            "{html}"
        );
        assert!(
            html.contains("FerroTEST Console 0.6.0 · Cadasto B.V. · <a href=\"/about\">About</a>"),
            "{html}"
        );
    }

    #[test]
    fn the_shell_holds_its_parts_in_order() {
        let en = Locale::english();
        let bar = topbar(
            &en,
            TopBar {
                sidebar: "nav",
                lockup: lockup("/", mark(Glyph::LAYERS), "Ferro", "Test"),
                search: None,
                end: Html::text("end"),
            },
        );
        let html = shell(
            &en,
            Shell {
                topbar: bar,
                banner: None,
                sidebar: Html::text("side"),
                content: Html::text("page"),
                dock: Some(Html::text("dock")),
                toasts: Html::empty(),
                footer: footer(),
            },
        );
        let html = html.as_str();
        let order = [
            "fui-skip",
            "fui-topbar",
            "commandfor=\"nav\"",
            "fui-lockup",
            "side",
            "fui-main",
            "page",
            "dock",
            "fui-footer",
        ];
        let mut at = 0;
        for part in order {
            let found = html
                .get(at..)
                .and_then(|rest| rest.find(part))
                .unwrap_or_else(|| panic!("{part} after {at}: {html}"));
            at += found;
        }
    }
}
