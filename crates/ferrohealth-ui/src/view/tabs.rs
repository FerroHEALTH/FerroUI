// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Binder tabs: the sections of one place, joined to the one panel under
//! them, each tab a route (R3).
//!
//! The tabs are plain links, the one for the current page marked
//! `aria-current="page"`, so they work with no script and keep their state
//! in the URL. The current tab is raised to the panel's ground and opens
//! into it, so tab and panel read as one sheet. A tab may carry a count and
//! a draft badge. A tab whose panel the page already holds names it
//! ([`Tab::panel`]): the `tabs` module then switches to it in place, with
//! no document load, and the address follows. Past [`MANY`] tabs a phone
//! shows the sections as a select once the page has script; before that
//! the row of links serves. The keyboard tabs that switch panels in place
//! are [`keyboard_tabs`]. No specification governs the shape: our own
//! design.

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::number::grouped;
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
use crate::view::button::Variant;
use crate::view::form::SELECT;

/// The most tabs a phone shows as a row; a longer row is a select there.
pub const MANY: usize = 5;

/// What a tab counts: the number it shows and the words that name it.
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
    /// The id of the panel the page already holds for this tab, which the
    /// `tabs` module shows in place; `None` for a tab whose panel the
    /// route draws.
    pub panel: Option<String>,
}

impl Tab {
    /// The tab `key` leading to `href`, saying `label`.
    #[must_use]
    pub fn new(key: impl Into<String>, href: impl Into<String>, label: impl Into<Word>) -> Self {
        Self {
            key: key.into(),
            href: href.into(),
            label: label.into(),
            count: None,
            draft: false,
            panel: None,
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

    /// This tab, whose panel the page already holds under the id `panel`,
    /// so a click switches to it in place.
    #[must_use]
    pub fn panel(self, panel: impl Into<String>) -> Self {
        Self {
            panel: Some(panel.into()),
            ..self
        }
    }
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

struct TabView {
    key: String,
    href: String,
    current: bool,
    panel: Option<String>,
    label: Word,
    count: Option<(String, String)>,
    draft: bool,
}

#[derive(Template)]
#[template(path = "tabs/binder.html")]
struct Binder {
    sheet: String,
    label: String,
    select: Option<SelectView>,
    tabs: Vec<TabView>,
    draft: Word,
    panel: Html,
}

struct SelectView {
    go: Word,
    select_class: &'static str,
    button_class: &'static str,
    choices: Vec<(String, String, bool, Option<&'static str>)>,
}

/// Returns the binder tabs `tabs` named `label`, the one whose key is
/// `current` marked as the page, joined to `panel`. A page with one or two
/// sections has none, and a tab this object cannot have is left out, never
/// disabled.
#[must_use]
pub fn binder(locale: &Locale, label: impl Into<String>, tabs: Vec<Tab>, current: &str, panel: Html) -> Html {
    joined(
        locale,
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
pub(crate) fn joined(locale: &Locale, row: TabRow, panel: Html, class: Option<&'static str>) -> Html {
    let TabRow {
        label,
        tabs,
        current,
    } = row;
    let many = tabs.len() > MANY;
    let select = many.then(|| SelectView {
        go: tr(locale, Text::TabGo),
        select_class: SELECT,
        button_class: Variant::Secondary.class(),
        choices: tabs
            .iter()
            .filter(|tab| tab.key != "keyboard")
            .map(|tab| {
                let words = if tab.draft {
                    tr_with(
                        locale,
                        Text::TabDraftChoice,
                        &[("tab", Arg::from(tab.label.text()))],
                    )
                } else {
                    tab.label.clone()
                };
                (
                    tab.href.clone(),
                    words.text().to_owned(),
                    tab.key == current,
                    words.lang(),
                )
            })
            .collect(),
    });
    let mut sheet = String::from("fui-tabbed");
    if many {
        sheet.push_str(" fui-tabbed-many");
    }
    if let Some(class) = class {
        sheet.push(' ');
        sheet.push_str(class);
    }
    Binder {
        sheet,
        label,
        select,
        tabs: tabs
            .into_iter()
            .map(|tab| TabView {
                current: tab.key == current,
                key: tab.key,
                href: tab.href,
                panel: tab.panel,
                label: tab.label,
                count: tab
                    .count
                    .map(|count| (grouped(locale, count.number), count.words)),
                draft: tab.draft,
            })
            .collect(),
        draft: tr(locale, Text::TabDraft),
        panel,
    }
    .html()
}

/// The tab a key moves to from `current` of `count`, or `None` for a key
/// the keyboard tabs do not take.
#[must_use]
pub fn moved(key: &str, current: usize, count: usize) -> Option<usize> {
    let last = count.checked_sub(1)?;
    match key {
        "ArrowRight" => Some(if current >= last { 0 } else { current + 1 }),
        "ArrowLeft" => Some(if current == 0 { last } else { current - 1 }),
        "Home" => Some(0),
        "End" => Some(last),
        _ => None,
    }
}

/// Returns the keyboard tabs `prefix` named `label`: one tab per word in
/// `tabs` over its panel in `panels`, the first shown and the others
/// hidden, switched in place by the `tabs` module with the keys of the
/// WAI-ARIA Authoring Practices tabs pattern (automatic activation).
///
/// Every panel is in the page, so a reader with no script reads the first;
/// where every panel must be reachable with no script a page uses
/// [`binder`], whose tabs are routes.
#[must_use]
pub fn keyboard_tabs(label: &str, prefix: &str, tabs: Vec<String>, panels: Vec<Html>) -> Html {
    let buttons: String = tabs
        .iter()
        .enumerate()
        .map(|(index, words)| {
            let on = index == 0;
            format!(
                "<button type=\"button\" role=\"tab\" id=\"{prefix}-tab-{index}\" class=\"fui-tab\" aria-controls=\"{prefix}-panel-{index}\" aria-selected=\"{on}\" tabindex=\"{}\">{}</button>",
                if on { "0" } else { "-1" },
                Html::text(words)
            )
        })
        .collect();
    let panels: String = panels
        .into_iter()
        .enumerate()
        .map(|(index, body)| {
            format!(
                "<div role=\"tabpanel\" id=\"{prefix}-panel-{index}\" class=\"fui-tab-panel\" aria-labelledby=\"{prefix}-tab-{index}\"{} tabindex=\"0\">{body}</div>",
                if index == 0 { "" } else { " hidden" }
            )
        })
        .collect();
    Html::raw(format!(
        "<div class=\"fui-tabbed\"><div role=\"tablist\" aria-label=\"{}\" class=\"fui-tabs\" data-fui=\"tabs\">{buttons}</div>{panels}</div>",
        Html::text(label)
    ))
}

#[cfg(test)]
mod tests {
    use super::{Tab, binder, moved};
    use crate::html::Html;
    use crate::i18n::Locale;

    #[test]
    fn the_arrows_wrap_and_home_and_end_go_to_the_ends() {
        assert_eq!(Some(1), moved("ArrowRight", 0, 3));
        assert_eq!(Some(0), moved("ArrowRight", 2, 3), "wraps forward");
        assert_eq!(Some(2), moved("ArrowLeft", 0, 3), "wraps back");
        assert_eq!(Some(0), moved("Home", 2, 3));
        assert_eq!(Some(2), moved("End", 0, 3));
        assert_eq!(None, moved("Enter", 0, 3));
        assert_eq!(None, moved("End", 0, 0));
    }

    #[test]
    fn a_binder_marks_the_current_tab_and_names_an_in_page_panel() {
        let en = Locale::english();
        let tabs = vec![
            Tab::new("overview", "/t", "Overview").panel("panel-overview"),
            Tab::new("paths", "/t?tab=paths", "Paths").count(12, "12 nodes"),
        ];
        let html = binder(&en, "Template views", tabs, "paths", Html::text("body"));
        let html = html.as_str();
        assert!(html.contains(r#"<a href="/t" class="fui-tab" data-tab="overview" data-fui-panel="panel-overview">"#), "{html}");
        assert!(html.contains(r#"href="/t?tab=paths" class="fui-tab" aria-current="page" data-tab="paths">"#), "{html}");
        assert!(html.contains("12 nodes"), "{html}");
        assert!(html.contains("data-fui-tab-keys"), "{html}");
        assert!(!html.contains("fui-tabs-select"), "{html}");
    }
}
