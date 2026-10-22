// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's islands: each interactive widget a page shows, with the
//! state it shares, as the one part of the page the browser hydrates.
//!
//! The pages draw everything else on the server alone. Each island is a
//! thin wrapper over the builder its page defines, so the page reads as
//! one piece and the island only marks what the browser hydrates (the
//! Leptos book, "Guide: Islands").
#![expect(
    clippy::mem_forget,
    reason = "the island macro of leptos_macro 0.8 keeps each hydrated island alive by forgetting its view state, in the browser entry it writes"
)]
#![expect(
    clippy::must_use_candidate,
    reason = "the #[island] macro drops a #[must_use] from the function it writes"
)]
#![expect(
    missing_docs,
    reason = "the #[island] macro writes an undocumented struct of each island's props as they cross to the browser"
)]

use ferrohealth_ui::view::help::HelpWords;
use ferrohealth_ui::widget::combobox::ComboboxWords;
use ferrohealth_ui::widget::dialog::ModalWords;
use ferrohealth_ui::widget::side_panel::SidePanelWords;
use ferrohealth_ui::widget::upload::UploadWords;
use leptos::prelude::*;

use crate::locale::Words;

/// The confirm button of a confirmation dialog, which raises a toast.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn ConfirmAndToast(
    /// The id of the dialog it confirms.
    dialog: String,
    /// Whether it is the danger dialog's.
    danger: bool,
    /// Its words, in the page's locale.
    words: Words,
) -> impl IntoView {
    crate::pages::overlays::confirm_and_toast(dialog, danger, &words)
}

/// The overlays page's modal dialogs and the rows that open them.
#[island]
pub fn ModalDialogs(
    /// Their words, in the page's locale.
    words: Words,
    /// The dialogs' own words, in the page's locale.
    modal: ModalWords,
) -> impl IntoView {
    crate::pages::overlays::modal_dialogs(words, modal)
}

/// The overlays page's side panel, around the body the server drew.
#[island]
pub fn AccessPanel(
    /// The panel's title, in the page's locale.
    title: String,
    /// The panel's own words, in the page's locale.
    words: SidePanelWords,
    /// The body, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::pages::overlays::access_panel(title, words, children().into_any())
}

/// The overlays page's buttons that raise a toast in each tone.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn ToastButtons(
    /// Their words, in the page's locale.
    words: Words,
    /// The tables page, where the success toast leads.
    tables: String,
) -> impl IntoView {
    crate::pages::overlays::toast_buttons(&words, tables)
}

/// The overview's segmented control bound to state, in a cluster with the
/// controls the server drew as its children.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn FormatSegments(
    /// Its words, in the page's locale.
    words: Words,
    /// The segmented links and the posted segments.
    children: Children,
) -> impl IntoView {
    crate::pages::overview::format_segments(&words, children().into_any())
}

/// The layouts page's concept tree, which keeps its open items in the
/// address.
#[island]
pub fn ConceptTree(
    /// The key of the brand whose page it is on.
    brand: String,
    /// The concept the address selects.
    concept: Option<String>,
    /// The open items the address names.
    open: Option<String>,
    /// Its words, in the page's locale.
    words: Words,
) -> impl IntoView {
    crate::pages::layouts::concept_tree(brand, concept, open, words)
}

/// The layouts page's runner form: its help switch and the fields it
/// explains.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn RunnerForm(
    /// The key of the brand whose page it is on.
    brand: String,
    /// Its words, in the page's locale.
    words: Words,
    /// The help switch's words, in the page's locale.
    help: HelpWords,
) -> impl IntoView {
    crate::pages::layouts::runner_form_in(&brand, &words, help)
}

/// The forms page's pick-or-type fields.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn PickOrTyped(
    /// Its words, in the page's locale.
    words: Words,
    /// The combobox's own words, in the page's locale.
    combo: ComboboxWords,
) -> impl IntoView {
    crate::pages::forms::pick_or_typed_body(&words, combo)
}

/// The forms page's upload field.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn UploadField(
    /// Its words, in the page's locale.
    words: Words,
    /// The field's own words, in the page's locale.
    upload: UploadWords,
) -> impl IntoView {
    crate::pages::forms::upload_body(&words, upload)
}

/// The forms page's highlighted editor and the size it reports.
#[island]
pub fn DraftEditor(
    /// Its words, in the page's locale.
    words: Words,
) -> impl IntoView {
    crate::pages::forms::editor_body(words)
}

/// The forms page's editor inside a reloading transition.
#[island]
pub fn ReloadingEditor(
    /// Its words, in the page's locale.
    words: Words,
) -> impl IntoView {
    crate::pages::forms::reloading_editor_body(words)
}

/// The forms page's filter and the list it filters.
#[island]
#[expect(
    clippy::needless_pass_by_value,
    reason = "an island takes its props by value, as they arrive from the page"
)]
pub fn FileFilter(
    /// Its words, in the page's locale.
    words: Words,
) -> impl IntoView {
    crate::pages::forms::filter_body(&words)
}

/// The overlays page's keyboard shortcut legend and the button and key
/// that open it.
#[island]
pub fn ShortcutLegend(
    /// The page's shortcuts, their words in the page's locale.
    groups: Vec<ferrohealth_ui::view::kbd::ShortcutGroup>,
    /// The legend's own words, in the page's locale.
    words: ferrohealth_ui::widget::shortcuts::LegendWords,
    /// The words of the button that opens it.
    opener: String,
) -> impl IntoView {
    crate::pages::overlays::shortcut_legend_with(groups, words, opener)
}
