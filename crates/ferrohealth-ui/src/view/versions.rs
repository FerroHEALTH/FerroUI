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

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, Word, tr, tr_with};
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
    pub when: Html,
    /// Its contribution, a link the console draws.
    pub contribution: Html,
    /// Whether this is the version the page reads, marked `aria-current`.
    pub read: bool,
}

/// The At a time row under a version history.
#[derive(Debug)]
pub struct AtTime {
    /// The console's route the form opens with a `GET`, its field's value
    /// in the query.
    pub action: String,
    /// The date and time field: [`crate::view::date::date_field`] over a
    /// [`crate::view::date::DateField`] with its time, labelled
    /// [`at_time_label`].
    pub field: Html,
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
pub fn at_time_label(locale: &Locale) -> Word {
    tr(locale, Text::VersionsAtTime)
}

/// Returns a version's name as its table shows it: "Version 3" with the
/// Current mark beside it, kept on one line, or the Deleted state after it.
#[must_use]
pub fn version_cell(locale: &Locale, number: &str, standing: Standing) -> Html {
    let name = tr_with(locale, Text::VersionsCell, &[("value", Arg::from(number))]);
    let mark = match standing {
        Standing::Current => format!(
            "<span class=\"{}\">{}</span>",
            Tone::Accent.badge(),
            tr(locale, Text::VersionsCurrent)
        ),
        Standing::Older => String::new(),
        Standing::Deleted => {
            String::from(state(Mark::Off, tr(locale, Text::VersionsDeleted), None))
        }
    };
    Html::raw(format!(
        "<span class=\"fui-version-cell\">{name}{mark}</span>"
    ))
}

/// The columns of a version history, built where it is drawn so their
/// headers follow the page's locale.
fn columns(locale: &Locale) -> [Column; 5] {
    let column = |key: &'static str, label: Text, role: Role| Column {
        role,
        ..Column::text(key, tr(locale, label))
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
fn row(locale: &Locale, version: Version, columns: &[Column; 5]) -> String {
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
    let name = title_cell(
        href.as_deref(),
        version_cell(locale, &number, standing),
        None,
    );
    format!(
        "<tr{}>{name}{}{}{}{}</tr>",
        if read { " aria-current=\"true\"" } else { "" },
        cell(change_column, Html::text(change)),
        cell(committer_column, Html::text(committer)),
        cell(when_column, when),
        cell(contribution_column, contribution)
    )
}

/// Returns the At a time row `at`: its field and Open on one row, then its
/// quiet line.
///
/// It is a plain `GET` form, so Open works with no script; a time is no
/// patient identifier.
#[must_use]
pub fn at_time(locale: &Locale, at: AtTime) -> Html {
    let AtTime {
        action,
        field,
        opens,
    } = at;
    Html::raw(format!(
        "<form method=\"get\" action=\"{}\" class=\"fui-at-time\">{field}<button type=\"submit\" class=\"{}\">{}</button><span class=\"fui-small fui-muted\">{}</span></form>",
        Html::text(action),
        Variant::Secondary.class(),
        tr(locale, Text::VersionsOpen),
        Html::text(opens)
    ))
}

/// Returns the version history `history`: the table, then At a time under
/// it.
///
/// A history with no version shows one quiet line in its table. At a time
/// sits under the table with a hairline and a gap. Put it in a
/// [`crate::view::surface::panel_section`], where the table runs edge to
/// edge.
#[must_use]
pub fn version_history(locale: &Locale, history: VersionHistory) -> Html {
    let VersionHistory {
        caption,
        versions,
        at_time: at,
    } = history;
    let columns = columns(locale);
    let body = if versions.is_empty() {
        Body::Empty(Html::raw(format!(
            "<span class=\"fui-muted\">{}</span>",
            tr(locale, Text::VersionsNone)
        )))
    } else {
        Body::Rows(Html::raw(
            versions
                .into_iter()
                .map(|version| row(locale, version, &columns))
                .collect::<Vec<_>>()
                .concat(),
        ))
    };
    let table = data_table(DataTable {
        caption: &caption,
        columns: &columns,
        sorting: crate::table::view::Sorting::default(),
        body,
    });
    table.then(at.map(|at| at_time(locale, at)).unwrap_or_default())
}

/// Returns the notice over an older version: "You are reading version
/// `version` of `count`." and the link to the latest at `latest`.
#[must_use]
pub fn older_version(
    locale: &Locale,
    version: &str,
    count: u32,
    latest: impl Into<String>,
) -> Html {
    let reading = tr_with(
        locale,
        Text::VersionsReading,
        &[("value", Arg::from(version)), ("total", Arg::from(count))],
    );
    notice(
        Tone::Info,
        Announce::Status,
        None,
        Html::raw(format!(
            "{reading} <a href=\"{}\">{}</a>",
            Html::text(latest.into()),
            tr(locale, Text::VersionsLatest)
        )),
    )
}
