// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! T2, the detail page: the trail, the title with its state and a one-line
//! subtitle, the action group, and binder tabs over one panel.
//!
//! The trail is always there under a list and becomes one back link on a
//! phone (R6). The title row holds one primary action plus More (F14), or
//! the previous and next steps of a queue (R5, [`stepper`]). The tabs
//! appear only when the object has more than one section, each tab a
//! route; a tab the object cannot have is left out, never disabled. An
//! editor opens with [`crate::view::surface::editor_heading`] instead,
//! whose trail ends with the thing it edits.

use leptos::prelude::*;

use crate::i18n::{Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::surface::{Header, page_heading};
use crate::view::template::tabbed;

/// What a detail page shows.
#[derive(Debug)]
pub struct DetailPage {
    /// The trail, the title with its state badge, and the subtitle.
    pub header: Header,
    /// The title row's actions: one primary plus More, or a [`stepper`].
    pub actions: Option<AnyView>,
    /// The binder tabs, when the object has more than one section.
    pub tabs: Option<AnyView>,
    /// The section in view: the tab's panel, or the whole page under the
    /// header when there are no tabs.
    pub body: AnyView,
}

/// Returns a detail page (T2): its header, then the section in view, under
/// binder tabs when it has them.
#[must_use]
pub fn detail_page(page: DetailPage) -> AnyView {
    let DetailPage {
        header,
        actions,
        tabs,
        body,
    } = page;
    // TODO(#153): the title row takes the action group of one primary plus More.
    let heading = page_heading(header, actions).into_any();
    let section = match tabs {
        Some(tabs) => tabbed(tabs, body, "fui-tabbed"),
        None => body,
    };
    view! {
        {heading}
        <div class="fui-stack">{section}</div>
    }
    .into_any()
}

/// The previous and next items of a queue, from the page of one of them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stepper {
    /// The queue's name, such as "Candidates".
    pub label: String,
    /// Where this item stands, such as "Candidate 3 of 12".
    pub position: String,
    /// What one item is, in the steps' names: "Previous candidate".
    pub what: String,
    /// The previous item's page, when there is one.
    pub previous: Option<String>,
    /// The next item's page, when there is one.
    pub next: Option<String>,
}

/// Returns one step of a [`stepper`]: a link to `href`, or, at the end of
/// the queue, a link with no address marked unavailable (WAI-ARIA 1.2,
/// `aria-disabled`).
fn step(href: Option<String>, words: String, name: String, glyph: Glyph, last: bool) -> AnyView {
    let words = view! { <span class="fui-step-words">{words}</span> }.into_any();
    let inner = if last {
        view! {
            {words}
            {icon(glyph, "16")}
        }
        .into_any()
    } else {
        view! {
            {icon(glyph, "16")}
            {words}
        }
        .into_any()
    };
    match href {
        Some(href) => view! {
            <a href=href class=Variant::Secondary.small() aria-label=name>
                {inner}
            </a>
        }
        .into_any(),
        None => view! {
            <a
                class=Variant::Secondary.small()
                role="link"
                aria-label=name
                aria-disabled="true"
            >
                {inner}
            </a>
        }
        .into_any(),
    }
}

/// Returns the previous and next steps through a queue, with where this
/// item stands between them, for a detail page's title row (R5).
///
/// Each step is a link, so it works with no script; at an end of the queue
/// the step stays in place, marked unavailable. Below 40 rem the steps keep
/// their arrows and their words become their accessible names alone.
#[must_use]
pub fn stepper(stepper: Stepper) -> AnyView {
    let Stepper {
        label,
        position,
        what,
        previous,
        next,
    } = stepper;
    let previous_name = tr_with(Text::StepperPrevious, &[("what", what.clone().into())]);
    let next_name = tr_with(Text::StepperNext, &[("what", what.into())]);
    let previous = step(
        previous,
        tr(Text::StepPrevious),
        previous_name,
        Glyph::ARROW_LEFT,
        false,
    );
    let next = step(
        next,
        tr(Text::StepNext),
        next_name,
        Glyph::ARROW_RIGHT,
        true,
    );
    view! {
        <nav class="fui-stepper" aria-label=label>
            {previous}
            <span class="fui-stepper-position">{position}</span>
            {next}
        </nav>
    }
    .into_any()
}
