// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The patient-record components, drawn on three pages.
//!
//! The record compare and the work-list are on the tables page, the
//! timeline on the layouts page, and the masked identifiers with their
//! reveal on the forms page. Their
//! people, records and identifiers are the synthetic data of
//! [`crate::records_data`], identifiers of several countries side by side.

use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Direction, Sort};
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, RowCheck, Sorting, cell, check_cell, data_table, menu_cell,
    title_cell,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::compare::{
    Compare, CompareField, CompareRecord, Verdict, compare, compare_stacked,
};
use ferrohealth_ui::view::kbd::{Shortcut, shortcut_hint};
use ferrohealth_ui::view::status::{badge, banner};
use ferrohealth_ui::view::surface::{card, titled_panel};
use ferrohealth_ui::view::template::list::{HeadingCount, ListCard, list_card};
use ferrohealth_ui::view::timeline::{Event, timeline};
use ferrohealth_ui::widget::action_menu::{self, Act, Action, Item, Menu};
use ferrohealth_ui::widget::identifier::{
    Domain, Identifier, IdentifierWords, KEEP, Masked, Reveal, Revealed, identifier,
    identifier_list, identifier_static,
};
use ferrohealth_ui::widget::worklist::{WorkRow, row, worklist_foot};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use serde::{Deserialize, Serialize};

use crate::locale::{Msg, tr, tr_with};
use crate::records_data::{
    FAMILY_CHANGE, IDENTIFIERS, LEADING, OTHERS, POSTAL_ADDED, SourceRecord, Synthetic, pair_names,
    pair_wait,
};

/// The seconds a revealed identifier shows.
const REVEAL_SECONDS: u32 = 30;

/// The synthetic identifier `key`.
fn synthetic(key: &str) -> Option<&'static Synthetic> {
    IDENTIFIERS.iter().find(|each| each.key == key)
}

/// The masked form of the synthetic identifier `key`, as a console's
/// server draws it.
fn masked(key: &str) -> Option<Masked> {
    synthetic(key).map(|each| {
        Masked::new(
            Domain::new(each.domain).system(each.system),
            each.value,
            KEEP,
        )
    })
}

/// The masked forms of `keys`, in their order.
fn masked_all(keys: &[&str]) -> Vec<Masked> {
    keys.iter().filter_map(|key| masked(key)).collect()
}

// ---- The record compare, on the tables page ----

/// A record of the compare: its values per field, the last its identifiers.
fn record(source: &SourceRecord) -> CompareRecord {
    let ids = masked_all(source.ids);
    let mut cells: Vec<Option<ViewFn>> = source
        .values
        .into_iter()
        .map(|value| value.map(|value| ViewFn::from(move || value.into_any())))
        .collect();
    cells.push((!ids.is_empty()).then(|| ViewFn::from(move || identifier_list(&ids))));
    CompareRecord {
        key: source.key.to_owned(),
        title: source.title.to_owned(),
        sub: Some(source.sub.to_owned()),
        values: cells,
    }
}

/// The fields the records are compared on.
fn fields() -> Vec<CompareField> {
    [
        ("family", Msg::RecordsFamilyName),
        ("given", Msg::RecordsGivenNames),
        ("born", Msg::RecordsBirthDate),
        ("postal", Msg::RecordsPostalCode),
        ("ids", Msg::RecordsIdentifiers),
    ]
    .into_iter()
    .map(|(key, msg)| CompareField {
        key: key.to_owned(),
        label: tr(msg),
    })
    .collect()
}

/// The verdicts of the compare: a field (by key) of the other record at an
/// index, against the leading record.
fn verdict(field: &str, other: usize) -> Verdict {
    match (field, other) {
        ("family", 1) | ("postal", 0) => Verdict::Differ,
        ("postal" | "given", 2) => Verdict::Absent,
        ("ids", _) => Verdict::Unmarked,
        _ => Verdict::Agree,
    }
}

/// The record compare: a leading record and three others, the differing
/// rows first, its stacked form for a phone, and two records that agree.
#[must_use]
pub fn compare_section() -> AnyView {
    let fields = fields();
    let leading = record(&LEADING);
    let others = OTHERS.each_ref().map(record);
    let caption = tr(Msg::RecordsCompareCaption);
    let table = Compare {
        id: "records-compare",
        caption: &caption,
        fields: &fields,
        leading: &leading,
        others: &others,
        verdict: &verdict,
        differ_first: true,
    };
    let wide = compare(table);
    let stacked = compare_stacked(
        Compare {
            id: "records-compare-stacked",
            ..table
        },
        0,
    );
    let agreeing = &others[..1];
    let agree_caption = tr(Msg::RecordsCompareCaptionAgree);
    let agree = |field: &str, _: usize| {
        if field == "ids" || field == "postal" {
            Verdict::Unmarked
        } else {
            Verdict::Agree
        }
    };
    let all_agree = compare(Compare {
        id: "records-compare-agree",
        caption: &agree_caption,
        fields: fields.get(..3).unwrap_or(&fields),
        leading: &leading,
        others: agreeing,
        verdict: &agree,
        differ_first: false,
    });
    view! {
        {card(tr(Msg::RecordsCompare), wide)}
        {card(tr(Msg::RecordsCompareStacked), stacked)}
        {card(agree_caption.clone(), all_agree)}
    }
    .into_any()
}

// ---- The work-list, on the tables page ----

/// One candidate pair of the review queue.
struct Pair {
    /// The pair's key.
    key: String,
    /// Record A's name and birth year.
    a: String,
    /// Record B's name and birth year.
    b: String,
    /// The two records' identifier keys.
    ids: [&'static str; 2],
    /// The match grade.
    grade: &'static str,
    /// How long it has waited.
    wait: String,
}

/// The review queue's pairs: twenty, so a row starts out of view.
fn pairs() -> Vec<Pair> {
    let keys = ["bsn", "nhs", "nir", "rrn", "pnr", "ssn", "mrn"];
    (1_u32..=20)
        .map(|number| {
            let index = usize::try_from(number).unwrap_or_default();
            Pair {
                key: format!("pair-{number:02}"),
                a: pair_names(number).0,
                b: pair_names(number).1,
                ids: [
                    keys.get(index % keys.len()).copied().unwrap_or("mrn"),
                    keys.get((index + 3) % keys.len()).copied().unwrap_or("mrn"),
                ],
                grade: if number <= 8 { "probable" } else { "possible" },
                wait: pair_wait(number),
            }
        })
        .collect()
}

/// The work-list's columns.
fn queue_columns() -> [Column; 7] {
    [
        Column::text("select", tr(Msg::RecordsSelectEvery)).role(Role::Check),
        Column::text("a", tr(Msg::RecordsRecordA)).role(Role::Title),
        Column::text("b", tr(Msg::RecordsRecordB)).role(Role::Sub),
        Column::text("domains", tr(Msg::RecordsDomains)).role(Role::Wide),
        Column::text("grade", tr(Msg::RecordsGrade)).sortable(),
        Column::text("waiting", tr(Msg::RecordsWaiting))
            .numeric()
            .role(Role::Quiet),
        Column::text("actions", tr(Msg::RecordsActions)).role(Role::Menu),
    ]
}

/// The row menu of a pair.
fn pair_menu(pair: &Pair, open: &str) -> AnyView {
    let name = format!("{} · {}", pair.a, pair.b);
    action_menu::row_menu(Menu::new(
        format!("{}-menu", pair.key),
        tr_with(Msg::RecordsActionsFor, &[("name", name.into())]),
        vec![Item::new(Action::new(
            Some(Glyph::ARROW_RIGHT),
            tr(Msg::RecordsOpenPair),
            Act::Link(open.to_owned()),
        ))],
    ))
}

/// The review queue as a work-list: a checkbox, both records, their
/// domains, the grade and the wait per row, its order of three keys stated
/// in the card's header row, the foot with the keys and the count.
#[must_use]
pub fn worklist_section() -> AnyView {
    let columns = queue_columns();
    let [_, _, b, domains, grade, waiting, _] = &columns;
    let open = crate::app::route_brand().map_or_else(
        || String::from("/"),
        |brand| format!("/{}/layouts", brand.key()),
    );
    let pairs = pairs();
    let total = u64::try_from(pairs.len()).unwrap_or(u64::MAX);
    let rows = pairs
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            let id = format!("{}-check", pair.key);
            let label = tr_with(Msg::RecordsSelect, &[("name", pair.a.clone().into())]);
            let tags = masked_all(&pair.ids)
                .iter()
                .map(|masked| identifier_static(masked, true))
                .collect_view();
            let tone = if pair.grade == "probable" {
                Tone::Info
            } else {
                Tone::Warn
            };
            let cells = view! {
                {check_cell(RowCheck {
                    id: &id,
                    name: "pairs",
                    value: &pair.key,
                    label: &label,
                    form: None,
                    checked: index == 3 || index == 4,
                })}
                {title_cell(Some(&open), pair.a.clone(), None)}
                {cell(b, pair.b.clone())}
                {cell(domains, view! { <span class="fui-cluster">{tags}</span> })}
                {cell(grade, badge(tone, pair.grade))}
                {cell(waiting, pair.wait.clone())}
                {menu_cell(pair_menu(pair, &open))}
            }
            .into_any();
            row(WorkRow {
                key: &pair.key,
                focused: index == 0,
                class: None,
                cells,
            })
        })
        .collect_view()
        .into_any();
    let order = tr(Msg::RecordsSortWords);
    let sort = Sort {
        column: String::from("grade"),
        direction: Direction::Descending,
    };
    let table = data_table(DataTable {
        caption: &tr(Msg::RecordsCandidatesCaption),
        columns: &columns,
        sorting: Sorting {
            sort: Some(&sort),
            stated: Some(&order),
            ..Sorting::default()
        },
        body: Body::Rows(rows),
    })
    .into_any();
    let hint = shortcut_hint(&[
        Shortcut {
            keys: String::from("j k | up down"),
            does: tr(Msg::RecordsMove),
        },
        Shortcut {
            keys: String::from("Enter"),
            does: tr(Msg::RecordsOpen),
        },
    ]);
    let foot = worklist_foot(Some(hint), None);
    let list =
        view! { <ferrohealth_ui::island::WorkList>{table}</ferrohealth_ui::island::WorkList> }
            .into_any();
    list_card(ListCard {
        title: tr(Msg::RecordsReviewQueue),
        count: Some(HeadingCount::All(total)),
        order: Some(order.clone()),
        tabs: None,
        actions: None,
        body: view! {
            {list}
            {foot}
        }
        .into_any(),
        pager: None,
    })
}

// ---- The timeline, on the layouts page ----

/// The events of a synthetic person's history, given oldest first, which
/// the timeline draws newest first.
fn events() -> Vec<Event> {
    let diff = [
        Change {
            key: tr(Msg::RecordsFamilyName),
            from: Some(FAMILY_CHANGE.0.to_owned()),
            to: Some(FAMILY_CHANGE.1.to_owned()),
        },
        Change {
            key: tr(Msg::RecordsPostalCode),
            from: None,
            to: Some(POSTAL_ADDED.to_owned()),
        },
    ];
    let caption = tr(Msg::RecordsHistoryVersion);
    vec![
        Event {
            key: String::from("created"),
            at: String::from("2026-09-01T08:00:00Z"),
            kind: tr(Msg::RecordsKindCreated),
            code: Some(String::from("create")),
            tone: Tone::Neutral,
            actor: None,
            title: tr(Msg::RecordsEventCreated),
            identifiers: masked_all(&["mrn"]),
            reason: None,
            body: None,
        },
        Event {
            key: String::from("linked"),
            at: String::from("2026-09-14T10:12:00Z"),
            kind: tr(Msg::RecordsKindLinked),
            code: Some(String::from("link.auto")),
            tone: Tone::Info,
            actor: Some(tr(Msg::RecordsMatcher)),
            title: tr(Msg::RecordsEventLinked),
            identifiers: masked_all(&["nhs", "nir"]),
            reason: None,
            body: None,
        },
        Event {
            key: String::from("version"),
            at: String::from("2026-10-02T13:40:00Z"),
            kind: tr(Msg::RecordsKindVersion),
            code: Some(String::from("version")),
            tone: Tone::Accent,
            actor: Some(tr(Msg::RecordsSteward)),
            title: tr(Msg::RecordsEventVersion),
            identifiers: Vec::new(),
            reason: Some(tr(Msg::RecordsReasonVersion)),
            body: Some(ViewFn::from(move || change_table(caption.clone(), &diff))),
        },
        Event {
            key: String::from("merged"),
            at: String::from("2026-10-20T09:05:00Z"),
            kind: tr(Msg::RecordsKindMerged),
            code: Some(String::from("merge")),
            tone: Tone::Warn,
            actor: Some(tr(Msg::RecordsSteward)),
            title: tr(Msg::RecordsEventMerged),
            identifiers: masked_all(&["bsn", "rrn", "pnr"]),
            reason: Some(tr(Msg::RecordsReasonMerged)),
            body: None,
        },
    ]
}

/// The timeline of a synthetic person.
#[must_use]
pub fn timeline_section() -> AnyView {
    card(
        tr(Msg::RecordsTimeline),
        timeline(tr(Msg::RecordsHistory), &events()),
    )
    .into_any()
}

// ---- The masked identifiers, on the forms page ----

/// Where a specimen row starts, as its server draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Start {
    /// Masked, its Reveal live.
    Masked,
    /// The round trip under way.
    Busy,
    /// Refused by the pretend server.
    Refused,
    /// Hidden again after its countdown.
    Expired,
}

/// One row of the identifiers table: its key, its masked form, where it
/// starts and whether the pretend server refuses its reveal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifierRow {
    /// The identifier's key.
    pub key: String,
    /// Its domain and visible tail.
    pub masked: Masked,
    /// Where it starts.
    pub start: Start,
    /// Whether a reveal is refused.
    pub refuses: bool,
}

/// The rows of the identifiers table: one per system, several countries.
fn identifier_rows() -> Vec<IdentifierRow> {
    IDENTIFIERS
        .iter()
        .filter_map(|each| {
            let start = match each.key {
                "nir" => Start::Busy,
                "rrn" => Start::Refused,
                "pnr" => Start::Expired,
                _ => Start::Masked,
            };
            masked(each.key).map(|masked| IdentifierRow {
                key: each.key.to_owned(),
                masked,
                start,
                refuses: matches!(each.key, "ssn" | "rrn"),
            })
        })
        .collect()
}

/// The words of the identifiers island, in the page's locale.
fn identifier_words() -> crate::locale::Words {
    crate::locale::Words::of(&[
        Msg::RecordsDomain,
        Msg::RecordsIdentifier,
        Msg::RecordsIdentifiersOf,
        Msg::RecordsRefusedWhy,
    ])
}

/// The masked identifiers: each system's identifier on its row, with the
/// reveal, and the specimens of a reveal under way, refused and expired.
#[must_use]
pub fn identifiers_section() -> AnyView {
    let reveal = use_query_map().with_untracked(|query| query.get("reveal"));
    let note = reveal.and_then(|key| synthetic(&key)).map(|each| {
        banner(
            Tone::Info,
            Glyph::EYE,
            tr_with(Msg::RecordsRevealTitle, &[("name", each.domain.into())]),
            &tr(Msg::RecordsRevealPage),
        )
    });
    let rows = identifier_rows();
    let words = identifier_words();
    let brand = crate::app::route_brand().map_or_else(String::new, |brand| brand.key().to_owned());
    let island = view! {
        <crate::islands::MaskedIdentifiers brand rows words ident=IdentifierWords::of_page() />
    }
    .into_any();
    titled_panel(
        tr(Msg::RecordsMasked),
        view! { <span class="fui-small fui-muted">{tr(Msg::RecordsMaskedNote)}</span> },
        view! {
            {note}
            {island}
        },
    )
    .into_any()
}

/// The pretend server's answer to a reveal of `key`: the whole synthetic
/// value, or the refusal.
fn answer(row: &IdentifierRow, words: &crate::locale::Words) -> Reveal {
    if row.refuses {
        return Reveal::Refused(words.get(Msg::RecordsRefusedWhy));
    }
    synthetic(&row.key).map_or(Reveal::Masked, |each| {
        Reveal::Shown(Revealed::new(each.value.to_owned(), REVEAL_SECONDS))
    })
}

/// The identifiers table the island [`crate::islands::MaskedIdentifiers`]
/// draws: each row's state its own, the reveal answered after a short
/// pause as a server would.
pub(crate) fn identifiers_body(
    brand: &str,
    rows: Vec<IdentifierRow>,
    words: &crate::locale::Words,
    ident: &IdentifierWords,
) -> AnyView {
    let body = rows
        .into_iter()
        .map(|each| {
            let state = RwSignal::new(match each.start {
                Start::Masked => Reveal::Masked,
                Start::Busy => Reveal::Busy,
                Start::Refused => Reveal::Refused(words.get(Msg::RecordsRefusedWhy)),
                Start::Expired => Reveal::Expired,
            });
            let live = each.start != Start::Busy;
            let answered = answer(&each, words);
            let on_reveal = Callback::new(move |()| {
                if live {
                    let answered = answered.clone();
                    set_timeout(
                        move || state.set(answered),
                        std::time::Duration::from_millis(400),
                    );
                }
            });
            let domain = each.masked.domain.label.clone();
            let fallback = format!("/{brand}/forms?reveal={}#identifiers", each.key);
            let widget = identifier(
                Identifier {
                    id: format!("identifier-{}", each.key),
                    masked: each.masked,
                    show_domain: false,
                    state,
                    fallback,
                    words: ident.clone(),
                },
                on_reveal,
            );
            view! {
                <tr data-key=each.key>
                    <th scope="row">{domain}</th>
                    <td data-label=words.get(Msg::RecordsIdentifier)>{widget}</td>
                </tr>
            }
        })
        .collect_view();
    let caption = words.get(Msg::RecordsIdentifiersOf);
    let label = caption.clone();
    view! {
        <div class="fui-table-scroll fui-table-stack" role="region" aria-label=label id="identifiers">
            <table class="fui-table">
                <caption>{caption}</caption>
                <thead>
                    <tr>
                        <th scope="col">{words.get(Msg::RecordsDomain)}</th>
                        <th scope="col">{words.get(Msg::RecordsIdentifier)}</th>
                    </tr>
                </thead>
                <tbody>{body}</tbody>
            </table>
        </div>
    }
    .into_any()
}
