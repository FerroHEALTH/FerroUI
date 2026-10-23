// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The words a console hands the components: made at run time from its own
//! catalog, a listing's noun chosen by count, an editor's title by its
//! mode, and the card-header action every console draws.

use ferrohealth_ui::i18n::Message;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::table::model::{Noun, OpenWindow, Paging, Window};
use ferrohealth_ui::table::pager::{Listing, link_open_pager, link_pager};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog};
use ferrohealth_ui::view::surface::{
    Crumb, Header, Mode, card_action, editor_heading, editor_title, titled_panel,
};
use ferrohealth_ui::widget::nav::{Entry, Group};
use leptos::prelude::*;

use crate::markup::has;

fluent_templates::static_loader! {
    static CONSOLE = {
        locales: "./tests/it/console",
        fallback_language: "en",
        customise: |bundle| bundle.set_use_isolating(false),
    };
}

/// The message `id` of the test's console catalog.
fn message(id: &'static str) -> Message {
    Message::new(&*CONSOLE, id)
}

/// `view` rendered to HTML in a page tree.
fn html(view: impl FnOnce() -> AnyView) -> String {
    Owner::new().with(|| view().to_html())
}

#[test]
fn a_listing_names_its_rows_by_the_consoles_noun_chosen_by_count() {
    let sets = Noun::new(message("value-sets"));
    let listing = || Listing::new(String::from("Value sets")).noun(sets);
    let page = html(|| {
        link_pager(listing(), Window::over(312, Paging::default()), &|_| {
            String::new()
        })
    });
    assert!(page.contains("1-25 of 312 value sets"), "{page}");
    assert!(page.contains(r#"aria-label="Value sets""#), "{page}");
    let one = html(|| {
        link_pager(listing(), Window::over(1, Paging::default()), &|_| {
            String::new()
        })
    });
    assert!(one.contains("1-1 of 1 value set<"), "{one}");
    let none = html(|| {
        link_pager(listing(), Window::over(0, Paging::default()), &|_| {
            String::new()
        })
    });
    assert!(
        none.contains("No value sets"),
        "the empty sentence is the console's: {none}"
    );
    let open = html(|| {
        link_open_pager(listing(), OpenWindow::of(Paging::default(), 25), &|_| {
            String::new()
        })
    });
    assert!(
        open.contains("Value sets 1 to 25"),
        "the opening form: {open}"
    );
}

#[test]
fn an_editor_says_new_when_it_creates_and_edit_when_it_edits() {
    let editor = message("file-editor");
    let list = || Crumb::new("Mapping files", "/files");
    let creating = html(|| {
        let header = Header {
            crumbs: vec![list()],
            mono: true,
            ..Header::default()
        };
        editor_heading(editor, &Mode::Creating, header, ActionGroup::none())
    });
    assert!(creating.contains("<h1>New mapping file</h1>"), "{creating}");
    assert!(
        has(&creating, "nav", &[("class", "fui-crumbs is-parent")]),
        "the trail ends with the list the new file joins: {creating}"
    );
    assert!(
        !creating.contains("aria-current"),
        "the trail never ends with the editor: {creating}"
    );
    let editing = Mode::Editing(String::from("vitals.model.yml"));
    let changing = html(|| {
        let header = Header {
            crumbs: vec![list(), Crumb::new("vitals.model.yml", "/files/vitals")],
            mono: true,
            ..Header::default()
        };
        editor_heading(editor, &editing, header, ActionGroup::none())
    });
    assert!(
        changing.contains("<h1>Edit</h1>"),
        "a sentence in the body face: {changing}"
    );
    assert!(
        changing.contains(r#"<li><a href="/files/vitals">"#)
            && !changing.contains("/ Edit")
            && !changing.contains("aria-current"),
        "the trail ends with the file it edits, never with Edit: {changing}"
    );
    let title = Owner::new().with(|| editor_title(editor, &Mode::Creating));
    assert_eq!(
        "New mapping file", title,
        "the document's title says the same"
    );
}

#[test]
fn the_card_header_action_is_a_link_with_its_words_then_an_arrow() {
    let panel = html(|| {
        titled_panel(
            "Members",
            card_action("/members", String::from("All members")),
            (),
        )
        .into_any()
    });
    assert!(
        panel
            .contains(r#"<a href="/members" class="fui-card-action"><span>All members</span><svg"#),
        "{panel}"
    );
    assert!(
        panel.contains(r#"aria-hidden="true""#),
        "the arrow is decoration: {panel}"
    );
    assert!(
        panel.contains("m12 5 7 7-7 7"),
        "the arrow follows the words: {panel}"
    );
}

#[test]
fn a_consoles_words_made_at_run_time_reach_every_label() {
    let groups = vec![Group {
        id: "nav-files",
        title: String::from("Files").into(),
        entries: vec![Entry {
            href: "/files".into(),
            label: format!("{} files", 3).into(),
            glyph: Glyph::FILES,
            exact: false,
            count: None,
        }],
    }];
    assert_eq!(
        "3 files", groups[0].entries[0].label,
        "a navigation entry's words"
    );
    let dialog = html(|| {
        confirm_dialog(
            Confirmation {
                id: "drop".into(),
                title: format!("Drop {}?", "vitals").into(),
                message: String::from("The file goes for good.").into(),
            },
            (),
        )
        .into_any()
    });
    assert!(dialog.contains("Drop vitals?"), "{dialog}");
}
