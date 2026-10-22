// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The tag: a small labelled mark for a kind, a format, an owner or a
//! namespace (R2), such as a pane head's format, a code viewer's owner and
//! language, or a permitted operation of a scope grant.
//!
//! A tag is a short word in the UI face on the sunken ground, on one line
//! that never breaks inside itself, and it is read as the text it shows.
//! The word starts with a capital, as every word the family draws for a
//! state or a kind does; where the value has a raw code (a language key, a
//! scope operation), the code follows inside the tag as a quiet hint in the
//! code face, so a reader who knows the code finds it. A glyph before the
//! word is hidden from assistive technology, since the word says it
//! already (F4's lock). An identifier-like tag, a version or an RM type,
//! draws in the code face. No specification governs the tag: our own
//! design.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};

/// A tag, drawn by [`Tag::view`]; [`tag`] draws the plain one.
///
/// ```
/// use ferrohealth_ui::view::tag::Tag;
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| Tag::new("FHIRconnect").code("fhirconnect").view().to_html());
/// assert!(html.contains("fui-tag-code"));
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
    /// Whether it marks something the console cannot change (F4).
    locked: bool,
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
            locked: false,
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

    /// Marks something the console cannot change (F4): a dashed edge, no
    /// ground, muted words and the lock before them.
    pub fn locked(mut self) -> Self {
        self.locked = true;
        self.glyph = Some(Glyph::LOCK);
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
        match (self.mono, self.locked) {
            (false, false) => "fui-tag",
            (true, false) => "fui-tag fui-tag-mono",
            (false, true) => "fui-tag fui-tag-locked",
            (true, true) => "fui-tag fui-tag-mono fui-tag-locked",
        }
    }

    /// Returns the tag.
    #[must_use]
    pub fn view(self) -> AnyView {
        let class = self.class();
        let Self {
            words,
            code,
            glyph,
            title,
            ..
        } = self;
        if glyph.is_none() && code.is_none() {
            return view! {
                <span class=class title=title>
                    {words}
                </span>
            }
            .into_any();
        }
        view! {
            <span class=class title=title>
                {glyph.map(|glyph| icon(glyph, "12"))}
                {words}
                {code.map(|code| view! { <span class="fui-tag-code">{code}</span> })}
            </span>
        }
        .into_any()
    }
}

/// Returns the plain tag `words`, such as a format in a pane head.
#[must_use]
pub fn tag(words: impl Into<String>) -> AnyView {
    Tag::new(words).view()
}
