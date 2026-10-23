// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The locale: a component draws its words from the catalog of the locale
//! the page provides, a throwaway catalog changes them with no code change,
//! and a message a catalog lacks falls back to English.

use ferrohealth_ui::i18n::number::grouped;
use ferrohealth_ui::i18n::{Locale, locale, provide_locale};
use ferrohealth_ui::island::CopyButton;
use ferrohealth_ui::table::model::{Paging, Window};
use ferrohealth_ui::table::pager::link_pager;
use ferrohealth_ui::theme::Language;
use ferrohealth_ui::view::diagnostics::Position;
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog};
use ferrohealth_ui::view::error_page::Status;
use ferrohealth_ui::view::kbd::{Shortcut, shortcut_hint};
use ferrohealth_ui::view::preferences::{Post, language_select};
use ferrohealth_ui::view::refusal::{Refusal, refusal};
use ferrohealth_ui::view::template::list::{HeadingCount, counted};
use ferrohealth_ui::widget::copy::{CopySource, CopyWords};
use ferrohealth_ui::widget::editor::{Editor, EditorWords, Opened, Panel, editor_panel};
use ferrohealth_ui::widget::identifier::{
    Domain, Identifier, IdentifierWords, Masked, Reveal, Revealed, identifier,
};
use ferrohealth_ui::widget::upload::SizeWords;
use leptos::prelude::*;

use crate::markup::has;

fluent_templates::static_loader! {
    static THROWAWAY = {
        locales: "./tests/it/locales",
        fallback_language: "qaa",
        customise: |bundle| {
            bundle.set_use_isolating(false);
            bundle.set_formatter(Some(ferrohealth_ui::i18n::number::format_number));
        },
    };
}

/// The throwaway locale, its catalog the test's own.
fn throwaway() -> Locale {
    Locale::parse("qaa").unwrap().with_catalog(&*THROWAWAY)
}

/// `view` rendered to HTML in a page that provides `locale`, or none.
fn render(locale: Option<Locale>, view: impl FnOnce() -> AnyView) -> String {
    Owner::new().with(|| {
        if let Some(locale) = locale {
            provide_locale(locale);
        }
        view().to_html()
    })
}

/// A pager of 30 rows on its first page, every step drawn.
fn pager() -> AnyView {
    link_pager(
        "Pages",
        Window::over(30, Paging::default()),
        &|paging: Paging| format!("/files?page={}", paging.page),
    )
}

#[test]
fn a_throwaway_catalog_changes_a_components_words_with_no_code_change() {
    let english = render(Some(Locale::english()), pager);
    for words in [
        "First",
        "Next",
        "Rows per page",
        "Page 1 of 2",
        "1-25 of 30 rows",
    ] {
        assert!(english.contains(words), "{words}: {english}");
    }
    let throwaway = render(Some(throwaway()), pager);
    for words in [
        "Qirst",
        "Qext",
        "Qows per qage",
        "Qage 1 qof 2",
        "1-25 qof 30 qows",
    ] {
        assert!(throwaway.contains(words), "{words}: {throwaway}");
    }
    for words in [">First<", ">Next<", "Rows per page", "Page 1 of 2"] {
        assert!(!throwaway.contains(words), "{words}: {throwaway}");
    }
    assert!(
        throwaway.contains("(unavailable)"),
        "a message the catalog lacks falls back to English: {throwaway}"
    );
}

#[test]
fn a_page_that_provides_no_locale_draws_english() {
    let none = render(None, pager);
    assert_eq!(render(Some(Locale::english()), pager), none);
    let dialog = || {
        confirm_dialog(
            Confirmation {
                id: "drop".into(),
                title: "Drop the query?".into(),
                message: "The stored query is removed.".into(),
            },
            (),
        )
        .into_any()
    };
    assert!(render(None, dialog).contains("Cancel"));
    assert!(render(Some(throwaway()), dialog).contains("Qancel"));
}

#[test]
fn the_page_reads_the_locale_it_provided() {
    let tag = Owner::new().with(|| {
        provide_locale(throwaway());
        locale().tag()
    });
    assert_eq!("qaa", tag, "a console writes it as <html lang>");
    assert_eq!("en", Owner::new().with(|| locale().tag()));
}

#[test]
fn the_language_select_names_each_choice_in_its_own_language() {
    let post = Post {
        action: String::from("/preference"),
        back: String::from("/preferences/language"),
    };
    let select = Owner::new_root(Some(std::sync::Arc::new(
        hydration_context::SsrSharedContext::new(),
    )))
    .with(|| {
        provide_locale(throwaway());
        language_select(
            "language-list",
            &Language::Chosen(String::from("qaa")),
            &[Locale::english(), throwaway()],
            &Locale::english(),
            &post,
        )
        .to_html()
    });
    assert!(
        has(
            &select,
            "button",
            &[("lang", "en"), ("aria-selected", "false")]
        ),
        "{select}"
    );
    assert!(select.contains(">English<"), "{select}");
    assert!(
        has(
            &select,
            "button",
            &[("lang", "qaa"), ("aria-selected", "true")]
        ),
        "the page's language is chosen: {select}"
    );
    assert!(
        select.contains(">Qaa<"),
        "each language in its own name: {select}"
    );
    assert!(
        has(
            &select,
            "input",
            &[("name", "language"), ("value", "browser")]
        ),
        "Follow the browser posts the browser's key: {select}"
    );
    assert!(
        has(&select, "input", &[("name", "language"), ("value", "qaa")]),
        "{select}"
    );
    assert!(
        has(
            &select,
            "input",
            &[("name", "back"), ("value", "/preferences/language")]
        ),
        "{select}"
    );
    assert!(
        select.contains("English, from your browser"),
        "the browser's language under its choice: {select}"
    );
}

#[test]
fn the_footer_draws_its_version_as_data_in_every_locale() {
    use ferrohealth_ui::view::shell::{Footer, footer};
    let text = Footer {
        product: "FerroUI Gallery".into(),
        version: "0.1.8".into(),
        about: "/about".into(),
    };
    for locale in [Locale::english(), throwaway()] {
        let page = render(Some(locale), || footer(text.clone()).into_any());
        assert!(page.contains("FerroUI Gallery"), "{page}");
        assert!(
            page.contains("FerroUI Gallery 0.1.8 · "),
            "the About page's form, the product then its version: {page}"
        );
        assert!(
            !page.contains("[0.1.8"),
            "the version never goes through a catalog: {page}"
        );
    }
}

/// A pager over 312,000 rows on page 6,000.
fn large_pager() -> AnyView {
    link_pager(
        "Pages",
        Window::over(312_000, Paging::read(Some("6000"), Some("50"))),
        &|paging: Paging| format!("/files?page={}", paging.page),
    )
}

#[test]
fn a_number_a_message_takes_is_grouped_in_the_marks_of_the_page_language() {
    let english = render(Some(Locale::english()), large_pager);
    assert!(
        english.contains("299,951-300,000 of 312,000 rows"),
        "{english}"
    );
    assert!(english.contains("Page 6,000 of 6,240"), "{english}");
    let throwaway = render(Some(throwaway()), large_pager);
    assert!(
        throwaway.contains("299.951-300.000 qof 312.000 qows"),
        "a catalog brings its own marks with no code change: {throwaway}"
    );
}

#[test]
fn a_count_drawn_outside_a_message_is_grouped_in_the_page_language() {
    let count = || counted(String::from("Rows"), HeadingCount::All(4_000_000_000)).into_any();
    let english = render(Some(Locale::english()), count);
    assert!(english.contains("· 4,000,000,000"), "{english}");
    let throwaway = render(Some(throwaway()), count);
    assert!(throwaway.contains("· 4.000.000.000"), "{throwaway}");
    assert_eq!("312,000", render(None, || grouped(312_000).into_any()));
}

/// `view` rendered to HTML in a page tree with the server's shared context,
/// as a page draws an island, in a page that provides `locale`.
fn page(locale: Locale, view: impl FnOnce() -> AnyView) -> String {
    Owner::new_root(Some(std::sync::Arc::new(
        hydration_context::SsrSharedContext::new(),
    )))
    .with(|| {
        provide_locale(locale);
        view().to_html()
    })
}

/// The code editor over `lines` lines, with no mark.
fn editor_of(lines: usize) -> AnyView {
    editor_panel(
        Panel {
            editor: Editor {
                id: "draft",
                language: ferrohealth_ui::highlight::Language::Yaml,
                text: Signal::stored("a\n".repeat(lines)),
                marks: Signal::stored(Vec::new()),
                editable: Signal::stored(false),
            },
            label: String::from("The draft"),
            lead: String::from("YAML"),
            facts: Vec::new(),
            opened: Opened::New,
            caret: RwSignal::new(Position::default()),
            tools: None,
            end: None,
            words: EditorWords::of_page(),
        },
        |_| {},
    )
}

/// The masked identifier drawn shown, `seconds` left on its countdown.
fn identifier_shown(seconds: u32) -> AnyView {
    identifier(
        Identifier {
            id: String::from("id-1"),
            masked: Masked::new(Domain::new("MRN"), "SYN-7Q2-XYZ", 3),
            show_domain: true,
            state: RwSignal::new(Reveal::Shown(Revealed::new(
                String::from("SYN-7Q2-XYZ"),
                seconds,
            ))),
            fallback: String::from("/reveal/1"),
            words: IdentifierWords::of_page(),
        },
        Callback::new(|()| {}),
    )
}

#[test]
fn an_island_groups_the_counts_it_learns_in_the_browser() {
    let english = render(Some(Locale::english()), || editor_of(1234));
    assert!(english.contains("YAML · 1,234 lines"), "{english}");
    let qaa = render(Some(throwaway()), || editor_of(1234));
    assert!(
        qaa.contains("1.234 lines"),
        "the line count is grouped by the page's mark: {qaa}"
    );
    let size = |locale| {
        Owner::new().with(|| {
            provide_locale(locale);
            String::from(SizeWords::of_page().size(1023))
        })
    };
    assert_eq!("1,023 bytes", size(Locale::english()));
    assert_eq!(
        "1.023 bytes",
        size(throwaway()),
        "the upload field's size is grouped by the page's mark"
    );
    let shown = render(Some(throwaway()), || identifier_shown(1200));
    assert!(
        shown.contains("1.200 s left"),
        "the identifier's countdown takes the same path: {shown}"
    );
    let words = Owner::new().with(|| {
        provide_locale(throwaway());
        serde_json::to_string(&EditorWords::of_page()).unwrap()
    });
    assert!(
        words.contains(r#""grouping":{"group":"."}"#),
        "the mark reaches the island with its words: {words}"
    );
}

#[test]
fn a_library_word_that_falls_back_to_english_is_marked_english() {
    let shortcuts = || {
        shortcut_hint(&[Shortcut {
            keys: String::from("j | down"),
            does: String::from("Next row"),
        }])
    };
    let copy = || {
        view! {
            <CopyButton
                id=String::from("copy")
                label=ferrohealth_ui::i18n::Word::from("Copy")
                source=CopySource::Text(String::from("GET /Patient"))
                words=CopyWords::of_page()
            />
        }
        .into_any()
    };
    let post = Post {
        action: String::from("/preference"),
        back: String::from("/preferences/language"),
    };
    let language = || {
        language_select(
            "language-list",
            &Language::Browser,
            &[Locale::english()],
            &Locale::english(),
            &post,
        )
    };
    let refused = || {
        refusal(&Refusal {
            status: Some(422),
            summary: String::from("The mapping did not validate."),
            issues: Vec::new(),
            request: None,
            evidence: None,
        })
        .into_any()
    };
    let families: [(&str, &dyn Fn() -> AnyView, &str); 8] = [
        ("tables", &pager, "(unavailable)"),
        ("keys", &shortcuts, "or"),
        ("islands", &copy, "Copied to the clipboard."),
        ("preferences", &language, "Follow the browser"),
        ("refusals", &refused, "422"),
        ("editor", &|| editor_of(3), "3 lines"),
        ("records", &|| identifier_shown(30), "Hide"),
        (
            "error pages",
            &|| Status::NotFound.line().into_render(),
            "Not found",
        ),
    ];
    for (family, view, word) in families {
        let english = page(Locale::english(), view);
        assert!(english.contains(word), "{family}: {english}");
        assert!(
            !english.contains(r#"<span lang="en">"#) && !english.contains("&quot;en&quot;:"),
            "{family}: an English page marks no word: {english}"
        );
        let qaa = page(throwaway(), view);
        assert!(
            qaa.contains(r#"<span lang="en">"#) || qaa.contains("&quot;en&quot;:"),
            "{family}: a word the page's catalog lacks is marked English: {qaa}"
        );
    }
    let found = page(throwaway(), pager);
    assert!(
        found.contains(">Qirst<") && !found.contains(r#"<span lang="en">Qirst"#),
        "a word the page's catalog holds is left as it is: {found}"
    );
}
