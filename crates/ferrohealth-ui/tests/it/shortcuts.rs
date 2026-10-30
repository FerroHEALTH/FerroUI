// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The keys a console binds through the `keys` module (#526, #530): the
//! legend that loads it and lists the go-to pairs as their links, and the
//! side panel opener that announces a row's key.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Word;
use ferrohealth_ui::script;
use ferrohealth_ui::view::button::Variant;
use ferrohealth_ui::view::kbd::{Shortcut, ShortcutGroup};
use ferrohealth_ui::view::shortcuts::{
    GoTo, GoToPair, KEYS_JOB, shortcut_legend, shortcut_legend_with_go_to,
};
use ferrohealth_ui::view::side_panel::{
    Opener, SidePanel, SidePanelWords, side_panel, side_panel_with_shortcut,
};

use crate::markup::{has, tags};
use crate::render::{en, html};

/// The go-to pairs of a terminology console.
fn go_to() -> GoTo {
    GoTo {
        title: String::from("Go to"),
        lead: String::from("g"),
        pairs: vec![
            GoToPair {
                key: String::from("d"),
                href: String::from("/"),
                does: String::from("Dashboard"),
            },
            GoToPair {
                key: String::from("c"),
                href: String::from("/code-systems?q=<a>"),
                does: String::from("Code systems"),
            },
        ],
    }
}

/// The groups the legend lists before the pairs.
fn groups() -> Vec<ShortcutGroup> {
    vec![ShortcutGroup {
        title: String::from("Everywhere"),
        shortcuts: vec![Shortcut {
            keys: String::from("?"),
            does: String::from("Show the shortcuts"),
        }],
    }]
}

#[test]
fn the_keys_module_ships_under_its_job() {
    let module = script::module(&format!("{KEYS_JOB}.js")).expect("keys.js ships");
    assert!(
        module.text.contains("aria-keyshortcuts"),
        "it reads what a control announces"
    );
}

#[test]
fn every_legend_loads_the_keys_module_beside_its_own() {
    let legend = html(shortcut_legend(&en(), groups()));
    assert!(
        has(
            &legend,
            "dialog",
            &[("id", "shortcuts"), ("data-fui", "dialog shortcuts keys")]
        ),
        "{legend}"
    );
    assert!(
        !legend.contains("data-fui-go"),
        "no pairs, no go-to group: {legend}"
    );
}

#[test]
fn the_go_to_pairs_follow_the_groups_each_listed_as_its_link() {
    let legend = html(shortcut_legend_with_go_to(&en(), groups(), go_to()));
    assert_eq!(2, tags(&legend, "section").len(), "{legend}");
    assert!(
        has(
            &legend,
            "section",
            &[("aria-labelledby", "shortcuts-go"), ("data-fui-go", "g")]
        ),
        "{legend}"
    );
    assert!(has(&legend, "h3", &[("id", "shortcuts-go")]), "{legend}");
    assert!(legend.contains(">Go to</h3>"), "{legend}");
    assert!(
        has(&legend, "a", &[("href", "/"), ("data-fui-go-key", "d")]),
        "a pair is the link it follows: {legend}"
    );
    assert!(
        has(
            &legend,
            "a",
            &[
                ("href", "/code-systems?q=&#60;a&#62;"),
                ("data-fui-go-key", "c")
            ]
        ),
        "the address is escaped: {legend}"
    );
    assert!(
        legend.contains(r#"<kbd class="fui-kbd">G</kbd><kbd class="fui-kbd">C</kbd>"#),
        "the keys pressed in turn: {legend}"
    );
}

#[test]
fn the_go_to_pairs_read_as_a_group_for_a_console_s_own_page() {
    let group = go_to().group();
    assert_eq!("Go to", group.title);
    assert_eq!(
        vec!["g d", "g c"],
        group
            .shortcuts
            .iter()
            .map(|shortcut| shortcut.keys.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!("Code systems", group.shortcuts[1].does);
}

/// A row's side panel, opened by `o` when `keys` holds it.
fn panel(keys: Option<&str>) -> String {
    let panel = SidePanel {
        id: String::from("pair-17-glance"),
        title: String::from("The pair at a glance"),
        opener: Opener {
            label: String::from("Glance"),
            glyph: None,
            variant: Variant::Ghost,
            href: String::from("/queue/pair-17"),
        },
        body: Html::raw(String::from("<p>Two synthetic records.</p>")),
        words: SidePanelWords {
            close: Word::from("Close"),
        },
    };
    html(match keys {
        Some(keys) => side_panel_with_shortcut(panel, keys),
        None => side_panel(panel),
    })
}

#[test]
fn a_keyed_opener_announces_its_keys_and_asks_the_keys_module() {
    let keyed = panel(Some("o | d"));
    assert!(
        has(
            &keyed,
            "button",
            &[
                ("commandfor", "pair-17-glance"),
                ("aria-keyshortcuts", "O D"),
                ("data-fui", "side_panel keys"),
                ("hidden", "")
            ]
        ),
        "{keyed}"
    );
    let link = tags(&keyed, "a")
        .into_iter()
        .find(|link| link.get("href").map(String::as_str) == Some("/queue/pair-17"))
        .expect("the no-script link");
    assert!(
        !link.contains_key("aria-keyshortcuts"),
        "with no script the key is not announced: {keyed}"
    );
}

#[test]
fn an_opener_with_no_key_or_a_sequence_announces_none() {
    for drawn in [panel(None), panel(Some("g d"))] {
        assert!(!drawn.contains("aria-keyshortcuts"), "{drawn}");
        assert!(
            has(&drawn, "button", &[("data-fui", "side_panel")]),
            "{drawn}"
        );
    }
}
