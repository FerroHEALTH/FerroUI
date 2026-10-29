// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The surfaces a page is built from.
//!
//! They are the page header with its trail and meta line, the editor's
//! header, the card and the panel with its header action, the sections of a
//! tab panel (B10), the stat card, the definition grids, the well, the chip,
//! the disclosure, and the stack, grid, cluster and split that lay them out.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Message, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::action::{ActionGroup, action_group};
use crate::view::template::list::{HeadingCount, counted};

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
    /// One line under the title with inline markup, such as a bold word
    /// and a code link (`<strong>`, `<code>`, `<a>`), drawn in place of
    /// `subtitle` when there is one. It is phrasing content: no block
    /// inside it (HTML Living Standard, the `p` element).
    pub subtitle_view: Option<Html>,
    /// The page's ancestors, the root first; none for a page with no trail.
    pub crumbs: Vec<Crumb>,
    /// Whether the title is drawn in the code face, for an identifier.
    pub mono: bool,
    /// The state of what the page shows, beside its title (a state, a pill
    /// or a mode badge), when it has one.
    pub badge: Option<Html>,
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
pub fn page_heading(locale: &Locale, header: Header, actions: ActionGroup) -> Html {
    heading(locale, header, Trail::Here, action_group(locale, actions))
}

/// Returns the header [`page_heading`] draws, with `steps` (the previous
/// and next of a queue, R5) at the far end in place of an action group.
pub(crate) fn stepped_heading(locale: &Locale, header: Header, steps: Html) -> Html {
    heading(locale, header, Trail::Here, steps)
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "trail")]
struct TrailView<'a> {
    breadcrumb: Word,
    here: bool,
    crumbs: Vec<(Crumb, bool)>,
    back: Html,
    title: &'a str,
}

/// Returns the trail `crumbs` ending as `trail` says, after them the
/// current page `title` when it ends there.
fn trail_of(locale: &Locale, crumbs: Vec<Crumb>, trail: Trail, title: &str) -> Html {
    let last = crumbs.len().saturating_sub(1);
    let here = trail == Trail::Here;
    TrailView {
        breadcrumb: tr(locale, Text::Breadcrumb),
        here,
        crumbs: crumbs
            .into_iter()
            .enumerate()
            .map(|(at, crumb)| (crumb, here || at < last))
            .collect(),
        back: icon(Glyph::ARROW_LEFT, "14"),
        title,
    }
    .html()
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "heading")]
struct HeadingView {
    trail: Option<Html>,
    badge: Option<Html>,
    mono: bool,
    title: Word,
    actions: Html,
    subtitle: Option<Html>,
}

/// The header [`page_heading`] and [`editor_heading`] draw.
fn heading(locale: &Locale, header: Header, trail: Trail, actions: Html) -> Html {
    let Header {
        title,
        subtitle,
        subtitle_view,
        crumbs,
        mono,
        badge,
    } = header;
    let trail = (!crumbs.is_empty()).then(|| trail_of(locale, crumbs, trail, title.text()));
    HeadingView {
        trail,
        badge,
        mono,
        title,
        actions,
        subtitle: subtitle_view.or_else(|| subtitle.map(Html::from)),
    }
    .html()
}

/// Returns the header that opens a page: its `title`, one `subtitle` line
/// when there is one, and the action group `actions` at the far end.
#[must_use]
pub fn page_header(
    locale: &Locale,
    title: impl Into<Word>,
    subtitle: Option<String>,
    actions: ActionGroup,
) -> Html {
    page_heading(
        locale,
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
pub fn editor_title(locale: &Locale, message: Message, mode: &Mode) -> String {
    let name = match mode {
        Mode::Creating => String::new(),
        Mode::Editing(name) => name.clone(),
    };
    message.text_with(
        locale,
        &[("mode", mode.key().into()), ("name", name.into())],
    )
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
    locale: &Locale,
    message: Message,
    mode: &Mode,
    header: Header,
    actions: ActionGroup,
) -> Html {
    let title = editor_title(locale, message, mode);
    heading(
        locale,
        Header {
            title: Word::new(title),
            mono: false,
            ..header
        },
        Trail::Parent,
        action_group(locale, actions),
    )
}

/// `body` between the markup `open` and `close`, both the library's own.
fn wrap(open: &str, body: Html, close: &str) -> Html {
    let mut out = String::from(open);
    out.push_str(&String::from(body));
    out.push_str(close);
    Html::raw(out)
}

/// Returns the small muted line under a page's header that says who
/// answered the page (a replica, a server); nothing for no line.
#[must_use]
pub fn page_meta(line: Option<String>) -> Html {
    line.map(|line| wrap("<p class=\"fui-page-meta\">", Html::text(line), "</p>"))
        .unwrap_or_default()
}

/// Returns a card titled `title` holding `body`, a section under a
/// level-two heading.
#[must_use]
pub fn card(title: impl Into<String>, body: Html) -> Html {
    wrap(
        &format!(
            "<section class=\"fui-card\"><h2 class=\"fui-card-title\">{}</h2>",
            Html::text(title.into())
        ),
        body,
        "</section>",
    )
}

/// Returns a card titled `title` holding `body` that spans the whole row of
/// the grid it sits in.
#[must_use]
pub fn wide_card(title: impl Into<String>, body: Html) -> Html {
    wrap(
        &format!(
            "<section class=\"fui-card fui-card-wide\"><h2 class=\"fui-card-title\">{}</h2>",
            Html::text(title.into())
        ),
        body,
        "</section>",
    )
}

/// Returns a card holding `body` with no heading of its own.
#[must_use]
pub fn panel(body: Html) -> Html {
    wrap("<div class=\"fui-card\">", body, "</div>")
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "titled_panel")]
struct TitledPanel {
    title: String,
    actions: Html,
    body: Html,
}

/// Returns a panel: a card whose first row holds its title and actions.
///
/// It stands on a page of its own; inside a tab panel the parts are
/// [`panel_section`]s, never a card in the panel.
///
/// `title` and `actions` sit over `body`, which runs to the card's edges (a table, a list). Pass a
/// `body` with its own padding through [`panel_body`] when it is prose.
#[must_use]
pub fn titled_panel(title: impl Into<String>, actions: Html, body: Html) -> Html {
    TitledPanel {
        title: title.into(),
        actions,
        body,
    }
    .html()
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "card_action")]
struct CardAction {
    href: String,
    label: String,
    glyph: Html,
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
pub fn card_action(href: impl Into<String>, label: impl Into<String>) -> Html {
    CardAction {
        href: href.into(),
        label: label.into(),
        glyph: icon(Glyph::ARROW_RIGHT, "14"),
    }
    .html()
}

/// Returns `body` with the card's padding, for a panel's prose.
#[must_use]
pub fn panel_body(body: Html) -> Html {
    wrap("<div class=\"fui-panel-body\">", body, "</div>")
}

/// What a panel section's heading row shows.
#[derive(Debug, Default)]
pub struct SectionHead {
    /// The section's title, a level-three heading in the page outline.
    pub title: String,
    /// The count after the title, muted after a middle dot, when the
    /// section holds a list: every item, or the items shown of a larger set.
    pub count: Option<HeadingCount>,
    /// The section's actions at the row's end.
    pub actions: Option<Html>,
}

/// Returns the parts of a tab panel, `sections` drawn by [`panel_section`],
/// one above the other with a hairline between (B10).
///
/// The tab panel is their one frame: no part is a card inside it, on a
/// desktop or on a phone.
#[must_use]
pub fn panel_sections(sections: Html) -> Html {
    wrap("<div class=\"fui-panel\">", sections, "</div>")
}

/// Returns `sections` drawn by [`panel_section`] in a card of their own,
/// named `label`, for a page with no tabs: the card is their one frame.
#[must_use]
pub fn panel_sections_card(label: impl Into<String>, sections: Html) -> Html {
    wrap(
        &format!(
            "<section aria-label=\"{}\" class=\"fui-card fui-panel\">",
            Html::text(label.into())
        ),
        sections,
        "</section>",
    )
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "panel_section")]
struct PanelSection {
    title: Option<Html>,
    actions: Option<Html>,
    body: Html,
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
pub fn panel_section(locale: &Locale, head: Option<SectionHead>, body: Html) -> Html {
    let (title, actions) = match head {
        Some(SectionHead {
            title,
            count,
            actions,
        }) => (
            Some(match count {
                Some(count) => counted(locale, title, count),
                None => Html::text(title),
            }),
            actions,
        ),
        None => (None, None),
    };
    PanelSection {
        title,
        actions,
        body,
    }
    .html()
}

/// Returns the one sentence under a section's heading that says what the
/// section holds.
#[must_use]
pub fn section_intro(sentence: impl Into<String>) -> Html {
    wrap(
        "<p class=\"fui-small fui-muted fui-section-intro\">",
        Html::text(sentence.into()),
        "</p>",
    )
}

/// Returns the row of Add buttons under a list of rows in a section.
#[must_use]
pub fn add_row(buttons: Html) -> Html {
    wrap("<div class=\"fui-cluster fui-add-row\">", buttons, "</div>")
}

/// Returns a file beside its diagnostics in a tab panel: `file`, a
/// [`crate::view::code::code_viewer`] with its head, and `side`, a
/// [`panel_section`], split by a hairline and stacked under 64 rem.
#[must_use]
pub fn source_split(file: Html, side: Html) -> Html {
    wrap(
        "<div class=\"fui-panel fui-source-split\">",
        file.then(side),
        "</div>",
    )
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

#[derive(Template)]
#[template(path = "surface/surface.html", block = "stat")]
struct StatView {
    href: Option<String>,
    glyph: Html,
    value: String,
    label: Cow<'static, str>,
    mark: Html,
}

/// Returns a stat card: the glyph in a square on the accent's tint, the
/// value large in tabular figures, the label under it, the whole card a
/// link when the stat leads to a page.
///
/// Below 30 rem the card is a compact tile, the glyph, the value and the
/// label on one line, the label cut with an ellipsis and named whole in its
/// `title`.
#[must_use]
pub fn stat_card(stat: Stat) -> Html {
    stat_card_marked(stat, Html::empty())
}

/// Returns a stat card as [`stat_card`] draws it, with `mark` (a pill or a
/// badge that says more, such as "2 failing") after the label.
#[must_use]
pub fn stat_card_marked(stat: Stat, mark: Html) -> Html {
    StatView {
        href: stat.href,
        glyph: icon(stat.glyph, "20"),
        value: stat.value,
        label: stat.label,
        mark,
    }
    .html()
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "definitions")]
struct Definitions {
    rows: Vec<(Word, Html)>,
}

/// Returns a compact definition grid: each label beside its value from
/// 40 rem up, one above the other below it.
#[must_use]
pub fn definitions(rows: Vec<(impl Into<Word>, Html)>) -> Html {
    Definitions {
        rows: rows
            .into_iter()
            .map(|(label, value)| (label.into(), value))
            .collect(),
    }
    .html()
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "facts_row")]
struct FactsRow {
    rows: Vec<(String, Html)>,
}

/// Returns facts across a row, as a detail page opens: each label small and
/// upper case over its value, the columns sharing the width.
#[must_use]
pub fn facts_row(rows: Vec<(impl Into<String>, Html)>) -> Html {
    FactsRow {
        rows: rows
            .into_iter()
            .map(|(label, value)| (label.into(), value))
            .collect(),
    }
    .html()
}

/// Returns `value`, or the muted word "none" when it is empty, for a
/// definition whose value can be absent.
#[must_use]
pub fn or_none(locale: &Locale, value: String) -> Html {
    if value.is_empty() {
        wrap(
            "<span class=\"fui-muted\">",
            tr(locale, Text::NoneValue).into(),
            "</span>",
        )
    } else {
        Html::text(value)
    }
}

/// Returns `text` as a chip: a short code, such as a path or a scope, on
/// the accent's tint, in the code face.
///
/// A kind, a format, an owner or a namespace is a tag
/// ([`crate::view::tag::tag`]), in the UI face.
#[must_use]
pub fn chip(text: impl Into<String>) -> Html {
    wrap(
        "<span class=\"fui-chip\">",
        Html::text(text.into()),
        "</span>",
    )
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "linked")]
struct Linked {
    href: String,
    glyph: Html,
    title: String,
    go: Html,
    body: Html,
}

/// Returns a card that is one link to `href`: its title, then `body`.
///
/// A chevron after the title says the card leads somewhere; the whole card
/// takes the accent's edge under the pointer and the focus ring from the
/// keyboard. `body` holds no link or control of its own, since the card is
/// one.
#[must_use]
pub fn link_card(href: impl Into<String>, title: impl Into<String>, body: Html) -> Html {
    linked(href.into(), None, title.into(), body)
}

/// Returns the card [`link_card`] draws, with `glyph` before its title.
#[must_use]
pub fn link_card_glyph(
    href: impl Into<String>,
    glyph: Glyph,
    title: impl Into<String>,
    body: Html,
) -> Html {
    linked(href.into(), Some(glyph), title.into(), body)
}

/// The card both [`link_card`] and [`link_card_glyph`] draw.
fn linked(href: String, glyph: Option<Glyph>, title: String, body: Html) -> Html {
    Linked {
        href,
        glyph: glyph.map(|glyph| icon(glyph, "16")).unwrap_or_default(),
        title,
        go: icon(Glyph::CHEVRON_RIGHT, "16"),
        body,
    }
    .html()
}

/// Returns `body` in a well: a sunken box set apart inside a card.
#[must_use]
pub fn well(body: Html) -> Html {
    wrap("<div class=\"fui-well\">", body, "</div>")
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "disclosure")]
struct Disclosure {
    summary: String,
    body: Html,
}

/// Returns a disclosure: `summary` the browser folds `body` under.
#[must_use]
pub fn disclosure(summary: impl Into<String>, body: Html) -> Html {
    Disclosure {
        summary: summary.into(),
        body,
    }
    .html()
}

/// Returns `children` one above the other with the loose gap between.
#[must_use]
pub fn stack(children: Html) -> Html {
    wrap("<div class=\"fui-stack\">", children, "</div>")
}

/// Returns `children` in a grid of columns at least 16 rem wide that share
/// the whole row.
#[must_use]
pub fn grid(children: Html) -> Html {
    wrap("<div class=\"fui-grid\">", children, "</div>")
}

/// Returns `children` in the grid [`grid`] draws, each card at its own
/// height rather than stretched to its row's tallest: for cards of unlike
/// length side by side.
#[must_use]
pub fn grid_start(children: Html) -> Html {
    wrap(
        "<div class=\"fui-grid fui-grid-start\">",
        children,
        "</div>",
    )
}

/// Returns stat cards in the grid a dashboard opens with, closer together.
#[must_use]
pub fn stat_grid(children: Html) -> Html {
    wrap(
        "<div class=\"fui-grid fui-grid-stats\">",
        children,
        "</div>",
    )
}

/// Returns `children` in a wrapping row, centred on the cross axis.
#[must_use]
pub fn cluster(children: Html) -> Html {
    wrap("<div class=\"fui-cluster\">", children, "</div>")
}

#[derive(Template)]
#[template(path = "surface/surface.html", block = "split")]
struct Split {
    start: Html,
    end: Html,
}

/// Returns `start` and `end` on one wrapping row, `start` at the start and
/// `end` at the end: a header line with its actions.
#[must_use]
pub fn split(start: Html, end: Html) -> Html {
    Split { start, end }.html()
}
