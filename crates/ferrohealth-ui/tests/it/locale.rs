// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The locale: a component draws its words from the catalog of the locale
//! the page provides, a throwaway catalog changes them with no code change,
//! and a message a catalog lacks falls back to English.

use ferrohealth_ui::i18n::{Locale, locale, provide_locale};
use ferrohealth_ui::table::model::{Paging, Window};
use ferrohealth_ui::table::view::link_pager;
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog};
use ferrohealth_ui::view::theme_switch::locale_switch;
use leptos::prelude::*;

fluent_templates::static_loader! {
    static THROWAWAY = {
        locales: "./tests/it/locales",
        fallback_language: "qaa",
        customise: |bundle| bundle.set_use_isolating(false),
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
                id: "drop",
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
fn the_language_switch_names_each_choice_in_its_own_language() {
    let switch = render(Some(throwaway()), || {
        locale_switch(
            &[Locale::english(), throwaway()],
            String::from("/locale"),
            String::from("/"),
        )
    });
    assert!(switch.contains(r#"aria-label="Language""#), "{switch}");
    assert!(
        switch.contains(r#"value="en" lang="en" aria-pressed="false""#),
        "{switch}"
    );
    assert!(switch.contains(">English<"), "{switch}");
    assert!(
        switch.contains(r#"value="qaa" lang="qaa" aria-pressed="true""#),
        "the page's locale is pressed: {switch}"
    );
    assert!(switch.contains(">Qaa<"), "{switch}");
}

#[test]
fn the_footer_draws_its_version_as_data_in_every_locale() {
    use ferrohealth_ui::view::shell::{Footer, footer};
    let text = Footer {
        product: "FerroUI Gallery".into(),
        version: "0.1.8",
        about: "/about",
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
