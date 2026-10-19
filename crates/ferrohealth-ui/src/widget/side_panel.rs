// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The side panel: content the operator opens from a menu or a button that
//! is no navigation, such as the session's access and scopes, from the
//! inline end of the page.
//!
//! It is an HTML popover, so a button anywhere opens it with no script, the
//! browser closes it on Escape and on a press outside, and returns the
//! focus to the button (HTML, the `popover` attribute). The browser half
//! moves the focus to its Close button as it opens. Its body is built once
//! and kept, so a field in it keeps what was typed while it is closed.

use leptos::prelude::*;

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};

/// What a side panel shows.
#[derive(Debug)]
pub struct SidePanel {
    /// The panel's id, which its trigger opens; its heading is `id` and
    /// `-title`.
    pub id: &'static str,
    /// The id of its scrolling body, which a journey reads.
    pub body_id: &'static str,
    /// The heading.
    pub title: &'static str,
    /// The body.
    pub body: AnyView,
}

/// Returns a button of `class` that opens the side panel `id`, with `glyph`
/// beside `label`.
#[must_use]
pub fn side_panel_trigger(
    id: &'static str,
    class: &'static str,
    glyph: Glyph,
    label: &'static str,
) -> impl IntoView + use<> {
    view! {
        <button type="button" class=class popovertarget=id aria-controls=id>
            {icon(glyph, "16")}
            {label}
        </button>
    }
}

/// Returns the side panel `panel`.
#[must_use]
pub fn side_panel(panel: SidePanel) -> AnyView {
    let SidePanel {
        id,
        body_id,
        title,
        body,
    } = panel;
    let node = NodeRef::<leptos::html::Aside>::new();
    let title_id = format!("{id}-title");
    let labelled_by = title_id.clone();
    view! {
        <aside
            id=id
            popover="auto"
            role="dialog"
            aria-labelledby=labelled_by
            class="fui-side-panel"
            node_ref=node
            on:toggle=move |_| opened(node)
        >
            <div class="fui-side-panel-head">
                <h2 id=title_id class="fui-card-title">
                    {title}
                </h2>
                <button
                    type="button"
                    class="fui-icon-button fui-side-panel-close"
                    popovertarget=id
                    popovertargetaction="hide"
                    aria-label=tr(Text::Close)
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

/// Moves the focus to the panel's Close button, once it is open.
fn opened(node: NodeRef<leptos::html::Aside>) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let Some(panel) = node.get_untracked() else {
            return;
        };
        if !panel.matches(":popover-open").unwrap_or(false) {
            return;
        }
        let close = panel
            .query_selector(".fui-side-panel-close")
            .ok()
            .flatten()
            .and_then(|button| button.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(close) = close
            && close.focus().is_err()
        {
            leptos::logging::warn!("the browser refused the focus on the side panel");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: NodeRef<leptos::html::Aside> = node;
}
