// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's ready components: the widgets that own their state and
//! need nothing from a console's code, each ready to place in a page.
//!
//! Each one is a `#[component]` function of this module, and this module is
//! the only list of them, so two changes that each add one touch different
//! lines. The module keeps the name it had while the consoles rendered in
//! islands mode (FerroUI #410 moved them to whole-page hydration).
//!
//! Each component here is a thin wrapper over its widget function in
//! [`crate::widget`], so a console that needs the widget beside state of
//! its own calls that function in a component of its own instead. Its words
//! are props, resolved in the page's locale.
#![expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]

use leptos::prelude::*;

use crate::view::tabs::TabChoice;
use crate::widget::action_menu::Trigger;
use crate::widget::combobox::{ComboOption, ComboboxWords, KeyedCombobox};
use crate::widget::toast::{Flash, RegionWords};

/// The toast region, which draws `flash` as the page arrives and every
/// toast [`crate::widget::toast::raise`] shows after. A console draws it
/// once, in its shell.
#[component]
pub fn ToastRegion(
    /// The words it draws, in the page's locale
    /// ([`RegionWords::of_page`]).
    words: RegionWords,
    /// The toasts the server shows as the page arrives.
    flash: Vec<Flash>,
) -> impl IntoView {
    crate::widget::toast::toast_region(words, flash)
}

/// The polite region that says the title of each page the router draws in
/// the browser, and moves the focus to its heading. A console draws it
/// once, at its root, beside the toast region.
#[component]
pub fn PageAnnouncer() -> impl IntoView {
    crate::widget::announce::page_announcer()
}

/// The keyboard shortcut legend, opened by `?` outside a field and by the
/// user menu's Keyboard shortcuts ([`crate::widget::shortcuts`]). A console
/// draws it once on a page that has shortcuts.
#[component]
pub fn ShortcutLegend(
    /// The page's shortcuts, their words in the page's locale.
    groups: Vec<crate::view::kbd::ShortcutGroup>,
    /// The legend's own words, in the page's locale.
    words: crate::widget::shortcuts::LegendWords,
) -> impl IntoView {
    crate::widget::shortcuts::legend_on_key(groups, words)
}

/// Follows the binder's link tabs on the digit keys
/// ([`crate::widget::shortcuts::tab_keys`]); the binder draws it.
#[component]
pub fn TabKeys() -> impl IntoView {
    crate::widget::shortcuts::tab_keys()
}

/// Takes the page's form submits through the router, so a form loads no
/// new document ([`crate::widget::submit`]). A console draws it once,
/// inside its `<Router>`.
#[component]
pub fn FormNavigator() -> impl IntoView {
    crate::widget::submit::form_navigator()
}

/// The keyboard tabs `prefix`, named `label`, one tab per word in `tabs`,
/// over the panels [`crate::widget::tabs::tab_panels`] drew as its
/// children.
#[component]
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
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a ready component takes its props by value"
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
#[component]
pub fn KeyHint(
    /// The configuration key it names and copies.
    key: String,
    /// Its words, in the page's locale ([`crate::widget::copy::KeyHintWords::of_page`]).
    words: crate::widget::copy::KeyHintWords,
) -> impl IntoView {
    crate::widget::copy::key_hint(key, words)
}

/// The commit-confirm countdown `countdown` (F2): the ring and the time
/// left, counting down by the second once the page hydrates, with a polite
/// live region at its set points. The server draws the time left, so with
/// no WebAssembly the page shows it as it was drawn. A console draws it
/// through [`crate::view::lifecycle::trial_actions`].
#[component]
pub fn CommitCountdown(
    /// The countdown, its time left and its words
    /// ([`crate::widget::countdown::Countdown::new`]).
    countdown: crate::widget::countdown::Countdown,
) -> impl IntoView {
    crate::widget::countdown::countdown(countdown)
}

/// The copy button `id` saying `label`, putting what `source` names on the
/// clipboard when pressed ([`crate::widget::copy`]). The server draws it
/// hidden, so with no WebAssembly the text stays on the page to select.
#[component]
pub fn CopyButton(
    /// The button's id.
    id: String,
    /// Its words, in the page's locale, such as "Copy".
    #[prop(into)]
    label: crate::i18n::Word,
    /// What it copies, which the page already shows.
    source: crate::widget::copy::CopySource,
    /// The words it draws itself, in the page's locale
    /// ([`crate::widget::copy::CopyWords::of_page`]).
    words: crate::widget::copy::CopyWords,
) -> impl IntoView {
    crate::widget::copy::copy_from(id, label, source, words)
}

/// The hold-to-show button of the masked input `input` (R10): held, it
/// shows what was typed, and released, it masks it again. It names the
/// input by its id alone, so the typed value is never a prop; until the
/// page's WebAssembly runs it is disabled and says the field stays masked.
/// A console draws it through [`crate::view::form::Inner::HoldToShow`].
#[component]
pub fn HoldToShow(
    /// The id of the masked input it shows.
    input: String,
    /// Its words, in the page's locale
    /// ([`crate::widget::masked::HoldWords::of_page`]).
    words: crate::widget::masked::HoldWords,
) -> impl IntoView {
    crate::widget::masked::hold_to_show(input, words)
}

/// The side panel `id` headed `title`, around the body the server drew as
/// its children, and its `opener`: a link to the content's own route until
/// the page hydrates, then the button that opens the panel
/// ([`crate::widget::side_panel`]).
#[component]
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

/// The work-list around the data table the server drew as its children,
/// its rows from [`crate::widget::worklist::row`]: the keys move the
/// focused row and Enter follows its row link ([`crate::widget::worklist`]).
#[component]
pub fn WorkList(
    /// The data table and the rows, drawn on the server.
    children: Children,
) -> impl IntoView {
    let focused = RwSignal::new(None);
    let on_open = Callback::new(|key: String| crate::widget::worklist::follow(&key));
    crate::widget::worklist::worklist(
        crate::widget::worklist::WorkList { focused, on_open },
        children().into_any(),
    )
}

/// The topology (F9) around the toolbar, the map and the List the server
/// drew as its children, with the class `class`: zoom by the cluster and
/// the keys, panning by the pointer, the quick look on "i" or Space and the
/// arrows along the List ([`crate::widget::topology`]). A console draws it
/// through [`crate::view::topology::topology`].
#[component]
pub fn TopologyKeys(
    /// The topology's classes, its view among them.
    class: String,
    /// The readout's words, in the page's locale.
    words: crate::widget::topology::TopologyWords,
    /// The toolbar, the map and the List, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::topology::topology_keys(class, words, children().into_any())
}

/// The phone's select over binder tabs, named `label`, on the choice that
/// leads to `chosen`, with the submit button saying `go`. A console draws it through
/// [`crate::view::tabs::binder`], which places it past
/// [`crate::view::tabs::MANY`] tabs.
#[component]
#[expect(
    clippy::needless_pass_by_value,
    reason = "a ready component takes its props by value"
)]
pub fn TabSelect(
    /// The tabs' name, in the page's locale.
    label: String,
    /// The submit button's words, in the page's locale.
    #[prop(into)]
    go: crate::i18n::Word,
    /// Each tab's route and words, in the page's locale.
    choices: Vec<TabChoice>,
    /// The route of the current tab.
    chosen: String,
) -> impl IntoView {
    crate::widget::tabs::tab_select(label, go, choices, &chosen)
}

/// The navigation drawer `id`, a modal dialog named `label`, around the
/// entries the server drew as its children, its close button saying
/// `close`, folded to its glyphs while `rail`. A console draws it through
/// [`crate::widget::nav::sidebar`].
#[component]
pub fn NavDrawer(
    /// The drawer's id, which the drawer button opens.
    id: String,
    /// The navigation's name, in the page's locale.
    label: String,
    /// The close button's words, in the page's locale.
    #[prop(into)]
    close: crate::i18n::Word,
    /// Whether the navigation is folded to its rail of glyphs.
    rail: bool,
    /// The groups of entries, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::nav::drawer(id, label, close, rail, children().into_any())
}

/// The user menu `id`, its sheet titled `label`, its button showing `name`,
/// around the head and the ARIA menu the server drew as its children, its
/// sheet's Cancel saying `cancel`. A console draws it through
/// [`crate::widget::menu::user_menu`].
#[component]
pub fn UserMenuButton(
    /// The popup's id, which its button opens; its ARIA menu's is this and
    /// `-menu`.
    id: String,
    /// The sheet's title, and the button's name when it shows no `name`,
    /// in the page's locale.
    label: String,
    /// The name its button shows, or `None` for its icon alone.
    name: Option<String>,
    /// The sheet's Cancel, in the page's locale.
    #[prop(into)]
    cancel: crate::i18n::Word,
    /// The head and the menu, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::menu::menu_button(id, label, name, cancel, children().into_any())
}

/// The menu or listbox `id` behind `trigger`, around the items the server
/// drew as its children. A console draws it through
/// [`crate::widget::action_menu::more_menu`], [`crate::widget::action_menu::row_menu`],
/// [`crate::widget::action_menu::variants_menu`],
/// [`crate::widget::action_menu::labelled_menu`] or
/// [`crate::widget::action_menu::listbox`].
#[component]
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
    #[prop(into)]
    cancel: crate::i18n::Word,
    /// What a copy item says came of it, in the page's locale; `None` for
    /// a menu with no copy item.
    copy: Option<crate::widget::copy::CopyWords>,
    /// The items or options, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::action_menu::menu_button(
        id,
        label,
        name,
        trigger,
        cancel,
        copy,
        children().into_any(),
    )
}

/// The combobox `id`, posting its value as `name`: a native select until
/// the page hydrates, then the filtering listbox
/// ([`crate::widget::combobox`]).
#[component]
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
#[component]
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
#[component]
pub fn ChartLegend(
    /// The id of the plot the series are drawn in.
    plot: String,
    /// The legend's name, in the page's locale.
    label: String,
    /// The series, each its name and its palette index.
    entries: Vec<crate::widget::chart::LegendEntry>,
    /// What a break in a line means, in the page's locale, when the chart
    /// has one.
    gap: Option<crate::i18n::Word>,
) -> impl IntoView {
    crate::widget::chart::legend(plot, label, entries, gap)
}

/// The top bar's search box `search`, which opens its typeahead panel
/// under the field, drawn in `words`. A console draws it through
/// [`crate::view::search::search_box`].
#[component]
pub fn Search(
    /// What the search box holds, its words in the page's locale.
    search: crate::view::search::SearchBox,
    /// The words the panel draws, in the page's locale.
    words: crate::view::search::SearchWords,
) -> impl IntoView {
    crate::widget::search::search_control(search, words)
}

/// The repeatable rows `id`, named `label` (F7), around the rows, the
/// blank row and Add the server drew as its children: Remove and Add post
/// the console's form until the page hydrates, then work in place. A
/// console draws it through [`crate::widget::repeat::repeat`].
#[component]
pub fn Repeater(
    /// The list's id.
    id: String,
    /// The list's name, in the page's locale.
    label: String,
    /// The rows, the blank row and Add, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::repeat::repeater(id, label, children().into_any())
}

/// The date field `field` with its picker (R23, R25): a popover under the
/// field from 40 rem, a bottom sheet below it. The server draws it closed,
/// and the typed text posts with its form whether or not the WebAssembly
/// runs. With the `split` feature it is lazy, its code loading from its own
/// chunks under `cargo leptos build --split`.
#[component]
pub fn DatePicker(
    /// The field, its bounds, its quick choices and its words
    /// ([`crate::widget::date::DateWords::of_page`]).
    field: crate::widget::date::DateField,
) -> impl IntoView {
    crate::widget::date::date_field(field)
}

/// The date range `range` of a history filter with its picker (R22,
/// R25): its From and To fields until the WebAssembly runs, then one button
/// naming the range. With the `split` feature it is lazy, its code loading
/// from its own chunks under `cargo leptos build --split`.
#[component]
pub fn DateRangePicker(
    /// The range, its ends, its presets and its words.
    range: crate::widget::date::DateRange,
) -> impl IntoView {
    crate::widget::date::date_range(range)
}

/// The tree `tree` (M1, E4, E5): its items fold in place and the keyboard
/// walks them; an unread level and every fold with no WebAssembly follow
/// the toggle's link, which the console's server answers. With the `split`
/// feature it is lazy, its code loading from its own chunks under
/// `cargo leptos build --split`.
#[component]
pub fn TreeView(
    /// The tree, its items and its open and selected items as the server
    /// drew them.
    tree: crate::widget::tree::TreeData,
    /// Its words ([`crate::widget::tree::TreeWords::of_page`]).
    words: crate::widget::tree::TreeWords,
) -> impl IntoView {
    crate::widget::tree::tree_island(tree, words)
}

/// The form of one preference in Your preferences, posting its control
/// (the server children) and `back` to `action`: a Save button saying
/// `save` until the page hydrates, then a post on every change, the focus
/// back on the control of the row just `saved`. A console draws it through
/// [`crate::view::preferences::select_preference`] and the page
/// [`crate::view::preferences::preferences_page`].
#[component]
pub fn PreferenceForm(
    /// The console's route that keeps the preference in its cookie.
    action: String,
    /// The page to return to.
    back: String,
    /// The Save button's words, in the page's locale.
    #[prop(into)]
    save: crate::i18n::Word,
    /// Whether this preference was just saved.
    saved: bool,
    /// The control, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::widget::preference::preference_form(action, back, save, saved, children().into_any())
}

/// The watch over the console's session ([`crate::widget::session`]): it
/// leaves a protected page for sign-in the moment the session ends, warns
/// before an idle timeout with Stay signed in, and ends every tab of the
/// console together. A console draws it once, in the shell of a signed-in
/// page.
#[component]
pub fn SessionWatch(
    /// The session route, the sign-in and sign-out addresses, what the
    /// session had left as the server drew the page, and the words
    /// ([`crate::widget::session::Watch::new`]).
    watch: crate::widget::session::Watch,
) -> impl IntoView {
    crate::widget::session::session_watch(watch)
}

/// Tells every other tab of the console that the session has ended, so each
/// leaves for sign-in ([`crate::widget::session::gone`]). A console draws it
/// on its signed-out page; it draws nothing.
#[component]
pub fn SessionEnded() -> impl IntoView {
    Effect::new(move |_| crate::widget::session::gone());
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_library_component_is_an_island() {
        // NOTE: tachys 0.2 `src/html/islands.rs`: a client render builds no island children,
        // so an island would draw nothing after a client navigation.
        let source = include_str!("island.rs");
        let islands = source
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with("#[") && line.contains("island"))
            .count();
        assert_eq!(0, islands, "every ready component is a #[component]");
    }
}
