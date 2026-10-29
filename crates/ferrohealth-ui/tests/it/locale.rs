// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The locale: a component draws its words from the catalog of the locale
//! the console hands it, a throwaway catalog changes them with no code
//! change, and a message a catalog lacks falls back to English.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Locale;
use ferrohealth_ui::i18n::number::grouped;
use ferrohealth_ui::table::model::{Paging, Window};
use ferrohealth_ui::table::pager::link_pager;
use ferrohealth_ui::theme::Language;
use ferrohealth_ui::view::copy::CopyWords;
use ferrohealth_ui::view::diagnostics::Position;
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog};
use ferrohealth_ui::view::editor::{Editor, EditorWords, Opened, Panel, editor_panel};
use ferrohealth_ui::view::error_page::Status;
use ferrohealth_ui::view::identifier::{
    Domain, Identifier, IdentifierWords, Masked, Reveal, Revealed, identifier,
};
use ferrohealth_ui::view::kbd::{Shortcut, shortcut_hint};
use ferrohealth_ui::view::preferences::{Post, language_select};
use ferrohealth_ui::view::refusal::{Refusal, refusal};
use ferrohealth_ui::view::template::list::{HeadingCount, counted};
use ferrohealth_ui::view::upload::SizeWords;

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

/// `view` rendered to HTML in `locale`, or in the default locale for
/// none, as a console that chooses no locale of its own.
fn render(locale: Option<Locale>, view: impl FnOnce(&Locale) -> Html) -> String {
    let locale = locale.unwrap_or_default();
    view(&locale).into()
}

/// A pager of 30 rows on its first page, every step drawn.
fn pager(locale: &Locale) -> Html {
    link_pager(
        locale,
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
    let dialog = |locale: &Locale| {
        confirm_dialog(
            locale,
            Confirmation {
                id: "drop".into(),
                title: "Drop the query?".into(),
                message: "The stored query is removed.".into(),
            },
            Html::default(),
        )
    };
    assert!(render(None, dialog).contains("Cancel"));
    assert!(render(Some(throwaway()), dialog).contains("Qancel"));
}

#[test]
fn the_page_reads_the_locale_it_provided() {
    assert_eq!(
        "qaa",
        throwaway().tag(),
        "a console writes it as <html lang>"
    );
    assert_eq!("en", Locale::default().tag());
}

#[test]
fn the_language_select_names_each_choice_in_its_own_language() {
    let post = Post {
        action: String::from("/preference"),
        back: String::from("/preferences/language"),
    };
    let select: String = language_select(
        &throwaway(),
        "language-list",
        &Language::Chosen(String::from("qaa")),
        &[Locale::english(), throwaway()],
        &Locale::english(),
        &post,
    )
    .into();
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
        let page = render(Some(locale), |locale| footer(locale, text.clone()));
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
fn large_pager(locale: &Locale) -> Html {
    link_pager(
        locale,
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
    let count = |locale: &Locale| {
        counted(
            locale,
            String::from("Rows"),
            HeadingCount::All(4_000_000_000),
        )
    };
    let english = render(Some(Locale::english()), count);
    assert!(english.contains("· 4,000,000,000"), "{english}");
    let throwaway = render(Some(throwaway()), count);
    assert!(throwaway.contains("· 4.000.000.000"), "{throwaway}");
    assert_eq!("312,000", grouped(&Locale::default(), 312_000));
}

/// The code editor over `lines` lines, with no mark, in `locale`.
fn editor_of(locale: &Locale, lines: usize) -> Html {
    editor_panel(Panel {
        editor: Editor {
            id: "draft".into(),
            language: ferrohealth_ui::highlight::Language::Yaml,
            text: "a\n".repeat(lines),
            marks: Vec::new(),
            editable: false,
        },
        label: String::from("The draft"),
        lead: String::from("YAML"),
        facts: Vec::new(),
        opened: Opened::New,
        caret: Position::default(),
        tools: None,
        end: None,
        words: EditorWords::of(locale),
    })
}

/// The masked identifier drawn shown, `seconds` left on its countdown, in
/// `locale`.
fn identifier_shown(locale: &Locale, seconds: u32) -> Html {
    identifier(Identifier {
        id: String::from("id-1"),
        masked: Masked::new(Domain::new("MRN"), "SYN-7Q2-XYZ", 3),
        show_domain: true,
        state: Reveal::Shown(Revealed::new(String::from("SYN-7Q2-XYZ"), seconds)),
        fallback: String::from("/reveal/1"),
        words: IdentifierWords::of(locale),
    })
}

#[test]
fn a_script_module_groups_the_counts_it_learns_in_the_browser() {
    let english = render(Some(Locale::english()), |locale| editor_of(locale, 1234));
    assert!(english.contains("YAML · 1,234 lines"), "{english}");
    let qaa = render(Some(throwaway()), |locale| editor_of(locale, 1234));
    assert!(
        qaa.contains("1.234 lines"),
        "the line count is grouped by the page's mark: {qaa}"
    );
    let size = |locale: Locale| String::from(SizeWords::of(&locale).size(1023));
    assert_eq!("1,023 bytes", size(Locale::english()));
    assert_eq!(
        "1.023 bytes",
        size(throwaway()),
        "the upload field's size is grouped by the page's mark"
    );
    let shown = render(Some(throwaway()), |locale| identifier_shown(locale, 1200));
    assert!(
        shown.contains("1.200 s left"),
        "the identifier's countdown takes the same path: {shown}"
    );
    let words = serde_json::to_string(&EditorWords::of(&throwaway())).unwrap();
    assert!(
        words.contains(r#""grouping":{"group":"."}"#),
        "the mark reaches the module with its words: {words}"
    );
}

/// A view drawn in the locale it is given.
type View<'a> = dyn Fn(&Locale) -> Html + 'a;

#[test]
fn a_library_word_that_falls_back_to_english_is_marked_english() {
    let shortcuts = |locale: &Locale| {
        shortcut_hint(
            locale,
            &[Shortcut {
                keys: String::from("j | down"),
                does: String::from("Next row"),
            }],
        )
    };
    let copied = |locale: &Locale| {
        let words = CopyWords::of(locale);
        Html::raw(format!("<p>{}</p>", words.said_copied))
    };
    let post = Post {
        action: String::from("/preference"),
        back: String::from("/preferences/language"),
    };
    let language = |locale: &Locale| {
        language_select(
            locale,
            "language-list",
            &Language::Browser,
            &[Locale::english()],
            &Locale::english(),
            &post,
        )
    };
    let refused = |locale: &Locale| {
        refusal(
            locale,
            &Refusal {
                status: Some(422),
                summary: String::from("The mapping did not validate."),
                issues: Vec::new(),
                request: None,
                evidence: None,
            },
        )
    };
    let families: [(&str, &View<'_>, &str); 8] = [
        ("tables", &pager, "(unavailable)"),
        ("keys", &shortcuts, "or"),
        ("copy", &copied, "Copied to the clipboard."),
        ("preferences", &language, "Follow the browser"),
        ("refusals", &refused, "422"),
        ("editor", &|locale| editor_of(locale, 3), "3 lines"),
        ("records", &|locale| identifier_shown(locale, 30), "Hide"),
        (
            "error pages",
            &|locale| Status::NotFound.line(locale).into(),
            "Not found",
        ),
    ];
    for (family, view, word) in families {
        let english = render(Some(Locale::english()), view);
        assert!(english.contains(word), "{family}: {english}");
        assert!(
            !english.contains(r#"<span lang="en">"#) && !english.contains("&#34;en&#34;:"),
            "{family}: an English page marks no word: {english}"
        );
        let qaa = render(Some(throwaway()), view);
        assert!(
            qaa.contains(r#"<span lang="en">"#) || qaa.contains("&#34;en&#34;:"),
            "{family}: a word the page's catalog lacks is marked English: {qaa}"
        );
    }
    let found = render(Some(throwaway()), pager);
    assert!(
        found.contains(">Qirst<") && !found.contains(r#"<span lang="en">Qirst"#),
        "a word the page's catalog holds is left as it is: {found}"
    );
}
