// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The toast region: short notices an action raises.
//!
//! A screen reader reads them as they arrive (a polite live region), each with its tone's icon
//! beside its words and a close button, and each dismissed on its own after
//! [`DWELL_MS`]. A toast only repeats what the page already shows, so nothing
//! is lost when it goes (WCAG 2.2 SC 2.2.1).
//!
//! A console provides the region once ([`provide_toasts`]) and draws it
//! ([`toast_region`]); any component pushes onto it through the context.
//! A toast holds no clinical data: it says what happened, never what was in
//! it.

use leptos::prelude::*;

use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// How long a toast stays, in milliseconds.
pub const DWELL_MS: u64 = 6000;

/// One toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    /// Its id, unique within the region.
    pub id: u32,
    /// Its tone.
    pub tone: Tone,
    /// Its words.
    pub text: String,
}

/// The toast region's state, shared through the context.
#[derive(Debug, Clone, Copy)]
pub struct Toasts {
    /// The toasts shown, oldest first.
    list: RwSignal<Vec<Toast>>,
    /// The id the next toast takes.
    next: RwSignal<u32>,
}

impl Toasts {
    /// Shows a toast of `tone` saying `text`.
    pub fn push(self, tone: Tone, text: impl Into<String>) {
        let id = self.next.get_untracked();
        self.next.set(id.wrapping_add(1));
        self.list.update(|list| {
            list.push(Toast {
                id,
                tone,
                text: text.into(),
            });
        });
        self.dismiss_later(id);
    }

    /// Removes the toast `id`.
    pub fn dismiss(self, id: u32) {
        self.list.update(|list| list.retain(|toast| toast.id != id));
    }

    /// Removes the toast `id` after [`DWELL_MS`].
    fn dismiss_later(self, id: u32) {
        #[cfg(target_arch = "wasm32")]
        set_timeout(
            move || self.dismiss(id),
            std::time::Duration::from_millis(DWELL_MS),
        );
        #[cfg(not(target_arch = "wasm32"))]
        let _: (Self, u32) = (self, id);
    }
}

/// Provides the toast region's state to the components under the caller,
/// and returns it.
#[must_use = "the region draws the toasts it returns"]
pub fn provide_toasts() -> Toasts {
    let toasts = Toasts {
        list: RwSignal::new(Vec::new()),
        next: RwSignal::new(0),
    };
    provide_context(toasts);
    toasts
}

/// The toast region's state, when a parent provided it.
#[must_use]
pub fn use_toasts() -> Option<Toasts> {
    use_context::<Toasts>()
}

/// The icon of a toast of `tone`.
const fn glyph(tone: Tone) -> Glyph {
    match tone {
        Tone::Ok => Glyph::CIRCLE_CHECK,
        Tone::Warn => Glyph::TRIANGLE_ALERT,
        Tone::Bad => Glyph::CIRCLE_X,
        Tone::Info | Tone::Neutral | Tone::Accent => Glyph::INFO,
    }
}

/// Returns the region that draws `toasts`.
#[must_use]
pub fn toast_region(toasts: Toasts) -> impl IntoView {
    view! {
        <div class="fui-toasts" role="status" aria-live="polite" aria-label="Notices">
            <For each=move || toasts.list.get() key=|toast| toast.id let:toast>
                {
                    let class = format!("fui-toast {}", toast.tone.class());
                    let id = toast.id;
                    view! {
                        <div class=class>
                            {icon(glyph(toast.tone), "16")}
                            <p class="fui-toast-text">{toast.text}</p>
                            <button
                                type="button"
                                class="fui-button fui-button-ghost fui-button-small"
                                on:click=move |_| toasts.dismiss(id)
                            >
                                {icon(Glyph::X, "14")}
                                <span class="fui-sr-only">"Dismiss"</span>
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}
