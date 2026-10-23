// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The surfaces a page is built from.
//!
//! They are the page header with its trail and meta line, the editor's
//! header, the card and the panel with its header action, the sections of a
//! tab panel (B10), the stat card, the definition grids, the well, the chip,
//! the disclosure, and the stack, grid, cluster and split that lay them out.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Message, Text, Word, tr};

use crate::icon::{Glyph, icon};
use crate::view::action::{ActionGroup, action_group};

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
#[derive(Debug, Default)]
pub struct Header {
    /// The page's title, also the trail's last, current, crumb.
    pub title: Word,
    /// One line under the title, when there is one.
    pub subtitle: Option<Word>,
    /// The page's ancestors, the root first; none for a page with no trail.
    pub crumbs: Vec<Crumb>,
    /// Whether the title is drawn in the code face, for an identifier.
    pub mono: bool,
    /// The state of what the page shows, beside its title (a state, a pill
    /// or a mode badge), when it has one.
    pub badge: Option<AnyView>,
}

/// How a page's trail ends.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Trail {
    /// With the page itself, the current crumb, after its ancestors.
    Here,
    /// With the thing an editor edits or the list a new thing joins, so
    /// the trail never ends with the editor itself.
    Parent,
}

/// Returns the header that opens a page: the trail of its ancestors when it
/// has one, its title with its badge and the action group `actions` (F14)
/// at the far end, and its subtitle line.
///
/// The trail is a `nav` named "Breadcrumb" holding an ordered list, the
/// current page last and marked `aria-current="page"`, the separators hidden
/// from assistive technology (WAI-ARIA Authoring Practices, the breadcrumb
/// pattern). Below 40 rem it is one back link to the parent, the title
/// under it saying where you are.
#[must_use]
pub fn page_heading(header: Header, actions: ActionGroup) -> impl IntoView {
    heading(header, Trail::Here, action_group(actions))
}

/// Returns the header [`page_heading`] draws, with `steps` (the previous
/// and next of a queue, R5) at the far end in place of an action group.
pub(crate) fn stepped_heading(header: Header, steps: AnyView) -> AnyView {
    heading(header, Trail::Here, steps).into_any()
}

/// Returns a crumb's link, with the arrow it shows as a back link.
fn crumb_link(crumb: Crumb) -> impl IntoView {
    view! {
        <a href=crumb.href>
            <span class="fui-crumb-back" aria-hidden="true">
                {icon(Glyph::ARROW_LEFT, "14")}
            </span>
            {crumb.label}
        </a>
    }
}

/// Returns the separator after a crumb, hidden from assistive technology.
fn crumb_sep() -> impl IntoView {
    view! {
        <span class="fui-crumb-sep" aria-hidden="true">
            "/"
        </span>
    }
}

/// Returns the trail `crumbs` ending as `trail` says, after them the
/// current page `title` when it ends there.
fn trail_of(crumbs: Vec<Crumb>, trail: Trail, title: &str) -> AnyView {
    let last = crumbs.len().saturating_sub(1);
    let links = crumbs
        .into_iter()
        .enumerate()
        .map(|(at, crumb)| {
            let sep = (trail == Trail::Here || at < last).then(crumb_sep);
            view! { <li>{crumb_link(crumb)} {sep}</li> }
        })
        .collect_view();
    match trail {
        Trail::Here => view! {
            <nav class="fui-crumbs" aria-label=tr(Text::Breadcrumb)>
                <ol>{links} <li aria-current="page">{title.to_owned()}</li></ol>
            </nav>
        }
        .into_any(),
        Trail::Parent => view! {
            <nav class="fui-crumbs is-parent" aria-label=tr(Text::Breadcrumb)>
                <ol>{links}</ol>
            </nav>
        }
        .into_any(),
    }
}

/// The header [`page_heading`] and [`editor_heading`] draw.
fn heading(header: Header, trail: Trail, actions: AnyView) -> impl IntoView {
    let Header {
        title,
        subtitle,
        crumbs,
        mono,
        badge,
    } = header;
    let trail = (!crumbs.is_empty()).then(|| trail_of(crumbs, trail, title.text()));
    // Two arms, so a title in the body face writes no `class` attribute.
    let h1 = if mono {
        view! { <h1 class="fui-page-title-mono">{title}</h1> }.into_any()
    } else {
        view! { <h1>{title}</h1> }.into_any()
    };
    let title_line = match badge {
        Some(badge) => view! { <div class="fui-page-title-line">{h1} {badge}</div> }.into_any(),
        None => h1,
    };
    view! {
        <header class="fui-page-header">
            {trail} <div class="fui-page-heading">
                {title_line}
                <div class="fui-actions">{actions}</div>
            </div>
            {subtitle.map(|line| view! { <p class="fui-page-subtitle">{line}</p> })}
        </header>
    }
}

/// Returns the header that opens a page: its `title`, one `subtitle` line
/// when there is one, and the action group `actions` at the far end.
#[must_use]
pub fn page_header(
    title: impl Into<Word>,
    subtitle: Option<String>,
    actions: ActionGroup,
) -> impl IntoView {
    page_heading(
        Header {
            title: title.into(),
            subtitle: subtitle.map(Word::from),
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

/// The title of an editor in `mode`, such as "New code system" or "Edit".
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
///        *[edit] Edit
///     }
/// ```
///
/// The heading's trail names the thing edited, so the title need not; a
/// console whose document title must name it words a second message with
/// `$name`.
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
/// Its title is [`editor_title`]: "New code system" while creating, and
/// the console's words for a change ("Edit") while editing. Its trail never
/// ends with the editor: pass in `header.crumbs` the list a new thing
/// joins, or the trail down to the thing edited, that thing last, and on a
/// phone that last crumb is the back link.
///
/// `header.title` and `header.mono` are not read: the title is a sentence,
/// so it is drawn in the body face even around an identifier.
#[must_use]
pub fn editor_heading(
    message: Message,
    mode: &Mode,
    header: Header,
    actions: ActionGroup,
) -> AnyView {
    let title = editor_title(message, mode);
    heading(
        Header {
            title: Word::new(title),
            mono: false,
            ..header
        },
        Trail::Parent,
        action_group(actions),
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
/// It stands on a page of its own; inside a tab panel the parts are
/// [`panel_section`]s, never a card in the panel.
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

/// What a panel section's heading row shows.
#[derive(Debug, Default)]
pub struct SectionHead {
    /// The section's title, a level-three heading in the page outline.
    pub title: String,
    /// The count after the title, muted after a middle dot, when the
    /// section holds a list.
    pub count: Option<u64>,
    /// The section's actions at the row's end.
    pub actions: Option<AnyView>,
}

/// Returns the parts of a tab panel, `sections` drawn by [`panel_section`],
/// one above the other with a hairline between (B10).
///
/// The tab panel is their one frame: no part is a card inside it, on a
/// desktop or on a phone.
#[must_use]
pub fn panel_sections(sections: impl IntoView + 'static) -> AnyView {
    let sections = sections.into_any();
    view! { <div class="fui-panel">{sections}</div> }.into_any()
}

/// Returns `sections` drawn by [`panel_section`] in a card of their own,
/// named `label`, for a page with no tabs: the card is their one frame.
#[must_use]
pub fn panel_sections_card(label: impl Into<String>, sections: impl IntoView + 'static) -> AnyView {
    let sections = sections.into_any();
    view! {
        <section class="fui-card fui-panel" aria-label=label.into()>
            {sections}
        </section>
    }
    .into_any()
}

/// Returns one part of a tab panel (B10): its heading row, when it has
/// one, over `body`, with a hairline above it after the first part.
///
/// The heading row holds the title, the count and the actions. A table, a
/// diagnostics list or a code viewer in `body` runs edge to edge to the
/// panel's frame; prose keeps the section's inset. Each title is a real
/// heading in the page outline (WCAG 2.2 SC 1.3.1). No specification
/// governs the layout: our own design.
#[must_use]
pub fn panel_section(head: Option<SectionHead>, body: impl IntoView + 'static) -> AnyView {
    let body = body.into_any();
    let head = head.map(
        |SectionHead {
             title,
             count,
             actions,
         }| {
            let title = match count {
                Some(count) => crate::view::template::list::counted(title, count),
                None => title.into_any(),
            };
            let actions = actions.map(|actions| view! { <div class="fui-actions">{actions}</div> });
            view! {
                <div class="fui-panel-section-head">
                    <h3 class="fui-panel-section-title">{title}</h3>
                    {actions}
                </div>
            }
        },
    );
    view! { <section class="fui-panel-section">{head} {body}</section> }.into_any()
}

/// Returns the one sentence under a section's heading that says what the
/// section holds.
#[must_use]
pub fn section_intro(sentence: impl Into<String>) -> AnyView {
    view! { <p class="fui-small fui-muted fui-section-intro">{sentence.into()}</p> }.into_any()
}

/// Returns the row of Add buttons under a list of rows in a section.
#[must_use]
pub fn add_row(buttons: impl IntoView + 'static) -> AnyView {
    let buttons = buttons.into_any();
    view! { <div class="fui-cluster fui-add-row">{buttons}</div> }.into_any()
}

/// Returns a file beside its diagnostics in a tab panel: `file`, a
/// [`crate::view::code::code_viewer`] with its head, and `side`, a
/// [`panel_section`], split by a hairline and stacked under 64 rem.
#[must_use]
pub fn source_split(file: AnyView, side: AnyView) -> AnyView {
    view! { <div class="fui-panel fui-source-split">{file} {side}</div> }.into_any()
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

/// Returns a compact definition grid: each label beside its value from
/// 40 rem up, one above the other below it.
#[must_use]
pub fn definitions(rows: Vec<(impl Into<Word>, AnyView)>) -> impl IntoView {
    let rows = rows
        .into_iter()
        .map(|(label, value)| {
            let label: Word = label.into();
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

/// Returns `text` as a chip: a short code, such as a path or a scope, on
/// the accent's tint, in the code face.
///
/// A kind, a format, an owner or a namespace is a tag
/// ([`crate::view::tag::tag`]), in the UI face.
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
