// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The side panel: content opened from the inline end of the page.
//!
//! It holds what the operator opens from a button that is no navigation,
//! such as the session's access and scopes; a page draws it through
//! [`crate::island::SidePanel`]. Until the page has hydrated, its opener is a link to the content's own
//! route, so a reader with no WebAssembly reaches the content as a page.
//! Once it has hydrated, the opener is a button that shows the panel as an
//! HTML popover: the browser closes it on Escape and on a press outside
//! (HTML, the `popover` attribute). The panel moves the focus to its Close
//! button as it opens, through the directive
//! [`crate::widget::focus::focus_on_open`], and returns it to the opener
//! as it closes (WAI-ARIA Authoring Practices, the dialog pattern). From
//! 40 rem it is 24 rem wide at the inline end; below that it is a sheet
//! over the whole viewport. Its body is the server's, built once and kept,
//! so a field in it keeps what was typed while it is closed.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::widget::focus::focus_on_open;

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
    /// opener leads to until the page has hydrated.
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
    pub body: AnyView,
    /// The words the panel draws itself ([`SidePanelWords::of_page`]).
    pub words: SidePanelWords,
}

/// The words a side panel draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SidePanelWords {
    /// The Close button's name.
    pub close: String,
}

impl SidePanelWords {
    /// The panel's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            close: tr(Text::Close),
        }
    }
}

/// Returns the side panel `panel` and its opener: a link to the content's
/// route until the page has hydrated, then the button that opens it.
#[must_use]
pub fn side_panel(panel: SidePanel) -> AnyView {
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
    let hydrated = RwSignal::new(false);
    // NOTE: the Leptos book, "Guide: Islands"; an effect runs in the browser alone.
    Effect::new(move |_| hydrated.set(true));
    let button = NodeRef::<leptos::html::Button>::new();
    let aside = NodeRef::<leptos::html::Aside>::new();
    let on_toggle = move |_| browser::return_focus(aside, button);
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    let body_id = format!("{id}-body");
    let link_label = label.clone();
    let (target, controls, close_target) = (id.clone(), id.clone(), id.clone());
    view! {
        <a href=href class=variant.class() hidden=move || hydrated.get()>
            {glyph.map(|glyph| icon(glyph, "16"))}
            {link_label}
        </a>
        <button
            type="button"
            class=variant.class()
            popovertarget=target
            aria-controls=controls
            hidden=move || !hydrated.get()
            node_ref=button
        >
            {glyph.map(|glyph| icon(glyph, "16"))}
            {label}
        </button>
        <aside
            id=id
            popover="auto"
            role="dialog"
            aria-labelledby=labelled_by
            class="fui-side-panel"
            node_ref=aside
            on:toggle=on_toggle
            use:focus_on_open=".fui-side-panel-close"
        >
            <div class="fui-side-panel-head">
                <h2 id=title_id class="fui-card-title">
                    {title}
                </h2>
                <button
                    type="button"
                    class="fui-icon-button fui-side-panel-close"
                    popovertarget=close_target
                    popovertargetaction="hide"
                    aria-label=close
                >
                    {icon(Glyph::X, "16")}
                </button>
            </div>
            <div id=body_id class="fui-side-panel-body">
                {body}
            </div>
        </aside>
    }
    .into_any()
}

/// What the browser code adds to the panel.
mod browser {
    use leptos::prelude::*;

    /// Returns the focus to `opener` as the closed `panel` gives it up: when
    /// the panel held it, or when it fell to the page's body.
    pub(super) fn return_focus(
        panel: NodeRef<leptos::html::Aside>,
        opener: NodeRef<leptos::html::Button>,
    ) {
        #[cfg(target_arch = "wasm32")]
        {
            let (Some(element), Some(opener)) = (panel.get_untracked(), opener.get_untracked())
            else {
                return;
            };
            if element.matches(":popover-open").unwrap_or(false) {
                return;
            }
            let held = document().active_element().is_none_or(|active| {
                element.contains(Some(&active))
                    || document()
                        .body()
                        .is_some_and(|body| AsRef::<web_sys::Element>::as_ref(&body) == &active)
            });
            if held && opener.focus().is_err() {
                leptos::logging::warn!("the browser refused the focus on the side panel's opener");
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _: (NodeRef<leptos::html::Aside>, NodeRef<leptos::html::Button>) = (panel, opener);
    }
}
