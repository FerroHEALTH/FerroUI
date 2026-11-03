// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page: the fields, the checkbox row, the highlighted editor with
//! diagnostics that link to its lines, and the filter over a list.
//!
//! The page takes one post of its own: the note of the action answered on
//! its page, which it draws again with the count of its words or the
//! refusal, the note kept in its field and every field the form marks
//! secret left out (#485, #499).

mod answer;
mod dates;
mod editors;
mod inputs;
mod pickers;
mod repeat;

use ferrohealth_ui::html::Html;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::form::Posted;
use ferrohealth_ui::view::layout::{Arrangement, Gap, stack_at};
use ferrohealth_ui::view::surface::page_header;

use crate::locale::Msg;
use crate::page::Page;
use crate::visit::Visit;

use dates::dates;
use editors::{
    diagnostics_of_files, diagnostics_real_size, editor, editor_headers, filter, mapping_editor,
    new_mapping, upload,
};
use inputs::{federated, fields, inline, per_row, pick_or_typed, shapes};
use pickers::{choosing, grouped, long_label, nested_reveal};
use repeat::{edge_parts, endpoints};

/// The forms page, drawn again with the answer to the form body `posted`
/// when the page was posted to.
#[must_use]
pub fn page(v: &Visit, posted: Option<&str>) -> Page {
    let posted = posted.map(Posted::read);
    let posted = posted.as_ref();
    let header = page_header(
        &v.locale,
        v.tr(Msg::FormsEditor),
        Some(v.tr(Msg::FieldsGroupedFieldsUploadField)),
        ActionGroup::default(),
    );
    let pair = Arrangement::Grid.wrap(Html::join([
        fields(v),
        stack_at(Gap::Loose, Html::join([grouped(v), federated(v)])),
    ]));
    // Cards of unlike length side by side, each at its own height.
    let unlike = Arrangement::GridStart.wrap(Html::join([long_label(v), nested_reveal(v)]));
    let dated = Arrangement::GridStart.wrap(Html::join([dates(v), edge_parts(v)]));
    let shaped = Arrangement::GridStart.wrap(Html::join([shapes(v), pick_or_typed(v)]));
    let body = stack_at(
        Gap::Loose,
        Html::join([
            pair,
            dated,
            endpoints(v),
            editor_headers(v),
            inline(v),
            per_row(v),
            choosing(v),
            unlike,
            Arrangement::GridStart.wrap(Html::join([
                answer::answered_action(v, posted),
                answer::answered_in_place(v, posted),
            ])),
            shaped,
            upload(v),
            editor(v),
            mapping_editor(v),
            new_mapping(v),
            diagnostics_of_files(v),
            diagnostics_real_size(v),
            filter(v),
            super::records::identifiers_section(v),
            super::dock::long_form(v),
        ]),
    );
    let dock = super::dock::dock(v).unwrap_or_default();
    Page::shell(v.tr(Msg::FormsEditor), Html::join([header, body, dock]))
}
