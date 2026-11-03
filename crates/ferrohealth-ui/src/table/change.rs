// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The change table: a row per setting with its value before and after,
//! as a plan shows the difference between a running and a staged
//! configuration.
//!
//! A console that compares something else (files, fields) names the key
//! column itself. A history row says its change in prose instead
//! ([`plain_change`]): the value before struck through, then the value
//! after.
//!
//! The value removed is marked "−" and the value added "+", in words as
//! well as tone, and an absent value reads "not set", so a change never
//! rides on colour alone.

use crate::html::Html;
use crate::i18n::{Locale, Text, tr};
use crate::table::view::{Body, Column, DataTable, data_table};

/// One setting's change.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// The setting's name or path.
    pub key: String,
    /// Its value before, `None` when it was not set.
    pub from: Option<String>,
    /// Its value after, `None` when it is removed.
    pub to: Option<String>,
}

/// The columns of a change table, the key column headed `key` and the
/// others named in the page's locale.
fn columns(locale: &Locale, key: String) -> [Column; 3] {
    [
        Column::owned("key", key),
        Column::owned("from", tr(locale, Text::ChangeBefore)),
        Column::owned("to", tr(locale, Text::ChangeAfter)),
    ]
}

/// Returns one side of a change: `value` marked with `sign` in the removed
/// or the added tone, or "not set".
fn side(locale: &Locale, value: Option<&String>, sign: &'static str, removed: bool) -> String {
    let class = if removed {
        "fui-change fui-change-removed"
    } else {
        "fui-change fui-change-added"
    };
    match value {
        Some(value) => {
            let said = tr(
                locale,
                if removed {
                    Text::ChangeRemoved
                } else {
                    Text::ChangeAdded
                },
            )
            .map(|words| format!("{words} "));
            format!(
                "<span class=\"{class}\"><span aria-hidden=\"true\" class=\"fui-change-sign\">{sign}</span><span class=\"fui-sr-only\">{said}</span><code>{}</code></span>",
                Html::text(value)
            )
        }
        None => format!(
            "<span class=\"fui-muted\">{}</span>",
            tr(locale, Text::ChangeNotSet)
        ),
    }
}

/// Returns the table of `changes` captioned `caption`.
///
/// Its key column is headed `key` from the console's catalog (a file, a
/// path, a field) or, with `None`, by the library's word for a setting. An
/// empty list reads as a sentence.
#[must_use]
pub fn change_table(
    locale: &Locale,
    caption: impl Into<String>,
    key: Option<String>,
    changes: &[Change],
) -> Html {
    let caption = caption.into();
    if changes.is_empty() {
        return Html::raw(format!(
            "<p class=\"fui-muted\">{}</p>",
            tr(locale, Text::ChangeNone)
        ));
    }
    let before = Html::text(tr(locale, Text::ChangeBefore).text());
    let after = Html::text(tr(locale, Text::ChangeAfter).text());
    let rows: Html = changes
        .iter()
        .map(|change| {
            Html::raw(format!(
                "<tr><th scope=\"row\"><code>{}</code></th><td data-label=\"{before}\">{}</td><td data-label=\"{after}\">{}</td></tr>",
                Html::text(&change.key),
                side(locale, change.from.as_ref(), "\u{2212}", true),
                side(locale, change.to.as_ref(), "+", false)
            ))
        })
        .collect();
    data_table(DataTable::new(
        &caption,
        &columns(
            locale,
            key.unwrap_or_else(|| tr(locale, Text::ChangeSetting).text().to_owned()),
        ),
        Body::Rows(rows),
    ))
}

/// Returns the change from `before` to `after` in prose, as a history row
/// says it.
///
/// The value before is a `del`, struck through after "−" and the word
/// "was", and the value after an `ins` after "+" and "now". A side that is
/// `None` is left out.
#[must_use]
pub fn plain_change(locale: &Locale, before: Option<&str>, after: Option<&str>) -> Html {
    let was = before.map(|value| {
        format!(
            "<del class=\"fui-change fui-change-removed fui-change-plain\"><span class=\"fui-change-sign\" aria-hidden=\"true\">\u{2212}</span><span class=\"fui-sr-only\">{} </span><span class=\"fui-change-text\">{}</span></del>",
            tr(locale, Text::ChangeWas),
            Html::text(value)
        )
    });
    let now = after.map(|value| {
        format!(
            "<ins class=\"fui-change fui-change-added fui-change-plain\"><span class=\"fui-change-sign\" aria-hidden=\"true\">+</span><span class=\"fui-sr-only\">{} </span>{}</ins>",
            tr(locale, Text::ChangeNow),
            Html::text(value)
        )
    });
    Html::raw(
        [was, now]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(" "),
    )
}

#[cfg(test)]
mod tests {
    use super::plain_change;
    use crate::i18n::Locale;

    #[test]
    fn a_plain_change_leaves_out_the_side_it_does_not_have() {
        let locale = Locale::default();
        let both = String::from(plain_change(&locale, Some("a"), Some("b")));
        assert!(both.contains("</del> <ins "), "{both}");
        let removed = String::from(plain_change(&locale, Some("a"), None));
        assert!(
            removed.starts_with("<del ") && !removed.contains("<ins"),
            "{removed}"
        );
        let added = String::from(plain_change(&locale, None, Some("b")));
        assert!(
            added.starts_with("<ins ") && !added.contains("<del"),
            "{added}"
        );
        assert_eq!("", String::from(plain_change(&locale, None, None)));
    }
}
