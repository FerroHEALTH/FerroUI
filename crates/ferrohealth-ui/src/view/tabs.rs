// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Binder tabs: the sections of one place, joined to the one panel under
//! them, each tab a route.
//!
//! The tabs are plain links, the one for the current page marked
//! `aria-current="page"`, so they work with no script and keep their state
//! in the URL. The current tab is raised to the panel's ground and opens
//! into it, so tab and panel read as one sheet, and a card inside the panel
//! loses its own frame. A tab may carry a count, whose words name what it
//! counts, and a draft badge while its section holds a draft. A row that
//! scrolls opens with its current tab in view, and a tab whose key is
//! `keyboard` (Keyboard in Your preferences) leaves the row below 40 rem,
//! where there is no keyboard. Past [`MANY`] tabs a phone shows the
//! sections as a select once the page has hydrated, the island
//! [`crate::island::TabSelect`] over [`crate::widget::tabs::tab_select`]; before that the row of links serves. The
//! keyboard tabs that switch panels in place are [`crate::widget::tabs`].
//! No specification governs the shape: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::number::grouped;
use crate::i18n::{Arg, Text, Word, tr, tr_with};

/// The most tabs a phone shows as a row; a longer row is a select there.
pub const MANY: usize = 5;

/// What a tab counts: the number it shows and the words that name it,
/// such as `12` and "12 nodes", which assistive technology reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabCount {
    /// The number shown.
    pub number: u64,
    /// The number with its noun, in the console's words.
    pub words: String,
}

/// One binder tab.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tab {
    /// The key that names the tab, which the current one is matched by and
    /// which the tab carries as `data-tab`.
    pub key: String,
    /// The route the tab leads to.
    pub href: String,
    /// What the tab says.
    pub label: Word,
    /// What the section holds, when the template asks for a count.
    pub count: Option<TabCount>,
    /// Whether the section holds a draft.
    pub draft: bool,
}

impl Tab {
    /// The tab `key` leading to `href`, saying `label`, with no count and
    /// no draft.
    #[must_use]
    pub fn new(key: impl Into<String>, href: impl Into<String>, label: impl Into<Word>) -> Self {
        Self {
            key: key.into(),
            href: href.into(),
            label: label.into(),
            count: None,
            draft: false,
        }
    }

    /// This tab counting `number`, read as `words`.
    #[must_use]
    pub fn count(self, number: u64, words: impl Into<String>) -> Self {
        Self {
            count: Some(TabCount {
                number,
                words: words.into(),
            }),
            ..self
        }
    }

    /// This tab marked as holding a draft.
    #[must_use]
    pub fn draft(self) -> Self {
        Self {
            draft: true,
            ..self
        }
    }
}

/// One choice of the phone's tab select: where it leads and what it says.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TabChoice {
    /// The route it leads to.
    pub href: String,
    /// What it says, a draft named in it.
    pub words: Word,
}

/// Returns the tab `tab`, marked as the page when its key is `current`.
fn tab_link(tab: Tab, current: &str, draft: &Word) -> AnyView {
    let here = (tab.key == current).then_some("page");
    let count = tab.count.map(|count| {
        view! {
            <span class="fui-tab-count">
                <span aria-hidden="true">{grouped(count.number)}</span>
                <span class="fui-sr-only">{count.words}</span>
            </span>
        }
    });
    let badge = tab
        .draft
        .then(|| view! { <span class="fui-tab-draft">{draft.clone()}</span> });
    view! {
        <a href=tab.href class="fui-tab" aria-current=here data-tab=tab.key>
            {tab.label}
            {count}
            {badge}
        </a>
    }
    .into_any()
}

/// The choices of the phone's select over `tabs`: every tab but the
/// keyboard one, which a phone leaves out.
fn choices(tabs: &[Tab]) -> Vec<TabChoice> {
    tabs.iter()
        .filter(|tab| tab.key != "keyboard")
        .map(|tab| TabChoice {
            href: tab.href.clone(),
            words: if tab.draft {
                tr_with(
                    Text::TabDraftChoice,
                    &[("tab", Arg::from(tab.label.text()))],
                )
            } else {
                tab.label.clone()
            },
        })
        .collect()
}

/// A row of binder tabs, for a page template to join to its panel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabRow {
    /// The tabs' name.
    pub label: String,
    /// The tabs, in their order.
    pub tabs: Vec<Tab>,
    /// The key of the tab the page is.
    pub current: String,
}

/// Returns the binder tabs `tabs` named `label`, the one whose key is
/// `current` marked as the page, joined to `panel`.
///
/// The template decides the tabs: a page with one or two sections has
/// none, and a tab this object cannot have is left out, never disabled.
#[must_use]
pub fn binder(label: impl Into<String>, tabs: Vec<Tab>, current: &str, panel: AnyView) -> AnyView {
    joined(
        TabRow {
            label: label.into(),
            tabs,
            current: current.to_owned(),
        },
        panel,
        None,
    )
}

/// Returns the binder tabs `row` joined to `panel`, the sheet taking the
/// extra class `class` when there is one, as a page template draws them.
pub(crate) fn joined(row: TabRow, panel: AnyView, class: Option<&'static str>) -> AnyView {
    let TabRow {
        label,
        tabs,
        current,
    } = row;
    let many = tabs.len() > MANY;
    let select = many.then(|| {
        let chosen = tabs
            .iter()
            .find(|tab| tab.key == current)
            .map(|tab| tab.href.clone())
            .unwrap_or_default();
        let go = tr(Text::TabGo);
        view! { <crate::island::TabSelect label=label.clone() go=go choices=choices(&tabs) chosen /> }
    });
    let draft = tr(Text::TabDraft);
    let links = tabs
        .into_iter()
        .map(|tab| tab_link(tab, &current, &draft))
        .collect_view();
    let mut sheet = String::from("fui-tabbed");
    if many {
        sheet.push_str(" fui-tabbed-many");
    }
    if let Some(class) = class {
        sheet.push(' ');
        sheet.push_str(class);
    }
    view! {
        <div class=sheet>
            {select}
            <nav class="fui-tabs" aria-label=label>
                {links}
            </nav>
            <div class="fui-tab-panel">{panel}</div>
        </div>
    }
    .into_any()
}
