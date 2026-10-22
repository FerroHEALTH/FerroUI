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

use leptos::prelude::*;

/// The charts page's line chart and its legend.
#[island]
pub fn RequestsChart() -> impl IntoView {
    crate::pages::charts::requests_chart()
}

/// The confirm button of a confirmation dialog, which raises a toast.
#[island]
pub fn ConfirmAndToast(
    /// The id of the dialog it confirms.
    dialog: String,
    /// Whether it is the danger dialog's.
    danger: bool,
) -> impl IntoView {
    crate::pages::overlays::confirm_and_toast(dialog, danger)
}

/// The overlays page's modal dialogs and the rows that open them.
#[island]
pub fn ModalDialogs() -> impl IntoView {
    crate::pages::overlays::modal_dialogs()
}

/// The overlays page's side panel, around the body the server drew.
#[island]
pub fn AccessPanel(
    /// The body, drawn on the server.
    children: Children,
) -> impl IntoView {
    crate::pages::overlays::access_panel(children().into_any())
}

/// The overlays page's buttons that raise a toast in each tone.
#[island]
pub fn ToastButtons() -> impl IntoView {
    crate::pages::overlays::toast_buttons()
}

/// The overview's segmented control bound to state, in a cluster with the
/// controls the server drew as its children.
#[island]
pub fn FormatSegments(
    /// The segmented links and the posted segments.
    children: Children,
) -> impl IntoView {
    crate::pages::overview::format_segments(children().into_any())
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
) -> impl IntoView {
    crate::pages::layouts::concept_tree(brand, concept, open)
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
) -> impl IntoView {
    crate::pages::layouts::runner_form_in(&brand)
}

/// The forms page's pick-or-type fields.
#[island]
pub fn PickOrTyped() -> impl IntoView {
    crate::pages::forms::pick_or_typed_body()
}

/// The forms page's upload field.
#[island]
pub fn UploadField() -> impl IntoView {
    crate::pages::forms::upload_body()
}

/// The forms page's highlighted editor and the size it reports.
#[island]
pub fn DraftEditor() -> impl IntoView {
    crate::pages::forms::editor_body()
}

/// The forms page's editor inside a reloading transition.
#[island]
pub fn ReloadingEditor() -> impl IntoView {
    crate::pages::forms::reloading_editor_body()
}

/// The forms page's filter and the list it filters.
#[island]
pub fn FileFilter() -> impl IntoView {
    crate::pages::forms::filter_body()
}

/// The forms page's code system combobox.
#[island]
pub fn SystemPicker() -> impl IntoView {
    crate::pages::forms::system_picker()
}

/// The forms page's combobox of one clause.
#[island]
pub fn ClausePicker(
    /// The clause's place, from 1.
    place: u8,
) -> impl IntoView {
    crate::pages::forms::clause_picker(place)
}
