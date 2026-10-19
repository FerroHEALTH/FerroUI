// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The surfaces a page is built from.
//!
//! They are the page header with its trail and meta line, the editor's
//! header, the card and the panel with its header action, the stat card, the table card, the
//! definition grids, the well, the chip, the disclosure, and the stack,
//! grid, cluster and split that lay them out.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Message, Text, tr};

use crate::icon::{Glyph, icon};

/// One ancestor of a page in its breadcrumb trail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Crumb {
    /// The words of the link.
    pub label: String,
    /// Where it leads.
    pub href: String,
}

impl Crumb {
    /// Returns the crumb `label` leading to `href`.
    #[must_use]
    pub fn new(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
        }
    }
}

/// What a page header shows.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Header {
    /// The page's title, also the trail's last, current, crumb.
    pub title: String,
    /// One line under the title, when there is one.
    pub subtitle: Option<String>,
    /// The page's ancestors, the root first; none for a page with no trail.
    pub crumbs: Vec<Crumb>,
    /// Whether the title is drawn in the code face, for an identifier.
    pub mono: bool,
}

/// Returns the header that opens a page: the trail of its ancestors when it
/// has one, its title with `actions` at the far end, and its subtitle line.
///
/// The trail is a `nav` named "Breadcrumb" holding an ordered list, the
/// current page last and marked `aria-current="page"`, the separators hidden
/// from assistive technology (WAI-ARIA Authoring Practices, the breadcrumb
/// pattern).
#[must_use]
pub fn page_heading(header: Header, actions: impl IntoView) -> impl IntoView {
    let title = header.title.clone();
    heading(header, Signal::stored(title), actions)
}

/// Returns the header [`page_heading`] draws, its title following `title`.
///
/// The title and the trail's current crumb follow the signal, so a page
/// whose route names the thing it shows keeps its heading true when the
/// route changes under client-side navigation; `header.title` is not read.
#[must_use]
pub fn page_heading_live(
    header: Header,
    title: Signal<String>,
    actions: impl IntoView,
) -> impl IntoView {
    heading(header, title, actions)
}

/// The header both [`page_heading`] and [`page_heading_live`] draw.
fn heading(header: Header, title: Signal<String>, actions: impl IntoView) -> impl IntoView {
    let Header {
        subtitle,
        crumbs,
        mono,
        ..
    } = header;
    let trail = (!crumbs.is_empty()).then(|| {
        let links = crumbs
            .into_iter()
            .map(|crumb| {
                view! {
                    <li>
                        <a href=crumb.href>{crumb.label}</a>
                        <span class="fui-crumb-sep" aria-hidden="true">
                            "/"
                        </span>
                    </li>
                }
            })
            .collect_view();
        view! {
            <nav class="fui-crumbs" aria-label=tr(Text::Breadcrumb)>
                <ol>{links} <li aria-current="page">{move || title.get()}</li></ol>
            </nav>
        }
    });
    // Two arms, so a title in the body face writes no `class` attribute.
    let h1 = if mono {
        view! { <h1 class="fui-page-title-mono">{move || title.get()}</h1> }.into_any()
    } else {
        view! { <h1>{move || title.get()}</h1> }.into_any()
    };
    view! {
        <header class="fui-page-header">
            {trail} <div class="fui-page-heading">
                {h1}
                <div class="fui-actions">{actions}</div>
            </div>
            {subtitle.map(|line| view! { <p class="fui-page-subtitle">{line}</p> })}
        </header>
    }
}

/// Returns the header that opens a page: its `title`, one `subtitle` line
/// when there is one, and `actions` at the far end.
#[must_use]
pub fn page_header(
    title: impl Into<String>,
    subtitle: Option<String>,
    actions: impl IntoView,
) -> impl IntoView {
    page_heading(
        Header {
            title: title.into(),
            subtitle,
            ..Header::default()
        },
        actions,
    )
}

/// What an editor does: create a new thing, or change the one named.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    /// It creates a new thing.
    Creating,
    /// It changes the thing named, by its name, title or path.
    Editing(String),
}

impl Mode {
    /// The mode's `$mode` value: `new` or `edit`.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Creating => "new",
            Self::Editing(_) => "edit",
        }
    }
}

/// The title of an editor in `mode`, such as "New code system" or "Edit
/// Vital signs".
///
/// It is the console's `message` with `$mode` set to `new` or `edit` and
/// `$name` to the name of the thing changed (empty while creating).
///
/// The message is the console's, so its catalog words both titles whole:
///
/// ```ftl
/// code-system-editor =
///     { $mode ->
///         [new] New code system
///        *[edit] Edit { $name }
///     }
/// ```
///
/// A console writes the same words as the document's title.
#[must_use]
pub fn editor_title(message: Message, mode: &Mode) -> String {
    let name = match mode {
        Mode::Creating => String::new(),
        Mode::Editing(name) => name.clone(),
    };
    message.text_with(&[("mode", mode.key().into()), ("name", name.into())])
}

/// Returns the header that opens an editor in `mode`, so a reader tells from
/// the heading whether the form creates or changes.
///
/// It is [`page_heading`] with its title and the trail's current crumb from
/// [`editor_title`].
///
/// `header.title` and `header.mono` are not read: the title is a sentence,
/// so it is drawn in the body face even around an identifier.
#[must_use]
pub fn editor_heading(
    message: Message,
    mode: &Mode,
    header: Header,
    actions: impl IntoView + 'static,
) -> AnyView {
    let title = editor_title(message, mode);
    page_heading(
        Header {
            title,
            mono: false,
            ..header
        },
        actions,
    )
    .into_any()
}

/// Returns the small muted line under a page's header that says who
/// answered the page (a replica, a server); nothing for no line.
#[must_use]
pub fn page_meta(line: Option<String>) -> impl IntoView {
    line.map(|line| view! { <p class="fui-page-meta">{line}</p> })
}

/// Returns a card titled `title` holding `body`, a section under a
/// level-two heading.
#[must_use]
pub fn card(title: impl Into<String>, body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! {
        <section class="fui-card">
            <h2 class="fui-card-title">{title.into()}</h2>
            {body}
        </section>
    }
}

/// Returns a card titled `title` holding `body` that spans the whole row of
/// the grid it sits in.
#[must_use]
pub fn wide_card(title: impl Into<String>, body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! {
        <section class="fui-card fui-card-wide">
            <h2 class="fui-card-title">{title.into()}</h2>
            {body}
        </section>
    }
}

/// Returns a card holding `body` with no heading of its own.
#[must_use]
pub fn panel(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class="fui-card">{body}</div> }
}

/// Returns a panel: a card whose first row holds its title and actions.
///
/// `title` and `actions` sit over `body`, which runs to the card's edges (a table, a list). Pass a
/// `body` with its own padding through [`panel_body`] when it is prose.
#[must_use]
pub fn titled_panel(
    title: impl Into<String>,
    actions: impl IntoView + 'static,
    body: impl IntoView + 'static,
) -> impl IntoView {
    let actions = actions.into_any();
    let body = body.into_any();
    view! {
        <section class="fui-card fui-panel">
            <div class="fui-panel-head">
                <h2 class="fui-card-title">{title.into()}</h2>
                <div class="fui-actions">{actions}</div>
            </div>
            {body}
        </section>
    }
}

/// Returns the action at the far end of a card's or a panel's header, such
/// as "All members" leading to the members.
///
/// It is a link to `href`, the whole of what the card previews: its words
/// `label`, then an arrow.
///
/// Pass it as a [`titled_panel`]'s actions. The arrow is decoration beside
/// the words, hidden from assistive technology.
#[must_use]
pub fn card_action(href: impl Into<String>, label: impl Into<String>) -> AnyView {
    view! {
        <a href=href.into() class="fui-card-action">
            <span>{label.into()}</span>
            {icon(Glyph::ARROW_RIGHT, "14")}
        </a>
    }
    .into_any()
}

/// Returns `body` with the card's padding, for a panel's prose.
#[must_use]
pub fn panel_body(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class="fui-panel-body">{body}</div> }
}

/// What a stat card shows.
#[derive(Debug, Clone)]
pub struct Stat {
    /// The icon beside the value.
    pub glyph: Glyph,
    /// The measure, as text.
    pub value: String,
    /// What the measure is: a fixed word or one the console reads at run
    /// time.
    pub label: Cow<'static, str>,
    /// The page the card leads to, when it leads anywhere.
    pub href: Option<String>,
}

/// Returns the card around a stat's `inner`: the whole card a link to
/// `href` when the stat leads to a page.
fn stat_frame(href: Option<String>, inner: AnyView) -> AnyView {
    match href {
        Some(href) => view! {
            <a href=href class="fui-card fui-stat">
                {inner}
            </a>
        }
        .into_any(),
        None => view! { <div class="fui-card fui-stat">{inner}</div> }.into_any(),
    }
}

/// Returns a stat card: the glyph in a square on the accent's tint, the
/// value large in tabular figures, the label under it, the whole card a
/// link when the stat leads to a page.
///
/// Below 30 rem the card is a compact tile, the glyph, the value and the
/// label on one line, the label cut with an ellipsis and named whole in its
/// `title`.
#[must_use]
pub fn stat_card(stat: Stat) -> impl IntoView {
    let title = stat.label.to_string();
    let inner = view! {
        <span class="fui-stat-glyph">{icon(stat.glyph, "20")}</span>
        <div class="fui-stat-text">
            <div class="fui-stat-value">{stat.value}</div>
            <div class="fui-stat-label">
                <span title=title>{stat.label}</span>
            </div>
        </div>
    }
    .into_any();
    stat_frame(stat.href, inner)
}

/// Returns a stat card as [`stat_card`] draws it, with `mark` (a pill or a
/// badge that says more, such as "2 failing") after the label.
#[must_use]
pub fn stat_card_marked(stat: Stat, mark: impl IntoView + 'static) -> impl IntoView {
    let title = stat.label.to_string();
    let inner = view! {
        <span class="fui-stat-glyph">{icon(stat.glyph, "20")}</span>
        <div class="fui-stat-text">
            <div class="fui-stat-value">{stat.value}</div>
            <div class="fui-stat-label">
                <span title=title>{stat.label}</span>
                {mark}
            </div>
        </div>
    }
    .into_any();
    stat_frame(stat.href, inner)
}

/// Returns `table` in a box named `label` that shows every row.
///
/// On a wide screen the header sticks below the top bar; on a narrow one the
/// box scrolls sideways, focusable so the keyboard scrolls it (WCAG 2.2
/// SC 1.4.10, SC 2.1.1).
#[must_use]
pub fn table_card(label: impl Into<String>, table: impl IntoView) -> impl IntoView {
    view! {
        <div class="fui-table-scroll" role="region" aria-label=label.into() tabindex="0">
            {table}
        </div>
    }
}

/// Returns a compact definition grid: each label beside its value from
/// 40 rem up, one above the other below it.
#[must_use]
pub fn definitions(rows: Vec<(impl Into<String>, AnyView)>) -> impl IntoView {
    let rows = rows
        .into_iter()
        .map(|(label, value)| {
            let label = label.into();
            view! {
                <dt>{label}</dt>
                <dd>{value}</dd>
            }
        })
        .collect_view();
    view! { <dl class="fui-facts">{rows}</dl> }
}

/// Returns facts across a row, as a detail page opens: each label small and
/// upper case over its value, the columns sharing the width.
#[must_use]
pub fn facts_row(rows: Vec<(impl Into<String>, AnyView)>) -> impl IntoView {
    let rows = rows
        .into_iter()
        .map(|(label, value)| {
            let label = label.into();
            view! {
                <div>
                    <dt>{label}</dt>
                    <dd>{value}</dd>
                </div>
            }
        })
        .collect_view();
    view! { <dl class="fui-facts-row">{rows}</dl> }
}

/// Returns `value`, or the muted word "none" when it is empty, for a
/// definition whose value can be absent.
#[must_use]
pub fn or_none(value: String) -> AnyView {
    if value.is_empty() {
        view! { <span class="fui-muted">{tr(Text::NoneValue)}</span> }.into_any()
    } else {
        value.into_any()
    }
}

/// Returns `text` as a chip: a short code or name on the accent's tint, in
/// the code face.
#[must_use]
pub fn chip(text: impl Into<String>) -> impl IntoView {
    view! { <span class="fui-chip">{text.into()}</span> }
}

/// Returns a card that is one link to `href`: its title, then `body`.
///
/// A chevron after the title says the card leads somewhere; the whole card
/// takes the accent's edge under the pointer and the focus ring from the
/// keyboard. `body` holds no link or control of its own, since the card is
/// one.
#[must_use]
pub fn link_card(
    href: impl Into<String>,
    title: impl Into<String>,
    body: impl IntoView + 'static,
) -> AnyView {
    linked(href.into(), None, title.into(), body.into_any())
}

/// Returns the card [`link_card`] draws, with `glyph` before its title.
#[must_use]
pub fn link_card_glyph(
    href: impl Into<String>,
    glyph: Glyph,
    title: impl Into<String>,
    body: impl IntoView + 'static,
) -> AnyView {
    linked(href.into(), Some(glyph), title.into(), body.into_any())
}

/// The card both [`link_card`] and [`link_card_glyph`] draw.
fn linked(href: String, glyph: Option<Glyph>, title: String, body: AnyView) -> AnyView {
    view! {
        <a href=href class="fui-card fui-card-link">
            <h2 class="fui-card-title">
                {glyph.map(|glyph| icon(glyph, "16"))} <span>{title}</span>
                <span class="fui-card-go">{icon(Glyph::CHEVRON_RIGHT, "16")}</span>
            </h2>
            {body}
        </a>
    }
    .into_any()
}

/// Returns `body` in a well: a sunken box set apart inside a card.
#[must_use]
pub fn well(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class="fui-well">{body}</div> }
}

/// Returns a disclosure: `summary` the browser folds `body` under.
#[must_use]
pub fn disclosure(summary: impl Into<String>, body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! {
        <details class="fui-disclosure">
            <summary>{summary.into()}</summary>
            <div>{body}</div>
        </details>
    }
}

/// Returns `children` one above the other with the loose gap between.
#[must_use]
pub fn stack(children: impl IntoView + 'static) -> impl IntoView {
    let children = children.into_any();
    view! { <div class="fui-stack">{children}</div> }
}

/// Returns `children` in a grid of columns at least 16 rem wide that share
/// the whole row.
#[must_use]
pub fn grid(children: impl IntoView + 'static) -> impl IntoView {
    let children = children.into_any();
    view! { <div class="fui-grid">{children}</div> }
}

/// Returns `children` in the grid [`grid`] draws, each card at its own
/// height rather than stretched to its row's tallest: for cards of unlike
/// length side by side.
#[must_use]
pub fn grid_start(children: impl IntoView + 'static) -> impl IntoView {
    let children = children.into_any();
    view! { <div class="fui-grid fui-grid-start">{children}</div> }
}

/// Returns stat cards in the grid a dashboard opens with, closer together.
#[must_use]
pub fn stat_grid(children: impl IntoView + 'static) -> impl IntoView {
    let children = children.into_any();
    view! { <div class="fui-grid fui-grid-stats">{children}</div> }
}

/// Returns `children` in a wrapping row, centred on the cross axis.
#[must_use]
pub fn cluster(children: impl IntoView + 'static) -> impl IntoView {
    let children = children.into_any();
    view! { <div class="fui-cluster">{children}</div> }
}

/// Returns `start` and `end` on one wrapping row, `start` at the start and
/// `end` at the end: a header line with its actions.
#[must_use]
pub fn split(start: impl IntoView + 'static, end: impl IntoView + 'static) -> impl IntoView {
    let start = start.into_any();
    let end = end.into_any();
    view! {
        <div class="fui-split">
            <div>{start}</div>
            <div class="fui-cluster">{end}</div>
        </div>
    }
}
