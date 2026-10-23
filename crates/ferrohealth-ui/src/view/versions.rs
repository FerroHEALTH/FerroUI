// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The version history and At a time (E2).
//!
//! A versioned object's revisions are a table in a B10 section: the
//! version with its Current mark kept together, or Deleted as an F12 state;
//! the change type, who committed it, a relative time (family rule 11) and
//! the contribution as a link. The version being read is marked in its
//! table (`aria-current`). Under the table, At a time is one row of a date
//! and time field and Open, a plain `GET` form, with one quiet line on what
//! it opens. An older version opened elsewhere says so in one info notice
//! ("You are reading version 2 of 3. Read the latest"), and the title row
//! then leaves out the version. The revisions are the openEHR
//! `VERSIONED_OBJECT` and its `VERSION`s, each with its change type and
//! contribution (openEHR RM, Common Information Model, § Change Control).
//! No specification governs the look: our own design.

use leptos::prelude::*;

use crate::i18n::{Arg, Text, Word, tr, tr_with};
use crate::table::view::{Body, Column, DataTable, Role, cell, data_table, title_cell};
use crate::tone::Tone;
use crate::view::button::Variant;
use crate::view::notice::{Announce, notice};
use crate::view::state::{Mark, state};

/// Where a version stands in its object's history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Standing {
    /// The latest version, marked Current.
    Current,
    /// An earlier version.
    Older,
    /// The version that deleted the object, drawn as the Deleted state.
    Deleted,
}

/// One row of a version history.
#[derive(Debug)]
pub struct Version {
    /// The version's number or name, such as "3" or "1.1.0".
    pub number: String,
    /// Where it stands.
    pub standing: Standing,
    /// The page that reads it, which its name links to; `None` for a
    /// version with nothing to read, such as a deletion.
    pub href: Option<String>,
    /// Its change type in words, such as "Modification".
    pub change: String,
    /// Who committed it.
    pub committer: String,
    /// When it was committed, relative with the full timestamp in its
    /// tooltip, as the console draws it (family rule 11).
    pub when: AnyView,
    /// Its contribution, a link the console draws.
    pub contribution: AnyView,
    /// Whether this is the version the page reads, marked `aria-current`.
    pub read: bool,
}

/// The At a time row under a version history.
#[derive(Debug)]
pub struct AtTime {
    /// The console's route the form opens with a `GET`, its field's value
    /// in the query.
    pub action: String,
    /// The date and time field: [`crate::island::DatePicker`] over a
    /// [`crate::widget::date::DateField`] with its time, labelled
    /// [`at_time_label`].
    pub field: AnyView,
    /// The one quiet line on what it opens, such as "Opens the version
    /// current then."
    pub opens: String,
}

/// What a version history shows.
#[derive(Debug)]
pub struct VersionHistory {
    /// The table's caption, which names it.
    pub caption: String,
    /// The versions, the latest first.
    pub versions: Vec<Version>,
    /// The At a time row under the table, when the object can be read at a
    /// time.
    pub at_time: Option<AtTime>,
}

/// The label of the At a time field, for the [`AtTime::field`] a console
/// builds.
#[must_use]
pub fn at_time_label() -> Word {
    tr(Text::VersionsAtTime)
}

/// Returns a version's name as its table shows it: "Version 3" with the
/// Current mark beside it, kept on one line, or the Deleted state after it.
#[must_use]
pub fn version_cell(number: &str, standing: Standing) -> AnyView {
    let name = tr_with(Text::VersionsCell, &[("value", Arg::from(number))]);
    let mark = match standing {
        Standing::Current => Some(
            view! { <span class=Tone::Accent.badge()>{tr(Text::VersionsCurrent)}</span> }
                .into_any(),
        ),
        Standing::Older => None,
        Standing::Deleted => Some(state(Mark::Off, tr(Text::VersionsDeleted), None)),
    };
    view! { <span class="fui-version-cell">{name}{mark}</span> }.into_any()
}

/// The columns of a version history, built where it is drawn so their
/// headers follow the page's locale.
fn columns() -> [Column; 5] {
    let column = |key: &'static str, label: Text, role: Role| Column {
        role,
        ..Column::text(key, tr(label))
    };
    [
        column("version", Text::VersionsVersion, Role::Title),
        column("change", Text::VersionsChange, Role::Sub),
        column("committer", Text::VersionsCommittedBy, Role::Quiet),
        column("when", Text::VersionsWhen, Role::Meta),
        column("contribution", Text::VersionsContribution, Role::Wide),
    ]
}

/// Returns one row of the table.
fn row(version: Version, columns: &[Column; 5]) -> AnyView {
    let Version {
        number,
        standing,
        href,
        change,
        committer,
        when,
        contribution,
        read,
    } = version;
    let [
        _,
        change_column,
        committer_column,
        when_column,
        contribution_column,
    ] = columns;
    let name = title_cell(href.as_deref(), version_cell(&number, standing), None);
    let current = read.then_some("true");
    view! {
        <tr aria-current=current>
            {name}
            {cell(change_column, change)}
            {cell(committer_column, committer)}
            {cell(when_column, when)}
            {cell(contribution_column, contribution)}
        </tr>
    }
    .into_any()
}

/// Returns the At a time row `at`: its field and Open on one row, then its
/// quiet line.
///
/// It is a plain `GET` form, so Open works with no WebAssembly; a time is
/// no patient identifier (`leptos-ui.md` §0).
#[must_use]
pub fn at_time(at: AtTime) -> AnyView {
    let AtTime {
        action,
        field,
        opens,
    } = at;
    view! {
        <form class="fui-at-time" method="get" action=action>
            {field}
            <button type="submit" class=Variant::Secondary.class()>
                {tr(Text::VersionsOpen)}
            </button>
            <span class="fui-small fui-muted">{opens}</span>
        </form>
    }
    .into_any()
}

/// Returns the version history `history`: the table, then At a time under
/// it.
///
/// A history with no version shows one quiet line in its table. At a time
/// sits under the table with a hairline and a gap. Put it in a
/// [`crate::view::surface::panel_section`], where the table runs edge to
/// edge.
#[must_use]
pub fn version_history(history: VersionHistory) -> AnyView {
    let VersionHistory {
        caption,
        versions,
        at_time: at,
    } = history;
    let columns = columns();
    let body = if versions.is_empty() {
        Body::Empty(view! { <span class="fui-muted">{tr(Text::VersionsNone)}</span> }.into_any())
    } else {
        Body::Rows(
            versions
                .into_iter()
                .map(|version| row(version, &columns))
                .collect_view()
                .into_any(),
        )
    };
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: crate::table::view::Sorting::default(),
        body,
    });
    let at = at.map(at_time);
    view! {
        {table}
        {at}
    }
    .into_any()
}

/// Returns the notice over an older version: "You are reading version
/// `version` of `count`." and the link to the latest at `latest`.
#[must_use]
pub fn older_version(version: &str, count: u32, latest: impl Into<String>) -> AnyView {
    let reading = tr_with(
        Text::VersionsReading,
        &[("value", Arg::from(version)), ("total", Arg::from(count))],
    );
    let latest = latest.into();
    notice(
        Tone::Info,
        Announce::Status,
        None,
        view! {
            {reading}
            " "
            <a href=latest>{tr(Text::VersionsLatest)}</a>
        },
    )
    .into_any()
}
