// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The patient-record components, drawn on three pages.
//!
//! The record compare and the work-list are on the tables page, the
//! timeline on the layouts page, and the masked identifiers with their
//! reveal on the forms page. Their
//! people, records and identifiers are the synthetic data of
//! [`crate::records_data`], identifiers of several countries side by side.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Direction, Sort};
use ferrohealth_ui::table::view::{
    Body, Column, DataTable, Role, RowCheck, Sorting, cell, check_cell, data_table, menu_cell,
    title_cell,
};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action_menu::{self, Act, Action, Item, Menu};
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::change::{Change, change_table};
use ferrohealth_ui::view::compare::{
    Compare, CompareField, CompareRecord, Verdict, compare, compare_stacked,
};
use ferrohealth_ui::view::identifier::{
    Domain, Identifier, IdentifierWords, KEEP, Masked, Reveal, Revealed, identifier,
    identifier_list, identifier_static,
};
use ferrohealth_ui::view::kbd::{Shortcut, shortcut_hint};
use ferrohealth_ui::view::side_panel::{
    Opener, SidePanel, SidePanelWords, side_panel_with_shortcut,
};
use ferrohealth_ui::view::status::{BannerLink, badge, banner};
use ferrohealth_ui::view::surface::{card, definitions, titled_panel};
use ferrohealth_ui::view::template::list::{HeadingCount, ListCard, list_card};
use ferrohealth_ui::view::timeline::{Event, timeline};
use ferrohealth_ui::view::worklist::{WorkRow, row, worklist, worklist_foot};

use crate::locale::Msg;
use crate::markup::{h, t};
use crate::records_data::{
    FAMILY_CHANGE, IDENTIFIERS, LEADING, OTHERS, POSTAL_ADDED, SourceRecord, Synthetic, pair_names,
    pair_wait,
};
use crate::visit::Visit;

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

/// A record of the compare: its values per field, the last its identifiers.
fn record(v: &Visit, source: &SourceRecord) -> CompareRecord {
    let ids = masked_all(source.ids);
    let mut cells: Vec<Option<Html>> = source
        .values
        .into_iter()
        .map(|value| value.map(t))
        .collect();
    cells.push((!ids.is_empty()).then(|| identifier_list(&v.locale, &ids)));
    CompareRecord {
        key: source.key.to_owned(),
        title: source.title.to_owned(),
        sub: Some(source.sub.to_owned()),
        values: cells,
    }
}

/// The fields the records are compared on.
fn fields(v: &Visit) -> Vec<CompareField> {
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
        label: v.tr(msg),
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
pub fn compare_section(v: &Visit) -> Html {
    let fields = fields(v);
    let leading = record(v, &LEADING);
    let others = OTHERS.each_ref().map(|source| record(v, source));
    let caption = v.tr(Msg::RecordsCompareCaption);
    let table = Compare {
        id: "records-compare",
        caption: &caption,
        fields: &fields,
        leading: &leading,
        others: &others,
        verdict: &verdict,
        differ_first: true,
    };
    let wide = compare(&v.locale, table);
    let stacked = compare_stacked(
        &v.locale,
        Compare {
            id: "records-compare-stacked",
            ..table
        },
        0,
    );
    let agreeing = &others[..1];
    let agree_caption = v.tr(Msg::RecordsCompareCaptionAgree);
    let agree = |field: &str, _: usize| {
        if field == "ids" || field == "postal" {
            Verdict::Unmarked
        } else {
            Verdict::Agree
        }
    };
    let all_agree = compare(
        &v.locale,
        Compare {
            id: "records-compare-agree",
            caption: &agree_caption,
            fields: fields.get(..3).unwrap_or(&fields),
            leading: &leading,
            others: agreeing,
            verdict: &agree,
            differ_first: false,
        },
    );
    Html::join([
        card(v.tr(Msg::RecordsCompare), wide),
        card(v.tr(Msg::RecordsCompareStacked), stacked),
        card(agree_caption.clone(), all_agree),
    ])
}

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
fn queue_columns(v: &Visit) -> [Column; 7] {
    [
        Column::text("select", v.tr(Msg::RecordsSelectEvery)).role(Role::Check),
        Column::text("a", v.tr(Msg::RecordsRecordA)).role(Role::Title),
        Column::text("b", v.tr(Msg::RecordsRecordB)).role(Role::Sub),
        Column::text("domains", v.tr(Msg::RecordsDomains)).role(Role::Wide),
        Column::text("grade", v.tr(Msg::RecordsGrade)).sortable(),
        Column::text("waiting", v.tr(Msg::RecordsWaiting))
            .numeric()
            .role(Role::Quiet),
        Column::text("actions", v.tr(Msg::RecordsActions)).role(Role::Menu),
    ]
}

/// The row menu of a pair.
fn pair_menu(v: &Visit, pair: &Pair, open: &str) -> Html {
    let name = format!("{} · {}", pair.a, pair.b);
    action_menu::row_menu(
        &v.locale,
        &Menu::new(
            format!("{}-menu", pair.key),
            v.tr_with(Msg::RecordsActionsFor, &[("name", name.into())]),
            vec![Item::new(Action::new(
                Some(Glyph::ARROW_RIGHT),
                v.tr(Msg::RecordsOpenPair),
                Act::Link(open.to_owned()),
            ))],
        ),
    )
}

/// The side panel that holds the pair `pair` at a glance, which its row's
/// `o` opens while the row holds the focus (#530); with no script its
/// opener leads to the pair's page, `open`.
fn glance(v: &Visit, pair: &Pair, open: &str) -> Html {
    let name = format!("{} · {}", pair.a, pair.b);
    side_panel_with_shortcut(
        SidePanel {
            id: format!("{}-glance", pair.key),
            title: v.tr_with(Msg::RecordsGlanceOf, &[("name", name.into())]),
            opener: Opener {
                label: v.tr(Msg::RecordsGlance),
                glyph: Some(Glyph::EYE),
                variant: Variant::Ghost,
                href: open.to_owned(),
            },
            body: definitions(vec![
                (v.tr(Msg::RecordsRecordA), t(&pair.a)),
                (v.tr(Msg::RecordsRecordB), t(&pair.b)),
                (v.tr(Msg::RecordsGrade), t(pair.grade)),
                (v.tr(Msg::RecordsWaiting), t(&pair.wait)),
            ]),
            words: SidePanelWords::of(&v.locale),
        },
        "o",
    )
}

/// The review queue as a work-list: a checkbox, both records, their
/// domains, the grade and the wait per row, its order of three keys stated
/// in the card's header row, the foot with the keys and the count.
#[must_use]
pub fn worklist_section(v: &Visit) -> Html {
    let columns = queue_columns(v);
    let [_, _, b, domains, grade, waiting, _] = &columns;
    let open = format!("/{}/layouts", v.brand.key());
    let pairs = pairs();
    let total = u64::try_from(pairs.len()).unwrap_or(u64::MAX);
    let rows: Html = pairs
        .iter()
        .enumerate()
        .map(|(index, pair)| {
            let id = format!("{}-check", pair.key);
            let label = v.tr_with(Msg::RecordsSelect, &[("name", pair.a.clone().into())]);
            let tags: Html = masked_all(&pair.ids)
                .iter()
                .map(|masked| identifier_static(&v.locale, masked, true))
                .collect();
            let tone = if pair.grade == "probable" {
                Tone::Info
            } else {
                Tone::Warn
            };
            let cells = Html::join([
                check_cell(RowCheck {
                    id: &id,
                    name: "pairs",
                    value: &pair.key,
                    label: &label,
                    form: None,
                    checked: index == 3 || index == 4,
                }),
                title_cell(Some(&open), t(&pair.a), None),
                cell(b, t(&pair.b)),
                cell(
                    domains,
                    h(format!("<span class=\"fui-cluster\">{tags}</span>")),
                ),
                cell(grade, badge(tone, pair.grade)),
                cell(waiting, t(&pair.wait)),
                menu_cell(Html::join([
                    glance(v, pair, &open),
                    pair_menu(v, pair, &open),
                ])),
            ]);
            row(WorkRow {
                key: &pair.key,
                focused: index == 0,
                class: None,
                cells,
            })
        })
        .collect();
    let order = v.tr(Msg::RecordsSortWords);
    let sort = Sort {
        column: String::from("grade"),
        direction: Direction::Descending,
    };
    let table = data_table(DataTable {
        caption: &v.tr(Msg::RecordsCandidatesCaption),
        columns: &columns,
        sorting: Sorting {
            sort: Some(&sort),
            stated: Some(&order),
            ..Sorting::default()
        },
        body: Body::Rows(rows),
    });
    let hint = shortcut_hint(
        &v.locale,
        &[
            Shortcut {
                keys: String::from("j k | up down"),
                does: v.tr(Msg::RecordsMove),
            },
            Shortcut {
                keys: String::from("Enter"),
                does: v.tr(Msg::RecordsOpen),
            },
            Shortcut {
                keys: String::from("o"),
                does: v.tr(Msg::RecordsGlanceKey),
            },
        ],
    );
    let foot = worklist_foot(Some(hint), None);
    let list = worklist(table);
    list_card(
        &v.locale,
        ListCard {
            title: v.tr(Msg::RecordsReviewQueue),
            count: Some(HeadingCount::All(total)),
            order: Some(order.clone()),
            tabs: None,
            actions: None,
            body: Html::join([list, foot]),
            pager: None,
        },
    )
}

/// The events of a synthetic person's history, given oldest first, which
/// the timeline draws newest first.
fn events(v: &Visit) -> Vec<Event> {
    let diff = [
        Change {
            key: v.tr(Msg::RecordsFamilyName),
            from: Some(FAMILY_CHANGE.0.to_owned()),
            to: Some(FAMILY_CHANGE.1.to_owned()),
        },
        Change {
            key: v.tr(Msg::RecordsPostalCode),
            from: None,
            to: Some(POSTAL_ADDED.to_owned()),
        },
    ];
    let caption = v.tr(Msg::RecordsHistoryVersion);
    vec![
        Event {
            key: String::from("created"),
            at: String::from("2026-09-01T08:00:00Z"),
            kind: v.tr(Msg::RecordsKindCreated),
            code: Some(String::from("create")),
            tone: Tone::Neutral,
            actor: None,
            title: v.tr(Msg::RecordsEventCreated),
            identifiers: masked_all(&["mrn"]),
            reason: None,
            body: None,
        },
        Event {
            key: String::from("linked"),
            at: String::from("2026-09-14T10:12:00Z"),
            kind: v.tr(Msg::RecordsKindLinked),
            code: Some(String::from("link.auto")),
            tone: Tone::Info,
            actor: Some(v.tr(Msg::RecordsMatcher)),
            title: v.tr(Msg::RecordsEventLinked),
            identifiers: masked_all(&["nhs", "nir"]),
            reason: None,
            body: None,
        },
        Event {
            key: String::from("version"),
            at: String::from("2026-10-02T13:40:00Z"),
            kind: v.tr(Msg::RecordsKindVersion),
            code: Some(String::from("version")),
            tone: Tone::Accent,
            actor: Some(v.tr(Msg::RecordsSteward)),
            title: v.tr(Msg::RecordsEventVersion),
            identifiers: Vec::new(),
            reason: Some(v.tr(Msg::RecordsReasonVersion)),
            body: Some(change_table(&v.locale, caption.clone(), &diff)),
        },
        Event {
            key: String::from("merged"),
            at: String::from("2026-10-20T09:05:00Z"),
            kind: v.tr(Msg::RecordsKindMerged),
            code: Some(String::from("merge")),
            tone: Tone::Warn,
            actor: Some(v.tr(Msg::RecordsSteward)),
            title: v.tr(Msg::RecordsEventMerged),
            identifiers: masked_all(&["bsn", "rrn", "pnr"]),
            reason: Some(v.tr(Msg::RecordsReasonMerged)),
            body: None,
        },
    ]
}

/// The timeline of a synthetic person.
#[must_use]
pub fn timeline_section(v: &Visit) -> Html {
    card(
        v.tr(Msg::RecordsTimeline),
        timeline(&v.locale, v.tr(Msg::RecordsHistory), &events(v)),
    )
}

/// Where a specimen row starts, as its server draws it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Start {
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
#[derive(Debug, Clone, PartialEq, Eq)]
struct IdentifierRow {
    /// The identifier's key.
    key: String,
    /// Its domain and visible tail.
    masked: Masked,
    /// Where it starts.
    start: Start,
    /// Whether a reveal is refused.
    refuses: bool,
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

/// The pretend server's answer to a reveal of `row`: the whole synthetic
/// value, or the refusal.
fn answer(v: &Visit, row: &IdentifierRow) -> Reveal {
    if row.refuses {
        return Reveal::Refused(v.tr(Msg::RecordsRefusedWhy));
    }
    synthetic(&row.key).map_or(Reveal::Masked, |each| {
        Reveal::Shown(Revealed::new(each.value.to_owned(), REVEAL_SECONDS))
    })
}

/// The identifiers table: each row in its own state, each the address's
/// `reveal` names answered as the console's reveal page would answer it.
fn identifiers_table(v: &Visit, revealed: &[&str]) -> Html {
    let words = IdentifierWords::of(&v.locale);
    let brand = v.brand.key();
    let body: Html = identifier_rows()
        .into_iter()
        .map(|each| {
            let state = if revealed.contains(&each.key.as_str()) {
                answer(v, &each)
            } else {
                match each.start {
                    Start::Masked => Reveal::Masked,
                    Start::Busy => Reveal::Busy,
                    Start::Refused => Reveal::Refused(v.tr(Msg::RecordsRefusedWhy)),
                    Start::Expired => Reveal::Expired,
                }
            };
            let domain = each.masked.domain.label.clone();
            let widget = identifier(Identifier {
                id: format!("identifier-{}", each.key),
                fallback: format!("/{brand}/forms?reveal={}#identifiers", each.key),
                masked: each.masked,
                show_domain: false,
                state,
                words: words.clone(),
            });
            h(format!(
                "<tr data-key=\"{}\"><th scope=\"row\">{}</th><td data-label=\"{}\">{widget}</td></tr>",
                t(&each.key),
                t(domain),
                t(v.tr(Msg::RecordsIdentifier))
            ))
        })
        .collect();
    let caption = t(v.tr(Msg::RecordsIdentifiersOf));
    h(format!(
        "<div class=\"fui-table-scroll fui-table-stack\" role=\"region\" aria-label=\"{caption}\" id=\"identifiers\"><table class=\"fui-table\"><caption>{caption}</caption><thead><tr><th scope=\"col\">{}</th><th scope=\"col\">{}</th></tr></thead><tbody>{body}</tbody></table></div>",
        t(v.tr(Msg::RecordsDomain)),
        t(v.tr(Msg::RecordsIdentifier))
    ))
}

/// The masked identifiers: each system's identifier on its row, with the
/// reveal, and the specimens of a reveal under way, refused and expired.
#[must_use]
pub fn identifiers_section(v: &Visit) -> Html {
    let reveal = v.param("reveal");
    let revealed: Vec<&str> = v
        .query
        .pairs()
        .iter()
        .filter(|(name, _)| name == "reveal")
        .map(|(_, key)| key.as_str())
        .collect();
    let note = reveal
        .and_then(synthetic)
        .map(|each| {
            banner(
                Tone::Info,
                Glyph::EYE,
                v.tr_with(Msg::RecordsRevealTitle, &[("name", each.domain.into())]),
                &v.tr(Msg::RecordsRevealPage),
                Some(BannerLink {
                    words: v.tr(Msg::RecordsRevealStay),
                    href: String::from("?"),
                }),
            )
        })
        .unwrap_or_default();
    titled_panel(
        v.tr(Msg::RecordsMasked),
        h(format!(
            "<span class=\"fui-small fui-muted\">{}</span>",
            t(v.tr(Msg::RecordsMaskedNote))
        )),
        Html::join([note, identifiers_table(v, &revealed)]),
    )
}
