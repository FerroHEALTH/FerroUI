// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The toast region: short notices an action raises.
//!
//! A toast is a title, an optional line under it and an optional action
//! under the text, beside its tone's icon, with a Dismiss button. A screen
//! reader reads it as it arrives: the region is a polite live region, and a
//! toast of the bad tone is an alert (WAI-ARIA 1.2, the `status` and
//! `alert` roles). A toast without an action goes on its own after
//! [`DWELL_MS`], the count held while the pointer or the focus is on the
//! region; one with an action stays until it is dismissed, so the action
//! can be reached at any pace (WCAG 2.2 SC 2.2.1). The region shows at most
//! [`SHOWN_MAX`] toasts, the oldest that is no refusal and has no action
//! giving way first,
//! stays above the page's decision bar (R4) and scrolls past the viewport's
//! height, so it never covers the decisions (SC 2.4.11). A toast holds no
//! clinical data: it says what happened, never what was in it, and it never
//! repeats what a banner or a load report on the page already says.
//!
//! A console draws the region once in its shell, as the island
//! [`crate::island::ToastRegion`]. Any island raises a toast with
//! [`raise`]: every island of a page runs in one WebAssembly instance, so
//! they share one queue there, which exists in the browser alone. With no
//! WebAssembly the server draws the region with the [`Flash`] toasts the
//! console hands it, the outcome of a form post it redirected back from,
//! each with a Dismiss that is a native disclosure, so it closes with no
//! script. The page keeps the region's height clear at its foot while a
//! toast shows, so a toast covers no control of the page.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Text, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;

/// How long a toast without an action stays, in milliseconds, counted
/// while neither the pointer nor the focus is on the region.
pub const DWELL_MS: u64 = 6000;

/// The most toasts the region shows at once. Past it the oldest toast
/// that is no refusal and has no action leaves; a refusal or a toast with
/// an action stays, and the region scrolls.
pub const SHOWN_MAX: usize = 3;

/// A toast's action that leads somewhere: a link drawn as a small button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToastLink {
    /// What the action says, such as "Open History".
    pub label: String,
    /// Where it leads.
    pub href: String,
}

/// What a toast's action does.
#[derive(Debug, Clone)]
pub enum ToastAction {
    /// Leads to a page.
    Link(ToastLink),
    /// Runs `run` in the browser and dismisses the toast.
    Run {
        /// What the action says, such as "Undo".
        label: String,
        /// What it does.
        run: Callback<()>,
    },
}

/// A toast an island raises.
#[derive(Debug, Clone)]
pub struct Toast {
    /// Its tone; the bad tone is an alert.
    pub tone: Tone,
    /// What happened, in a few words.
    pub title: String,
    /// The line under the title, when there is more to say.
    pub line: Option<String>,
    /// The one thing to do about it, when there is one.
    pub action: Option<ToastAction>,
}

impl Toast {
    /// A toast of `tone` titled `title`, with no line and no action.
    #[must_use]
    pub fn new(tone: Tone, title: impl Into<String>) -> Self {
        Self {
            tone,
            title: title.into(),
            line: None,
            action: None,
        }
    }

    /// This toast with `line` under its title.
    #[must_use]
    pub fn line(self, line: impl Into<String>) -> Self {
        Self {
            line: Some(line.into()),
            ..self
        }
    }

    /// This toast with `action` under its text.
    #[must_use]
    pub fn action(self, action: ToastAction) -> Self {
        Self {
            action: Some(action),
            ..self
        }
    }
}

/// A toast the server draws as the page arrives, such as what a form post
/// the console redirected back from did (POST, redirect, GET).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flash {
    /// Its tone; the bad tone is an alert.
    pub tone: Tone,
    /// What happened, in a few words.
    pub title: String,
    /// The line under the title, when there is more to say.
    pub line: Option<String>,
    /// The page to go on to, when there is one.
    pub action: Option<ToastLink>,
}

impl From<Flash> for Toast {
    fn from(flash: Flash) -> Self {
        Self {
            tone: flash.tone,
            title: flash.title,
            line: flash.line,
            action: flash.action.map(ToastAction::Link),
        }
    }
}

/// The words the region draws itself, resolved on the server in the page's
/// locale, so the island carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegionWords {
    /// The region's name.
    pub region: String,
    /// The Dismiss button's name.
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
#[derive(Debug, Clone)]
struct Shown {
    /// Its id, unique within the page.
    id: u32,
    /// What it says and does.
    toast: Toast,
}

/// Takes the oldest toasts that are no refusal and have no action out of
/// `list` until it holds [`SHOWN_MAX`] or none of those is left to take.
fn trim(list: &mut Vec<Shown>) {
    while list.len() > SHOWN_MAX {
        let Some(oldest) = list
            .iter()
            .position(|shown| shown.toast.action.is_none() && shown.toast.tone != Tone::Bad)
        else {
            return;
        };
        list.remove(oldest);
    }
}

#[cfg(target_arch = "wasm32")]
thread_local! {
    /// The toasts shown, oldest first, shared by every island of the page.
    static QUEUE: ArcRwSignal<Vec<Shown>> = ArcRwSignal::new(Vec::new());
    /// The id the next toast takes.
    static NEXT: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
    /// Whether the pointer is on the region.
    static HOVERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// Whether the focus is in the region.
    static FOCUSED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    /// The count the running dwells belong to; a hold or a release starts
    /// another, so the dwells of the one before do nothing.
    static EPOCH: std::cell::Cell<u32> = const { std::cell::Cell::new(0) };
}

/// What holds the toasts on the screen.
#[derive(Debug, Clone, Copy)]
enum Hold {
    /// The pointer is on the region.
    Pointer,
    /// The focus is in the region.
    Focus,
}

/// Records whether `hold` is on the region: while either is, no toast
/// leaves, and when both have gone every toast without an action starts
/// its full dwell again (WCAG 2.2 SC 2.2.1).
fn held(hold: Hold, on: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let cell = match hold {
            Hold::Pointer => &HOVERED,
            Hold::Focus => &FOCUSED,
        };
        if cell.with(|cell| cell.replace(on)) == on {
            return;
        }
        EPOCH.with(|epoch| epoch.set(epoch.get().wrapping_add(1)));
        if paused() {
            return;
        }
        let waiting: Vec<u32> = QUEUE.with(|queue| {
            queue.with_untracked(|list| {
                list.iter()
                    .filter(|shown| shown.toast.action.is_none())
                    .map(|shown| shown.id)
                    .collect()
            })
        });
        for id in waiting {
            dismiss_later(id);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (Hold, bool) = (hold, on);
}

/// Whether the pointer or the focus holds the toasts.
#[cfg(target_arch = "wasm32")]
fn paused() -> bool {
    HOVERED.with(std::cell::Cell::get) || FOCUSED.with(std::cell::Cell::get)
}

/// Shows `toast` in the page's toast region, from an island's event
/// handler. On the server it shows nothing: the region there draws the
/// [`Flash`] toasts it was handed.
pub fn raise(toast: Toast) {
    #[cfg(target_arch = "wasm32")]
    {
        let id = NEXT.with(|next| {
            let id = next.get();
            next.set(id.wrapping_add(1));
            id
        });
        let stays = toast.action.is_some();
        QUEUE.with(|queue| {
            queue.update(|list| {
                list.push(Shown { id, toast });
                trim(list);
            });
        });
        if !stays {
            dismiss_later(id);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    drop(toast);
}

/// Removes the toast `id` after [`DWELL_MS`], unless the toasts are held
/// then or were held and released since.
#[cfg(target_arch = "wasm32")]
fn dismiss_later(id: u32) {
    let epoch = EPOCH.with(std::cell::Cell::get);
    set_timeout(
        move || {
            if paused() || EPOCH.with(std::cell::Cell::get) != epoch {
                return;
            }
            QUEUE.with(|queue| queue.update(|list| list.retain(|shown| shown.id != id)));
        },
        std::time::Duration::from_millis(DWELL_MS),
    );
}

/// The list the region draws, seeded with `seeded`: the page's queue in
/// the browser, a list of the render's own on the server.
fn shown(seeded: Vec<Shown>) -> ArcRwSignal<Vec<Shown>> {
    #[cfg(target_arch = "wasm32")]
    {
        let next = seeded.last().map_or(0, |shown| shown.id.wrapping_add(1));
        for shown in seeded.iter().filter(|shown| shown.toast.action.is_none()) {
            dismiss_later(shown.id);
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
        Tone::Bad => Glyph::CIRCLE_ALERT,
        Tone::Info | Tone::Neutral | Tone::Accent => Glyph::INFO,
    }
}

/// The role of a toast of `tone`: an alert for the bad tone, and none for
/// the others, which the polite region around them announces.
#[must_use]
pub const fn role(tone: Tone) -> Option<&'static str> {
    match tone {
        Tone::Bad => Some("alert"),
        Tone::Ok | Tone::Warn | Tone::Info | Tone::Neutral | Tone::Accent => None,
    }
}

/// Returns the toast region in `words`, showing `flash` as it arrives and
/// every toast [`raise`] shows after. A console draws it as the island
/// [`crate::island::ToastRegion`].
#[must_use]
pub fn toast_region(words: RegionWords, flash: Vec<Flash>) -> impl IntoView {
    let mut seeded = (0_u32..)
        .zip(flash)
        .map(|(id, flash)| Shown {
            id,
            toast: flash.into(),
        })
        .collect();
    trim(&mut seeded);
    let list = shown(seeded);
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let close = {
        let list = list.clone();
        move |id: u32| list.update(|shown| shown.retain(|each| each.id != id))
    };
    let RegionWords { region, dismiss } = words;
    view! {
        <div
            class="fui-toasts"
            role="status"
            aria-live="polite"
            aria-label=region
            on:pointerenter=move |_| held(Hold::Pointer, true)
            on:pointerleave=move |_| held(Hold::Pointer, false)
            on:focusin=move |_| held(Hold::Focus, true)
            on:focusout=move |_| held(Hold::Focus, false)
        >
            <For each=move || list.get() key=|shown| shown.id let:shown>
                {toast_view(shown, dismiss.clone(), close.clone(), hydrated)}
            </For>
        </div>
    }
}

/// Returns the toast `shown`, its Dismiss button named `dismiss` running
/// `close`, shown once the page has `hydrated`.
fn toast_view(
    shown: Shown,
    dismiss: String,
    close: impl Fn(u32) + Clone + Send + Sync + 'static,
    hydrated: RwSignal<bool>,
) -> AnyView {
    let Shown { id, toast } = shown;
    let class = format!("fui-toast {}", toast.tone.class());
    let line = toast.line.map(|line| view! { <span>{line}</span> });
    let action = toast.action.map(|action| match action {
        ToastAction::Link(ToastLink { label, href }) => view! {
            <a href=href class=Variant::Secondary.small()>
                {label}
            </a>
        }
        .into_any(),
        ToastAction::Run { label, run } => {
            let close = close.clone();
            view! {
                <button
                    type="button"
                    class=Variant::Secondary.small()
                    on:click=move |_| {
                        run.run(());
                        close(id);
                    }
                >
                    {label}
                </button>
            }
            .into_any()
        }
    });
    // NOTE: leptos-ui.md §15.8: until the page hydrates, Dismiss is a native
    // disclosure that hides its toast when closed; after, a button.
    let shut = dismiss.clone();
    view! {
        <div class=class role=role(toast.tone)>
            {icon(glyph(toast.tone), "16")}
            <div class="fui-toast-text">
                <strong>{toast.title}</strong>
                {line}
            </div>
            {action}
            <details open class="fui-toast-close" hidden=move || hydrated.get()>
                <summary class="fui-icon-button" aria-label=shut.clone() title=shut>
                    {icon(Glyph::X, "16")}
                </summary>
            </details>
            <button
                type="button"
                class="fui-icon-button"
                aria-label=dismiss.clone()
                title=dismiss
                hidden=move || !hydrated.get()
                on:click=move |_| close(id)
            >
                {icon(Glyph::X, "16")}
            </button>
        </div>
    }
    .into_any()
}
