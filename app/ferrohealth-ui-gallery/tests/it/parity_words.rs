// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The rules over words, counts and clocks written twice (#548): the
//! search's typeahead, the session watch, the upload field, the shared
//! phrase and digit grouping, the countdown and the identifier's reveal,
//! the combobox, the filter and the repeatable rows, each in the library's
//! Rust and again in its module, held to one answer over tables of inputs
//! (`parity` holds the harness).

use std::borrow::Cow;
use std::error::Error;

use ferrohealth_ui::i18n::number::Grouping;
use ferrohealth_ui::i18n::{Locale, Phrase};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::combobox::ComboOption;
use ferrohealth_ui::view::countdown::{clock, crossed, elapsed, ring_offset};
use ferrohealth_ui::view::filter::keeps;
use ferrohealth_ui::view::identifier::{Reveal, Revealed, ticked};
use ferrohealth_ui::view::repeat::fresh_key;
use ferrohealth_ui::view::search::{
    Escape, Hit, HitGroup, Mark, SearchAnswer, Side, Target, Way, escaped, hit_body, placed, runs,
    step, targets, wants,
};
use ferrohealth_ui::view::session::{Left, Message, plan, sign_in_address};
use ferrohealth_ui::view::upload::{Upload, UploadWords, admit, decode};
use serde_json::{Value, json};

use crate::parity::held;

/// Words typed into a field: empty, padded, cased, accented, outside the
/// BMP, and white space the two languages trim differently.
const TYPED: &[&str] = &[
    "",
    " ",
    "a",
    " ab ",
    "AB",
    "\u{c9}t\u{c9}",
    "\u{130}",
    "\u{3a3}A\u{3a3}",
    "\u{1f600}x",
    "stra\u{df}e",
    "\u{feff}ab",
    "ab\u{feff}",
    "\u{85}ab",
    "ab\u{85}",
    "\u{a0}ab\u{3000}",
    "\u{2028}ab",
];

#[tokio::test(flavor = "multi_thread")]
async fn the_search_typeahead_wants_words_and_marks_runs_alike() -> Result<(), Box<dyn Error>> {
    let values = [
        "Ana Doe",
        "ANA",
        "\u{c9}tienne \u{e9}t\u{e9}",
        "\u{130}stanbul i\u{307}",
        "Stra\u{df}e STRASSE",
        "\u{1f600}x\u{1f600}X",
        "\u{3a3}\u{3c3}\u{3c2}",
        "aaa",
        "",
    ];
    let mut cases = Vec::new();
    for words in TYPED {
        for from in [0_u8, 1, 2, 3] {
            cases.push((json!(["wants", words, from]), json!(wants(words, from))));
        }
        for value in values {
            let rust: Vec<Value> = runs(value, words)
                .into_iter()
                .map(|(run, marked)| json!([run, marked]))
                .collect();
            cases.push((json!(["runs", value, words]), json!(rust)));
        }
    }
    held(
        "search",
        "(m, ask, a, b) => ask === 'wants' ? m.wants(a, b) : m.runs(a, b)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_search_panel_steps_escapes_and_places_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for open in [true, false] {
        for empty in [true, false] {
            let named = match escaped(open, empty) {
                Escape::Close => "close",
                Escape::Clear => "clear",
                Escape::Pass => "pass",
            };
            cases.push((json!(["escaped", open, empty]), json!(named)));
        }
    }
    for count in 0..=4_usize {
        for active in std::iter::once(None).chain((0..=count).map(Some)) {
            for down in [true, false] {
                cases.push((
                    json!(["step", active, count, down]),
                    json!(step(active, count, down)),
                ));
            }
        }
    }
    for (left, right, wide, view) in [
        (16.0, 316.0, 400.0, 1280.0),
        (900.0, 1200.0, 400.0, 1280.0),
        (864.0, 1264.0, 400.0, 1280.0),
        (865.0, 1264.0, 400.0, 1280.0),
        (100.0, 415.0, 400.0, 420.0),
        (100.0, 416.0, 400.0, 420.0),
    ] {
        let side = match placed(left, right, wide, view) {
            Side::Start => "start",
            Side::End => "end",
            Side::Narrow => "narrow",
        };
        cases.push((json!(["placed", left, right, wide, view]), json!(side)));
    }
    held(
        "search",
        "(m, ask, a, b, c, d) => ask === 'escaped' ? m.escaped(a, b) : ask === 'step' ? m.step(a, b, c) : m.placed(a, b, c, d)",
        cases,
    )
    .await
}

/// A hit leading to `href`, named `name` over `line`.
fn hit(href: &str, name: &str, line: &str, mark: Option<Mark>, tag: Option<&str>) -> Hit {
    Hit {
        href: href.to_owned(),
        name: name.to_owned(),
        line: line.to_owned(),
        mark,
        tag: tag.map(str::to_owned),
    }
}

/// Answers of every shape: none, empty groups, more hits than a group
/// shows, and a way past.
fn answers() -> Vec<SearchAnswer> {
    let many: Vec<Hit> = (0..7)
        .map(|at| hit(&format!("/p/{at}"), "Ana", "", None, None))
        .collect();
    let group = |kind: &str, hits: Vec<Hit>| HitGroup {
        kind: kind.to_owned(),
        hits,
    };
    let way = Some(Way {
        href: String::from("/all"),
        words: String::from("Browse all"),
    });
    vec![
        SearchAnswer::default(),
        SearchAnswer {
            groups: vec![group("Empty", Vec::new())],
            ..SearchAnswer::default()
        },
        SearchAnswer {
            groups: vec![group("Empty", Vec::new())],
            more: way.clone(),
            ..SearchAnswer::default()
        },
        SearchAnswer {
            groups: vec![
                group("Patients", many.clone()),
                group("Empty", Vec::new()),
                group("Files", many.into_iter().take(2).collect()),
            ],
            more: way,
            ..SearchAnswer::default()
        },
    ]
}

#[tokio::test(flavor = "multi_thread")]
async fn the_search_panel_offers_the_same_targets() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for answer in answers() {
        let rust: Vec<Value> = targets(&answer)
            .into_iter()
            .map(|target| match target {
                Target::Hit(href) => json!({"hit": href}),
                Target::All => json!({"all": true}),
                Target::Way(href) => json!({"way": href}),
            })
            .collect();
        cases.push((json!([serde_json::to_value(&answer)?]), json!(rust)));
    }
    held("search", "(m, answer) => m.targets(answer)", cases).await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_search_panel_draws_a_hit_as_the_find_page_does() -> Result<(), Box<dyn Error>> {
    let hits = [
        hit("/a", "Ana Doe", "Born 1985", None, None),
        hit(
            "/b",
            "ANA <b>&",
            "a 'q' \"d\"",
            Some(Mark::Dot(Tone::Ok)),
            Some("MRN 1"),
        ),
        hit(
            "/c",
            "\u{c9}t\u{e9}",
            "",
            Some(Mark::Dot(Tone::Neutral)),
            None,
        ),
        hit("/d", "x", "y", Some(Mark::Dot(Tone::Bad)), Some("<t>")),
        hit("/e", "Ward", "Ana", Some(Mark::Glyph(Glyph::SEARCH)), None),
    ];
    let mut cases = Vec::new();
    for hit in &hits {
        for query in ["", "ana", "\u{e9}", "<", "zz"] {
            cases.push((
                json!([
                    serde_json::to_value(hit)?,
                    query,
                    hit_body(hit, query).to_string()
                ]),
                json!(true),
            ));
        }
    }
    held(
        "search",
        "(m, hit, query, rust) => {
          const canon = (node) => {
            if (node.nodeType === 3) return JSON.stringify(node.data);
            const names = [...node.attributes].map((a) => a.name + '=' + JSON.stringify(a.value)).sort();
            return '<' + node.localName + ' ' + names.join(' ') + '>' + [...node.childNodes].map(canon).join('') + '</' + node.localName + '>';
          };
          const holder = document.createElement('div');
          holder.innerHTML = rust;
          const icon = (name) => holder.querySelector('svg') ? holder.querySelector('svg').cloneNode(true) : document.createTextNode('no glyph ' + name);
          const drawn = document.createElement('div');
          drawn.append(...m.hitBody(hit, query, icon).filter((part) => part !== null));
          drawn.normalize();
          holder.normalize();
          return canon(drawn) === canon(holder) || [canon(drawn), canon(holder)];
        }",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_session_plans_and_tells_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for (idle, absolute) in [
        (0, 0),
        (900, 3600),
        (3600, 900),
        (900, 900),
        (10, 3600),
        (30, 30),
    ] {
        let left = Left { idle, absolute };
        for warn_at in [0, 20, 120, 2000] {
            let next = plan(left, warn_at);
            cases.push((
                json!(["plan", {"idle": idle, "absolute": absolute}, warn_at]),
                json!({"warnIn": next.warn_in, "endIn": next.end_in}),
            ));
        }
        for seconds in [0, 5, 900, 5000] {
            let later = left.after(seconds);
            cases.push((
                json!(["after", {"idle": idle, "absolute": absolute}, seconds]),
                json!({"idle": later.idle, "absolute": later.absolute}),
            ));
        }
        cases.push((
            json!(["text", {"left": {"idle": idle, "absolute": absolute}}, 0]),
            json!(Message::Extended(left).text()),
        ));
    }
    cases.push((
        json!(["text", {"ended": true}, 0]),
        json!(Message::Ended.text()),
    ));
    for text in [
        "ended",
        "ended ",
        " ended",
        "ENDED",
        "extended 10 20",
        "extended 0 0",
        "extended 10",
        "extended 10 20 30",
        "extended  10 20",
        "extended +10 20",
        "extended -1 20",
        "extended 010 20",
        "extended 4294967295 1",
        "extended 4294967296 1",
        "extended 99999999999999999999 1",
        "extended 1.5 2",
        "extended \u{661} 2",
        "",
        "other",
    ] {
        let parsed = Message::parse(text).map(|message| match message {
            Message::Ended => json!({"ended": true}),
            Message::Extended(left) => {
                json!({"left": {"idle": left.idle, "absolute": left.absolute}})
            }
        });
        cases.push((json!(["parse", text, 0]), json!(parsed)));
    }
    held(
        "session",
        "(m, ask, a, b) => ask === 'plan' ? m.plan(a, b) : ask === 'after' ? m.after(a, b) : ask === 'text' ? m.messageText(a) : m.parseMessage(a)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_session_sign_in_address_is_written_alike() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for sign_in in ["/sign-in", "/sign-in?brand=a", "https://id.example/in"] {
        for here in [
            "/",
            "/records?page=2&size=20",
            "/a b/\u{e9}\u{1f600}",
            "/x#frag",
            "/~user/a.b_c-d",
            "/100%",
            "",
        ] {
            cases.push((
                json!([sign_in, here]),
                json!(sign_in_address(sign_in, here)),
            ));
        }
    }
    held(
        "session",
        "(m, signIn, here) => m.signInAddress(signIn, here)",
        cases,
    )
    .await
}

/// The extensions the upload tables accept.
const ACCEPT: &[&[&str]] = &[&[], &[".json"], &[".json", ".YAML", ".xml"], &[".kml"]];

#[tokio::test(flavor = "multi_thread")]
async fn the_upload_field_sizes_and_refuses_a_file_alike() -> Result<(), Box<dyn Error>> {
    let english = Locale::english();
    let mut cases = Vec::new();
    for accept in ACCEPT {
        let words = UploadWords::of(&english, accept, 1024 * 1024);
        let wire = serde_json::to_value(&words)?;
        for max in [0_u64, 1, 1024, 5 * 1024 * 1024] {
            let upload = Upload {
                id: "parity",
                label: Cow::Borrowed("Parity"),
                choose: Cow::Borrowed("Choose"),
                accept,
                max_bytes: max,
                placeholder: Cow::Borrowed(""),
                words: words.clone(),
            };
            for name in [
                "a.json",
                "A.JSON",
                "notes.yaml",
                "map.xml.bak",
                "x.\u{212a}ml",
                "\u{130}.json",
                ".json",
                "json",
                "",
            ] {
                for bytes in [
                    0_u64,
                    1,
                    2,
                    1023,
                    1024,
                    1025,
                    1_048_576,
                    1_048_577,
                    123_456_789,
                ] {
                    let refusal = admit(&upload, name, bytes)
                        .err()
                        .map(|refused| refused.sentence(&words).text().to_owned());
                    cases.push((
                        json!(["refusal", wire, accept, max, name, bytes]),
                        json!(refusal),
                    ));
                }
            }
        }
        for bytes in [
            0_u64,
            1,
            2,
            999,
            1023,
            1024,
            1025,
            1_048_575,
            1_048_576,
            1_048_577,
            9_999_999_999,
        ] {
            cases.push((
                json!(["size", wire, accept, 0, "", bytes]),
                json!(words.sizes.size(bytes).text()),
            ));
        }
    }
    held(
        "upload",
        "(m, ask, words, accept, max, name, bytes) => ask === 'size' ? m.size(words.sizes, bytes) : m.refusal(words, accept, max, name, bytes)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_upload_field_decodes_a_file_alike() -> Result<(), Box<dyn Error>> {
    let files: &[&[u8]] = &[
        b"",
        b"plain text",
        b"\xef\xbb\xbfwith a mark",
        b"\xef\xbb\xbf\xef\xbb\xbftwo marks",
        b"\xef\xbbhalf a mark",
        "\u{e9}\u{1f600}".as_bytes(),
        b"bad \xff byte",
        b"cut \xe2\x82",
        b"\xc0\xaf overlong",
        b"\xed\xa0\x80 surrogate",
        b"\xf4\x90\x80\x80 past the end",
        b"\xf0\x9f\x98 three of four \xf0\x9f",
        b"\x00 nul",
    ];
    let cases = files
        .iter()
        .map(|bytes| {
            let (text, lossy) = decode(bytes);
            (json!([bytes]), json!({"text": text, "lossy": lossy}))
        })
        .collect();
    held(
        "upload",
        "(m, bytes) => m.decode(new Uint8Array(bytes))",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_shared_phrase_and_grouping_are_written_alike() -> Result<(), Box<dyn Error>> {
    let open = |name| Phrase::open_mark(name);
    let phrases = [
        String::new(),
        String::from("no marks"),
        format!("{} read.", open("file")),
        format!("{} and {} and {}", open("a"), open("b"), open("a")),
        format!("{}{}", open("a"), open("a")),
        format!("{} of {}", open("count"), open("missing")),
    ];
    let values = [
        vec![],
        vec![("a", "x"), ("file", "notes.txt")],
        vec![("a", "\u{e000}b\u{e001}"), ("b", "y")],
        vec![("a", "$&"), ("count", "1,024")],
    ];
    let mut cases = Vec::new();
    for phrase in &phrases {
        for wire in [json!(phrase), json!({"en": phrase})] {
            let read: Phrase = serde_json::from_value(wire.clone())?;
            for filled in &values {
                let object: serde_json::Map<String, Value> = filled
                    .iter()
                    .map(|(name, value)| ((*name).to_owned(), json!(value)))
                    .collect();
                cases.push((
                    json!(["fill", wire, object]),
                    json!(read.fill(filled).text()),
                ));
            }
        }
    }
    for group in [",", ".", "\u{202f}", "'", ""] {
        let grouping: Grouping = serde_json::from_value(json!({"group": group}))?;
        for count in [
            0_u64,
            7,
            999,
            1000,
            12_480,
            999_999,
            1_000_000,
            123_456_789_012,
        ] {
            cases.push((
                json!(["grouped", count, group]),
                json!(grouping.write(count)),
            ));
        }
    }
    held(
        "shared",
        "(m, ask, a, b) => ask === 'fill' ? m.fill(a, b) : m.grouped(a, b)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_countdown_clock_ring_and_set_points_agree() -> Result<(), Box<dyn Error>> {
    let mut cases = Vec::new();
    for seconds in [0_u32, 9, 59, 60, 61, 599, 3599, 3600, 3661, 86_399, 360_000] {
        cases.push((json!(["clock", seconds, 0]), json!(clock(seconds))));
    }
    for (left, total) in [
        (0, 0),
        (0, 60),
        (30, 60),
        (60, 60),
        (90, 60),
        (1, 3),
        (2, 3),
        (59, 60),
    ] {
        cases.push((
            json!(["ring", left, total]),
            json!(ring_offset(left, total)),
        ));
    }
    for before in [0_u32, 10, 11, 30, 31, 60, 61, 120] {
        for now in [0_u32, 9, 10, 29, 30, 59, 60, 61] {
            cases.push((json!(["crossed", before, now]), json!(crossed(before, now))));
        }
    }
    for (monotonic, wall) in [(0.0, 0.0), (1500.0, 900.0), (900.0, 1500.0), (-5.0, 3.0)] {
        cases.push((
            json!(["elapsed", monotonic, wall]),
            json!(elapsed(monotonic, wall)),
        ));
    }
    held(
        "countdown",
        "(m, ask, a, b) => ask === 'clock' ? m.clock(a) : ask === 'ring' ? m.ringOffset(a, b) : ask === 'crossed' ? m.crossed(a, b) : m.elapsed(a, b)",
        cases,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_identifier_reveal_counts_down_alike() -> Result<(), Box<dyn Error>> {
    let cases = [0_u32, 1, 2, 3, 60, 4_294_967_295]
        .into_iter()
        .map(|seconds| {
            let left = match ticked(Reveal::Shown(Revealed::new(String::from("x"), seconds))) {
                Reveal::Shown(revealed) => revealed.seconds(),
                _ => 0,
            };
            (json!([seconds.max(1)]), json!(left))
        })
        .collect();
    held("identifier", "(m, seconds) => m.ticked(seconds)", cases).await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_combobox_and_the_filter_keep_alike() -> Result<(), Box<dyn Error>> {
    let options = [
        ComboOption::new("nl", "Netherlands").detail("NL"),
        ComboOption::new("ab", "Abbey road"),
        ComboOption::new("tr", "T\u{fc}rkiye \u{130}stanbul"),
        ComboOption::new("de", "Stra\u{df}e"),
        ComboOption::new("gr", "\u{3a3}\u{39f}\u{3a3}"),
        ComboOption::new("x", "\u{1f600} emoji").detail("\u{c9}T\u{c9}"),
    ];
    let mut cases = Vec::new();
    for words in TYPED.iter().chain(&[
        "nl",
        "NL",
        "t\u{fc}r",
        "i\u{307}",
        "\u{3c3}\u{3bf}\u{3c2}",
        "\u{e9}t",
    ]) {
        let needle = words.trim().to_lowercase();
        for option in &options {
            cases.push((
                json!(["matches", serde_json::to_value(option)?, needle]),
                json!(option.matches(&needle)),
            ));
        }
        let texts: Vec<&str> = options.iter().map(|option| option.label.as_str()).collect();
        cases.push((json!(["keeps", words, texts]), json!(keeps(words, &texts))));
    }
    held(
        "combobox",
        "(m, ask, a, b) => ask === 'matches' ? m.matches(a, b) : null",
        cases
            .iter()
            .filter(|(input, _)| input[0] == "matches")
            .cloned()
            .collect(),
    )
    .await?;
    held(
        "filter",
        "(m, ask, words, texts) => m.keeps(words, texts)",
        cases
            .into_iter()
            .filter(|(input, _)| input[0] == "keeps")
            .collect(),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn the_repeat_rows_take_the_same_fresh_key() -> Result<(), Box<dyn Error>> {
    let taken_sets: [&[&str]; 4] = [
        &[],
        &["new-1"],
        &["new-1", "new-2", "new-4"],
        &["new-2", "a", "new-01"],
    ];
    let cases = taken_sets
        .iter()
        .map(|taken| (json!([taken]), json!(fresh_key(|key| taken.contains(&key)))))
        .collect();
    held(
        "repeat",
        "(m, taken) => m.freshKey((key) => taken.includes(key))",
        cases,
    )
    .await
}
