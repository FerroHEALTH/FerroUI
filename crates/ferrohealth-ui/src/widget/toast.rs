// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The toast region: short notices an action raises.
//!
//! A screen reader reads them as they arrive (a polite live region), each
//! with its tone's icon beside its words and a close button, and each
//! dismissed on its own after [`DWELL_MS`]. A toast only repeats what the
//! page already shows, so nothing is lost when it goes (WCAG 2.2
//! SC 2.2.1). A toast holds no clinical data: it says what happened, never
//! what was in it.
//!
//! A console draws the region once in its shell, as the island
//! [`crate::island::ToastRegion`]. Any island raises a toast with
//! [`raise`]: every island of a page runs in one WebAssembly instance, so
//! they share one queue there, which exists in the browser alone. With no
//! WebAssembly the server draws the region with the [`Flash`] toasts the
//! console hands it, the outcome of a form post it redirected back from.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;

/// How long a toast stays, in milliseconds.
pub const DWELL_MS: u64 = 6000;

/// A toast the server draws as the page arrives, such as what a form post
/// the console redirected back from did (POST, redirect, GET).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flash {
    /// Its tone.
    pub tone: Tone,
    /// Its words.
    pub text: String,
}

/// The words the region draws itself, resolved on the server in the page's
/// locale, so the island carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionWords {
    /// The region's name.
    pub region: String,
    /// The close button's name.
    pub dismiss: String,
}

impl RegionWords {
    /// The region's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            region: tr(Text::ToastRegion),
            dismiss: tr(Text::ToastDismiss),
        }
    }
}

/// One toast the region shows.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Shown {
    /// Its id, unique within the page.
    id: u32,
    /// Its tone.
    tone: Tone,
    /// Its words.
    text: String,
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// The toasts shown, oldest first, shared by every island of the page.
    static QUEUE: ArcRwSignal<Vec<Shown>> = ArcRwSignal::new(Vec::new());
    /// The id the next toast takes.
    static NEXT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// Shows a toast of `tone` saying `text` in the page's toast region, from
/// an island's event handler. On the server it shows nothing: the region
/// there draws the [`Flash`] toasts it was handed.
pub fn raise(tone: Tone, text: impl Into<String>) {
    #[cfg(target_arch = "wasm32")]
    {
        let id = NEXT.with(|next| {
            let id = next.get();
            next.set(id.wrapping_add(1));
            id
        });
        let text = text.into();
        QUEUE.with(|queue| queue.update(|list| list.push(Shown { id, tone, text })));
        dismiss_later(id);
    }
    #[cfg(not(target_arch = "wasm32"))]
    drop((tone, text.into()));
}

/// Removes the toast `id` after [`DWELL_MS`].
#[cfg(target_arch = "wasm32")]
fn dismiss_later(id: u32) {
    set_timeout(
        move || {
            QUEUE.with(|queue| queue.update(|list| list.retain(|toast| toast.id != id)));
        },
        std::time::Duration::from_millis(DWELL_MS),
    );
}

/// The list the region draws, seeded with `seeded`: the page's queue in
/// the browser, a list of the render's own on the server.
fn shown(seeded: Vec<Shown>) -> ArcRwSignal<Vec<Shown>> {
    #[cfg(target_arch = "wasm32")]
    {
        let next = seeded.last().map_or(0, |toast| toast.id.wrapping_add(1));
        for toast in &seeded {
            dismiss_later(toast.id);
        }
        NEXT.with(|cell| cell.set(next));
        QUEUE.with(|queue| {
            queue.set(seeded);
            queue.clone()
        })
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        ArcRwSignal::new(seeded)
    }
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

/// Returns the toast region in `words`, showing `flash` as it arrives and
/// every toast [`raise`] shows after. A console draws it as the island
/// [`crate::island::ToastRegion`].
#[must_use]
pub fn toast_region(words: RegionWords, flash: Vec<Flash>) -> impl IntoView {
    let seeded = (0_u32..)
        .zip(flash)
        .map(|(id, flash)| Shown {
            id,
            tone: flash.tone,
            text: flash.text,
        })
        .collect();
    let list = shown(seeded);
    let close = {
        let list = list.clone();
        move |id: u32| list.update(|shown| shown.retain(|toast| toast.id != id))
    };
    let RegionWords { region, dismiss } = words;
    view! {
        <div class="fui-toasts" role="status" aria-live="polite" aria-label=region>
            <For each=move || list.get() key=|toast| toast.id let:toast>
                {
                    let class = format!("fui-toast {}", toast.tone.class());
                    let id = toast.id;
                    let close = close.clone();
                    view! {
                        <div class=class>
                            {icon(glyph(toast.tone), "16")}
                            <p class="fui-toast-text">{toast.text}</p>
                            <button
                                type="button"
                                class="fui-button fui-button-ghost fui-button-small"
                                on:click=move |_| close(id)
                            >
                                {icon(Glyph::X, "14")}
                                <span class="fui-sr-only">{dismiss.clone()}</span>
                            </button>
                        </div>
                    }
                }
            </For>
        </div>
    }
}
