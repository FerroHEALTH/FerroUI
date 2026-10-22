// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's messages, one file per area.
//!
//! Each area declares its messages as [`Text`](crate::i18n::Text) constants in
//! `src/i18n/words/<area>.rs` and keeps their English in
//! `locales/en/<area>.ftl`, both in the order of the ids, so two changes to
//! different areas touch different files. The area is the component family
//! that draws the message. A catalog of another language holds the same
//! files in a directory named by its BCP 47 tag beside `locales/en/`.

/// Declares one area's messages: each constant names one message of the
/// catalog by its id, and the area's `ALL` lists them for `AREAS`.
macro_rules! texts {
    ($($(#[$meta:meta])+ $name:ident = $id:literal,)+) => {
        #[expect(
            non_upper_case_globals,
            reason = "a message is named as a word of the page, Text::Close, as when Text was an enum"
        )]
        impl Text {
            $($(#[$meta])+ pub(crate) const $name: Self = Self($id);)+
        }

        /// The area's messages, in the order of their ids.
        #[cfg(test)]
        pub(super) const ALL: &[Text] = &[$(Text::$name,)+];
    };
}

// The areas, one file each, kept in alphabetical order.
mod charts;
mod controls;
mod date;
mod documents;
mod error_page;
mod language;
mod preferences;
mod records;
mod refusals;
mod shell;
mod sign_in;
mod tables;
mod templates;
mod theme;

/// Every area by the name of its catalog file, with its messages.
#[cfg(test)]
pub(super) const AREAS: &[(&str, &[crate::i18n::Text])] = &[
    ("charts", charts::ALL),
    ("controls", controls::ALL),
    ("date", date::ALL),
    ("documents", documents::ALL),
    ("error_page", error_page::ALL),
    ("language", language::ALL),
    ("preferences", preferences::ALL),
    ("records", records::ALL),
    ("refusals", refusals::ALL),
    ("shell", shell::ALL),
    ("sign_in", sign_in::ALL),
    ("tables", tables::ALL),
    ("templates", templates::ALL),
    ("theme", theme::ALL),
];

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::AREAS;
    use crate::i18n::{Arg, CATALOG, Locale, fluent_args};

    /// The names of the placeables the library's messages take.
    const PLACEABLES: &[&str] = &[
        "accent", "action", "chart", "code", "count", "date", "day", "days", "document",
        "earliest", "file", "filter", "first", "from", "held", "key", "kinds", "label", "language",
        "last", "latest", "limit", "month", "note", "noun", "nouns", "page", "pages", "range",
        "shown", "size", "stated", "status", "summary", "tab", "time", "title", "to", "total",
        "value", "weekday", "what", "why", "words", "year",
    ];

    /// The ids of the messages the catalog file at `path` defines.
    fn ids(path: &Path) -> Vec<String> {
        let source = std::fs::read_to_string(path).expect("the catalog should be readable");
        let mut ids: Vec<String> = source
            .lines()
            .filter_map(|line| line.split_once(" ="))
            .map(|(id, _)| id)
            .filter(|id| {
                id.starts_with(|first: char| first.is_ascii_lowercase())
                    && id
                        .chars()
                        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
            })
            .map(str::to_owned)
            .collect();
        ids.sort();
        ids
    }

    /// The names of the files in the directory `path`, sorted.
    fn names(path: &Path) -> Vec<String> {
        let mut found: Vec<String> = std::fs::read_dir(path)
            .expect("the directory should be readable")
            .map(|entry| {
                entry
                    .expect("an entry should be readable")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        found.sort();
        found
    }

    #[test]
    fn the_shipped_languages_are_the_catalog_directories() {
        let found = names(&Path::new(env!("CARGO_MANIFEST_DIR")).join("locales"));
        assert_eq!(
            found,
            Locale::shipped(),
            "a catalog directory is a language"
        );
        assert!(
            found.contains(&String::from("en")),
            "English is the reference"
        );
    }

    #[test]
    fn every_area_declares_its_messages_in_the_order_of_their_ids() {
        for (area, texts) in AREAS {
            let declared: Vec<&str> = texts.iter().map(|text| text.id()).collect();
            let mut sorted = declared.clone();
            sorted.sort_unstable();
            assert_eq!(
                sorted, declared,
                "the area {area} is in the order of its ids"
            );
        }
        let mut every: Vec<&str> = AREAS
            .iter()
            .flat_map(|(_, texts)| texts.iter().map(|text| text.id()))
            .collect();
        let count = every.len();
        every.sort_unstable();
        every.dedup();
        assert_eq!(count, every.len(), "a message is declared in one area");
    }

    #[test]
    fn every_catalog_holds_every_message_and_no_other() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let mut files: Vec<String> = AREAS
            .iter()
            .map(|(area, _)| format!("{area}.ftl"))
            .collect();
        files.sort();
        for tag in Locale::shipped() {
            assert_eq!(
                files,
                names(&root.join(&tag)),
                "the catalog {tag} holds one file per area and no other"
            );
            for (area, texts) in AREAS {
                let mut declared: Vec<String> =
                    texts.iter().map(|text| text.id().to_owned()).collect();
                declared.sort();
                assert_eq!(
                    declared,
                    ids(&root.join(&tag).join(format!("{area}.ftl"))),
                    "the catalog {tag} holds the area {area} whole and nothing else"
                );
            }
            let language = tag.parse().expect("a catalog directory is a language");
            // Every placeable a message names, so each formats whole.
            let args: Vec<(&'static str, Arg)> = PLACEABLES
                .iter()
                .map(|name| (*name, Arg::from(3)))
                .collect();
            let args = fluent_args(&args);
            for text in AREAS.iter().flat_map(|(_, texts)| texts.iter()) {
                let words = CATALOG
                    .lookup_single_language(&language, text.id(), Some(&args))
                    .unwrap_or_else(|error| panic!("{tag} formats {}: {error}", text.id()));
                assert!(
                    !words.trim().is_empty(),
                    "{tag} draws {} as words",
                    text.id()
                );
            }
        }
    }
}
