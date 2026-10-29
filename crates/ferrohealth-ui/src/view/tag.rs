// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tag: a small labelled mark for a kind, a format, an owner or a
//! namespace (R2), such as a pane head's format, a code viewer's owner and
//! language, or a permitted operation of a scope grant.
//!
//! A tag is a short word in the UI face on the sunken ground, on one line
//! while it fits, and it is read as the text it shows. A tag wider than
//! its box wraps between its words and breaks inside a word only where the
//! word alone is wider, so it never widens the page (WCAG 2.2 SC 1.4.10).
//! The word starts with a capital; where the value has a raw code, the code
//! follows inside the tag as a quiet hint in the code face. A glyph before
//! the word is hidden from assistive technology. The provenance tag (F4,
//! [`Tag::locked`]) marks a value the console may not change: the tag with
//! a dashed edge and the lock before its word, naming its owner in its
//! tooltip and its accessible name. No specification governs the tag: our
//! own design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::icon::{Glyph, icon};

/// A tag, drawn by [`Tag::view`]; [`tag`] draws the plain one.
///
/// ```
/// use ferrohealth_ui::view::tag::Tag;
///
/// let html = Tag::new("FHIRconnect").code("fhirconnect").view();
/// assert!(html.as_str().contains("fui-tag-code"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Tag {
    /// The word, starting with a capital.
    words: String,
    /// The raw code after the word, when the value has one.
    code: Option<String>,
    /// The glyph before the word.
    glyph: Option<Glyph>,
    /// Whether it is set in the code face.
    mono: bool,
    /// What owns the value when the console cannot change it (F4).
    owner: Option<String>,
    /// The tooltip, which says more than the word.
    title: Option<String>,
}

impl Tag {
    /// The tag `words`, such as "FHIR R4", "Console" or "Read".
    pub fn new(words: impl Into<String>) -> Self {
        Self {
            words: words.into(),
            code: None,
            glyph: None,
            mono: false,
            owner: None,
            title: None,
        }
    }

    /// Adds the raw code `code` after the words as a quiet hint, such as
    /// `r` after "Read" in a scope grant.
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Adds `glyph` before the words, hidden from assistive technology.
    pub fn glyph(mut self, glyph: Glyph) -> Self {
        self.glyph = Some(glyph);
        self
    }

    /// Sets the tag in the code face, for a version or an RM type.
    pub fn mono(mut self) -> Self {
        self.mono = true;
        self
    }

    /// Marks a value the console cannot change (F4): a dashed edge, no
    /// ground, muted words and the lock before them, with `owner` as its
    /// tooltip and after its word in its accessible name. A later
    /// [`Tag::title`] replaces the tooltip alone.
    pub fn locked(mut self, owner: impl Into<String>) -> Self {
        let owner = owner.into();
        self.glyph = Some(Glyph::LOCK);
        self.title = Some(owner.clone());
        self.owner = Some(owner);
        self
    }

    /// Adds the tooltip `title`, which says where the value comes from or
    /// what it means.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// The tag's classes.
    fn class(&self) -> &'static str {
        match (self.mono, self.owner.is_some()) {
            (false, false) => "fui-tag",
            (true, false) => "fui-tag fui-tag-mono",
            (false, true) => "fui-tag fui-tag-locked",
            (true, true) => "fui-tag fui-tag-mono fui-tag-locked",
        }
    }

    /// Returns the tag.
    #[must_use]
    pub fn view(self) -> Html {
        let class = self.class();
        TagView {
            class,
            glyph: self.glyph.map(|glyph| icon(glyph, "12")),
            words: self.words,
            code: self.code,
            owner: self.owner,
            title: self.title,
        }
        .html()
    }
}

#[derive(Template)]
#[template(
    source = r#"<span class="{{ class }}"{% if let Some(title) = title %} title="{{ title }}"{% endif %}>{% if let Some(glyph) = glyph %}{{ glyph }}{% endif %}{{ words }}{% if let Some(code) = code %}<span class="fui-tag-code">{{ code }}</span>{% endif %}{% if let Some(owner) = owner %}<span class="fui-sr-only"> ({{ owner }})</span>{% endif %}</span>"#,
    ext = "html"
)]
struct TagView {
    class: &'static str,
    glyph: Option<Html>,
    words: String,
    code: Option<String>,
    owner: Option<String>,
    title: Option<String>,
}

/// Returns the plain tag `words`, such as a format in a pane head.
#[must_use]
pub fn tag(words: impl Into<String>) -> Html {
    Tag::new(words).view()
}
