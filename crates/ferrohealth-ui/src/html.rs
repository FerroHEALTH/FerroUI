// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Rendered markup: what every component returns and what a page is built
//! from.
//!
//! [`Html`] holds markup the library rendered or escaped itself, so a
//! console's Askama template writes it with `{{ panel }}` and it is never
//! escaped twice (it is `HtmlSafe`). A component takes its children as
//! `Html` too, so composition is by value: a card takes its body, the shell
//! takes the page. Text a console hands a component is escaped where the
//! component writes it. No specification governs this: our own design.

use std::fmt;

use askama::Template;
use askama::filters::HtmlSafe;

/// Markup the library rendered or escaped: safe to write into a page.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Html(String);

impl Html {
    /// No markup at all.
    #[must_use]
    pub const fn empty() -> Self {
        Self(String::new())
    }

    /// Markup the caller rendered or escaped itself, taken as it stands.
    ///
    /// A console passes only markup it made with the library or escaped
    /// with [`Html::text`]; raw user text here is an injection.
    #[must_use]
    pub const fn raw(markup: String) -> Self {
        Self(markup)
    }

    /// `text` escaped as a text node or an attribute value.
    #[must_use]
    pub fn text(text: impl AsRef<str>) -> Self {
        let mut out = String::with_capacity(text.as_ref().len());
        escape_into(&mut out, text.as_ref());
        Self(out)
    }

    /// The markup as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether it holds no markup.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Every piece of `parts`, one after the other.
    #[must_use]
    pub fn join<I: IntoIterator<Item = Self>>(parts: I) -> Self {
        parts.into_iter().collect()
    }

    /// This markup followed by `next`.
    #[must_use]
    pub fn then(mut self, next: Self) -> Self {
        let Self(tail) = next;
        self.0.push_str(&tail);
        self
    }

    /// The markup `template` renders.
    ///
    /// # Panics
    ///
    /// Never: a `String` refuses no write, so a template renders as its
    /// `Display` impls do.
    #[must_use]
    pub fn of<T: Template>(template: &T) -> Self {
        let mut out = String::new();
        // NOTE: std `impl fmt::Write for String` never fails, and every value a
        // library template writes is a String, a number or markup, so the one
        // error `render_into` can carry cannot happen.
        #[expect(
            clippy::expect_used,
            reason = "a String refuses no write and the library's Display impls never fail"
        )]
        template
            .render_into(&mut out)
            .expect("a template should render into a String");
        Self(out)
    }
}

impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl HtmlSafe for Html {}

impl From<Html> for String {
    fn from(html: Html) -> Self {
        html.0
    }
}

impl FromIterator<Html> for Html {
    fn from_iter<I: IntoIterator<Item = Html>>(parts: I) -> Self {
        let mut out = String::new();
        for part in parts {
            out.push_str(&part.0);
        }
        Self(out)
    }
}

impl Extend<Html> for Html {
    fn extend<I: IntoIterator<Item = Html>>(&mut self, parts: I) {
        for part in parts {
            self.0.push_str(&part.0);
        }
    }
}

/// Writes `text` into `out` with the five characters HTML reserves
/// escaped, so it reads as text in an element or an attribute.
pub fn escape_into(out: &mut String, text: &str) {
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            c => out.push(c),
        }
    }
}

/// The markup of `template`, the shape every component function returns.
pub trait Render: Template {
    /// The rendered markup.
    fn html(&self) -> Html
    where
        Self: Sized,
    {
        Html::of(self)
    }
}

impl<T: Template> Render for T {}

#[cfg(test)]
mod tests {
    use super::Html;

    #[test]
    fn text_is_escaped_and_raw_markup_is_kept() {
        assert_eq!("a &lt;b&gt; &amp; &quot;c&#x27;", Html::text("a <b> & \"c'").as_str());
        assert_eq!("<b>x</b>", Html::raw(String::from("<b>x</b>")).as_str());
        let joined = Html::join([Html::text("1"), Html::empty(), Html::text("2")]);
        assert_eq!("12", joined.as_str());
        assert!(Html::empty().is_empty());
    }
}
