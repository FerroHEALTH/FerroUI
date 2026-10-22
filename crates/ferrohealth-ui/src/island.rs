// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's islands: the widgets that own their state and need nothing
//! from a console's code, each ready to place in a page Leptos renders in
//! islands mode.
//!
//! They are the toast region, the keyboard tabs, the navigation drawer, the
//! menu and listbox behind every More, row menu and select button, the
//! combobox, the picker field, the chart legend, the copy button and the
//! side panel.
//!
//! An island is the only part of a page the browser hydrates (the Leptos
//! book, "Guide: Islands"). Its props cross to the browser as JSON in the
//! page, so they are plain data, and the words it draws are among them,
//! resolved on the server in the page's locale. Each island here is
//! a thin wrapper over its widget function in [`crate::widget`], so a
//! console that needs the widget beside state of its own calls that function
//! inside an island of its own instead.
#![expect(
    clippy::must_use_candidate,
    reason = "the #[island] macro drops a #[must_use] from the function it writes"
)]
#![expect(
    missing_docs,
    reason = "the #[island] macro writes an undocumented struct of each island's props as they cross to the browser"
)]
#![expect(
    clippy::mem_forget,
    reason = "the island macro of leptos_macro 0.8 keeps each hydrated island alive by forgetting its view state, in the browser entry it writes"
)]

use leptos::prelude::*;

use crate::view::tabs::TabChoice;
use crate::widget::action_menu::Trigger;
use crate::widget::combobox::{ComboOption, ComboboxWords, KeyedCombobox};
use crate::widget::toast::{Flash, RegionWords};

/// The toast region, which draws `flash` as the page arrives and every
/// toast [`crate::widget::toast::raise`] shows after. A console draws it
/// once, in its shell.
#[island]
pub fn ToastRegion(
    /// The words it draws, in the page's locale
    /// ([`RegionWords::of_page`]).
    words: RegionWords,
    /// The toasts the server shows as the page arrives.
    flash: Vec<Flash>,
) -> impl IntoView {
    crate::widget::toast::toast_region(words, flash)
}

/// The keyboard tabs `prefix`, named `label`, one tab per word in `tabs`,
/// over the panels [`crate::widget::tabs::tab_panels`] drew as its
/// children.
#[island]
pub fn KeyboardTabs(
    /// The tab list's name, in the page's locale.
    label: String,
    /// The prefix of every tab's and panel's id.
    prefix: String,
    /// Each tab's words, in the page's locale.
    tabs: Vec<String>,
    /// The panels, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::tabs::tab_list(label, prefix, tabs, children().into_any())
}

/// The view switch of the document pane `id`, named `label`, one link per
/// choice in `views`, over the views the server drew as its children; the
/// view `opening` shows first. A console draws it through
/// [`crate::view::document::document_pane`].
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn DocumentViews(
    /// The pane's id, which each view's id starts with.
    id: String,
    /// The switch's name, in the page's locale.
    label: String,
    /// The views offered, each with its words in the page's locale.
    views: Vec<crate::widget::document::ViewChoice>,
    /// The view shown first.
    opening: crate::widget::document::DocumentView,
    /// The views, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::document::view_switch(&id, label, views, opening, children().into_any())
}

/// The key hint of a setting, copying its `key` when pressed (R15). The
/// settings template draws it on each setting's label line.
#[island]
pub fn KeyHint(
    /// The configuration key it names and copies.
    key: String,
    /// Its words, in the page's locale ([`crate::widget::copy::KeyHintWords::of_page`]).
    words: crate::widget::copy::KeyHintWords,
) -> impl IntoView {
    crate::widget::copy::key_hint(key, words)
}

/// The copy button `id` saying `label`, putting `text` on the clipboard
/// when pressed ([`crate::widget::copy`]). The server draws it hidden, so
/// with no WebAssembly the text stays on the page to select.
#[island]
pub fn CopyButton(
    /// The button's id.
    id: String,
    /// Its words, in the page's locale, such as "Copy".
    label: String,
    /// The text it copies, which the page already shows.
    text: String,
    /// The words it draws itself, in the page's locale
    /// ([`crate::widget::copy::CopyWords::of_page`]).
    words: crate::widget::copy::CopyWords,
) -> impl IntoView {
    crate::widget::copy::copy_button(id, label, Signal::stored(text), words)
}

/// The side panel `id` headed `title`, around the body the server drew as
/// its children, and its `opener`: a link to the content's own route until
/// the page hydrates, then the button that opens the panel
/// ([`crate::widget::side_panel`]).
#[island]
pub fn SidePanel(
    /// The panel's id.
    id: String,
    /// The heading, in the page's locale.
    title: String,
    /// What opens it, with the route of its content.
    opener: crate::widget::side_panel::Opener,
    /// The words it draws itself, in the page's locale
    /// ([`crate::widget::side_panel::SidePanelWords::of_page`]).
    words: crate::widget::side_panel::SidePanelWords,
    /// The body, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::side_panel::side_panel(crate::widget::side_panel::SidePanel {
        id,
        title,
        opener,
        body: children().into_any(),
        words,
    })
}

/// The phone's select over binder tabs, named `label`, on the choice that
/// leads to `chosen`, with the submit button saying `go`. A console draws it through
/// [`crate::view::tabs::binder`], which places it past
/// [`crate::view::tabs::MANY`] tabs.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn TabSelect(
    /// The tabs' name, in the page's locale.
    label: String,
    /// The submit button's words, in the page's locale.
    go: String,
    /// Each tab's route and words, in the page's locale.
    choices: Vec<TabChoice>,
    /// The route of the current tab.
    chosen: String,
) -> impl IntoView {
    crate::widget::tabs::tab_select(label, go, choices, &chosen)
}

/// The navigation drawer `id`, named `label`, around the entries the
/// server drew as its children, its close button saying `close`, folded to
/// its glyphs while `rail`. A console draws it through
/// [`crate::widget::nav::sidebar`].
#[island]
pub fn NavDrawer(
    /// The navigation's id, which the drawer button opens.
    id: String,
    /// The navigation's name, in the page's locale.
    label: String,
    /// The close button's words, in the page's locale.
    close: String,
    /// Whether the navigation is folded to its rail of glyphs.
    rail: bool,
    /// The groups of entries, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::nav::drawer(id, label, close, rail, children().into_any())
}

/// The user menu `id`, named `label`, its button showing `name`, around the
/// items the server drew as its children, its sheet's Cancel saying
/// `cancel`. A console draws it through [`crate::widget::menu::user_menu`].
#[island]
pub fn UserMenuButton(
    /// The menu's id, which its button opens.
    id: String,
    /// The menu's name and its sheet's title, in the page's locale.
    label: String,
    /// The name its button shows, or `None` for its icon alone.
    name: Option<String>,
    /// The sheet's Cancel, in the page's locale.
    cancel: String,
    /// The items, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::menu::menu_button(id, label, name, cancel, children().into_any())
}

/// The menu or listbox `id` behind `trigger`, around the items the server
/// drew as its children. A console draws it through
/// [`crate::widget::action_menu::more_menu`], [`crate::widget::action_menu::row_menu`],
/// [`crate::widget::action_menu::variants_menu`] or
/// [`crate::widget::action_menu::listbox`].
#[island]
pub fn MenuButton(
    /// The menu's id, which its trigger controls.
    id: String,
    /// The menu's name and the title of the phone's sheet, in the page's
    /// locale.
    label: String,
    /// The trigger's name, in the page's locale.
    name: String,
    /// What the trigger is and the words it shows.
    trigger: Trigger,
    /// The sheet's Cancel, in the page's locale.
    cancel: String,
    /// The items or options, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::action_menu::menu_button(id, label, name, trigger, cancel, children().into_any())
}

/// The combobox `id`, posting its value as `name`: a native select until
/// the page hydrates, then the filtering listbox
/// ([`crate::widget::combobox`]).
#[island]
pub fn Combobox(
    /// The control's id.
    id: String,
    /// The name the form posts the value as.
    name: String,
    /// The label, in the page's locale.
    label: String,
    /// The words shown while nothing is chosen, in the page's locale.
    placeholder: String,
    /// The choices, in their order.
    options: Vec<ComboOption>,
    /// The value chosen, if any.
    selected: Option<String>,
    /// The id of the element that describes the control, such as its help
    /// line.
    described_by: Option<String>,
    /// The words it draws itself, in the page's locale
    /// ([`ComboboxWords::of_page`]).
    words: ComboboxWords,
) -> impl IntoView {
    crate::widget::combobox::keyed_combobox(KeyedCombobox {
        id,
        name,
        label,
        placeholder,
        options,
        selected: RwSignal::new(selected),
        described_by,
        words,
    })
}

/// The picker field `id`: a choice from what the server holds or a typed
/// value, posted as `name` ([`crate::widget::picker`]).
#[island]
pub fn PickerField(
    /// The text input's id; the select's is this with `-choice`.
    id: String,
    /// The name the input submits as.
    name: String,
    /// The input's label, in the page's locale.
    label: String,
    /// The select's label, in the page's locale.
    choose_label: String,
    /// The help line under both, in the page's locale.
    help: Option<String>,
    /// The choices, value and words.
    choices: Vec<(String, String)>,
    /// The value the input holds as the page arrives.
    value: String,
    /// The empty choice's words, in the page's locale.
    empty_word: String,
    /// The words of the choice that opens the input, in the page's locale.
    other_word: String,
) -> impl IntoView {
    crate::widget::picker::picker_field(crate::widget::picker::Picker {
        id,
        name,
        label: label.into(),
        choose_label: choose_label.into(),
        help: help.map(Into::into),
        choices: Signal::stored(choices),
        value: RwSignal::new(value),
        empty_word: empty_word.into(),
        other_word: other_word.into(),
    })
}

/// The legend of a line chart's plot `plot`: a static key until the page
/// hydrates, then a toggle per series. A console draws it through
/// [`crate::widget::chart::line_chart`].
#[island]
pub fn ChartLegend(
    /// The id of the plot the series are drawn in.
    plot: String,
    /// The legend's name, in the page's locale.
    label: String,
    /// The series, each its name and its palette index.
    entries: Vec<crate::widget::chart::LegendEntry>,
    /// What a break in a line means, in the page's locale, when the chart
    /// has one.
    gap: Option<String>,
) -> impl IntoView {
    crate::widget::chart::legend(plot, label, entries, gap)
}

/// The top bar's search box `search`, its answer under it, the server's
/// children, open when `open`, Enter following the hit `selected`. A
/// console draws it through [`crate::view::search::search_box`].
#[island]
pub fn Search(
    /// What the search box holds, its words in the page's locale.
    search: crate::view::search::SearchBox,
    /// The id of the hit Enter follows, the answer's first.
    selected: Option<String>,
    /// Whether the answer is open.
    open: bool,
    /// The answer, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::search::search_control(search, selected, open, children().into_any())
}

/// The date field `field` with its picker (R23, R25): a popover under the
/// field from 40 rem, a bottom sheet below it. The server draws it closed,
/// and the typed text posts with its form whether or not the WebAssembly
/// runs.
#[island]
pub fn DatePicker(
    /// The field, its bounds, its quick choices and its words
    /// ([`crate::widget::date::DateWords::of_page`]).
    field: crate::widget::date::DateField,
) -> impl IntoView {
    crate::widget::date::date_field(field)
}

/// The date range `range` of a history filter with its picker (R22,
/// R25): its From and To fields until the WebAssembly runs, then one button
/// naming the range.
#[island]
pub fn DateRangePicker(
    /// The range, its ends, its presets and its words.
    range: crate::widget::date::DateRange,
) -> impl IntoView {
    crate::widget::date::date_range(range)
}
