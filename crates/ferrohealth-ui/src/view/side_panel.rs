// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The side panel: content opened from the inline end of the page.
//!
//! It holds what the operator opens from a button that is no navigation,
//! such as the session's access and scopes. With no script its opener is a
//! link to the content's own route, so a reader reaches the content as a
//! page. Once the `side_panel` module runs, the opener is a button that
//! shows the panel as a modal HTML `dialog` through the
//! `show-modal` command: the page behind is inert, the focus moves to its
//! Close button, the browser closes it on Escape and on a press outside and
//! returns the focus to the opener (HTML Living Standard, the `dialog`
//! element and the `command` attribute; WAI-ARIA Authoring Practices, the
//! dialog (modal) pattern). From 40 rem it is 24 rem wide at the inline
//! end; below that it is a sheet over the whole viewport. Its body is the
//! server's, in the page whole, so a field in it keeps what was typed
//! while it is closed.
//!
//! An opener can carry keys ([`side_panel_with_shortcut`]), announced on
//! its button through `aria-keyshortcuts` and bound by the `keys` module
//! from 40 rem up, outside a field. In a work-list row (R9,
//! [`crate::view::worklist`]) the keys answer only while that row holds
//! the focus, so `o` opens the focused row's panel; the panel opens over
//! the row, and the browser returns the focus to the row when it closes.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::kbd::aria_keys;

/// What opens a side panel.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Opener {
    /// Its words, in the page's locale.
    pub label: String,
    /// The glyph before its words, if any.
    pub glyph: Option<Glyph>,
    /// How much it stands out.
    pub variant: Variant,
    /// The route of the panel's content as a page of its own, which the
    /// opener leads to with no script.
    pub href: String,
}

/// What a side panel shows.
#[derive(Debug)]
pub struct SidePanel {
    /// The panel's id: its heading is `id` and `-title`, its scrolling body
    /// `id` and `-body`.
    pub id: String,
    /// The heading, in the page's locale.
    pub title: String,
    /// What opens it.
    pub opener: Opener,
    /// The body, drawn on the server.
    pub body: Html,
    /// The words the panel draws itself ([`SidePanelWords::of`]).
    pub words: SidePanelWords,
}

/// The words a side panel draws itself, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidePanelWords {
    /// The Close button's name.
    pub close: Word,
}

impl SidePanelWords {
    /// The panel's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            close: tr(locale, Text::Close),
        }
    }
}

/// Returns the side panel `panel` and its opener: a link to the content's
/// route with no script, and the button that opens it once the
/// `side_panel` module runs.
#[must_use]
pub fn side_panel(panel: SidePanel) -> Html {
    drawn(panel, None)
}

/// Returns the side panel `panel` and its opener, whose button announces
/// `shortcut` (such as `o`, or `o | d` for two keys) and asks the `keys`
/// module to bind it.
///
/// A shortcut with no form in `aria-keyshortcuts` (keys pressed in turn,
/// such as `g d`) draws the opener with no key.
#[must_use]
pub fn side_panel_with_shortcut(panel: SidePanel, shortcut: &str) -> Html {
    drawn(panel, aria_keys(shortcut))
}

/// Returns `panel` with its opener announcing `keys`, if any.
fn drawn(panel: SidePanel, keys: Option<String>) -> Html {
    let SidePanel {
        id,
        title,
        opener:
            Opener {
                label,
                glyph,
                variant,
                href,
            },
        body,
        words: SidePanelWords { close },
    } = panel;
    PanelView {
        keys,
        href,
        class: variant.class(),
        glyph: glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        label,
        id,
        title,
        close,
        x: icon(Glyph::X, "16"),
        body,
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<a href="{{ href }}" class="{{ class }}">{{ glyph }}<span class="fui-side-panel-words">{{ label }}</span></a><button type="button" command="show-modal" commandfor="{{ id }}" aria-controls="{{ id }}" aria-haspopup="dialog"{% if let Some(keys) = keys %} aria-keyshortcuts="{{ keys }}"{% endif %} hidden data-fui="side_panel{% if keys.is_some() %} keys{% endif %}" class="{{ class }}">{{ glyph }}<span class="fui-side-panel-words">{{ label }}</span></button><dialog id="{{ id }}" closedby="any" aria-labelledby="{{ id }}-title" data-fui="dialog" class="fui-side-panel"><div class="fui-side-panel-head"><h2 id="{{ id }}-title" class="fui-card-title">{{ title }}</h2><button type="button" command="close" commandfor="{{ id }}" aria-label="{{ close }}" class="fui-icon-button fui-side-panel-close">{{ x }}</button></div><div id="{{ id }}-body" class="fui-side-panel-body">{{ body }}</div></dialog>"#,
    ext = "html"
)]
struct PanelView {
    keys: Option<String>,
    href: String,
    class: &'static str,
    glyph: Html,
    label: String,
    id: String,
    title: String,
    close: Word,
    x: Html,
    body: Html,
}
