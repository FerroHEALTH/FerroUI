// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! An archetype id and an archetype path in mono, each breaking only where a
//! reader expects it (E5, `design/COMPONENTS.md`).
//!
//! An archetype id or an ADL 2 HRID (`fui-archetype`) breaks only after a
//! dot, and a path (`fui-path`) only after a slash, a dot or an opening
//! bracket: each segment is a `<span>` that never wraps, with a `<wbr>`
//! after each break. A path scrolls sideways when one segment is wider than
//! its box. No specification governs the breaks: our own design.

use crate::html::Html;

/// The segments of `text`, each ending after one of `breaks` but the last.
pub(crate) fn segments(text: &str, breaks: &[char]) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    for character in text.chars() {
        current.push(character);
        if breaks.contains(&character) {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// The segments `parts`, each a `<span>` with a break offered after it.
pub(crate) fn broken(parts: Vec<String>) -> Html {
    parts
        .into_iter()
        .map(|part| Html::raw(format!("<span>{}</span><wbr>", Html::text(part))))
        .collect()
}

/// Returns the archetype id or HRID `id` in mono, breaking only after a
/// dot, the whole id in its tooltip.
#[must_use]
pub fn archetype(id: &str) -> Html {
    Html::raw(format!(
        "<code class=\"fui-archetype\" title=\"{}\">{}</code>",
        Html::text(id),
        broken(segments(id, &['.']))
    ))
}

/// Returns the archetype path `path` in mono, breaking only after a slash,
/// a dot or an opening bracket.
#[must_use]
pub fn path(path: &str) -> Html {
    Html::raw(format!(
        "<span class=\"fui-path\">{}</span>",
        broken(segments(path, &['/', '.', '[']))
    ))
}

#[cfg(test)]
mod tests {
    use super::segments;

    #[test]
    fn an_id_breaks_after_each_dot_and_a_path_after_each_slash_dot_or_bracket() {
        assert_eq!(
            vec!["openEHR-EHR-OBSERVATION.", "blood_pressure.", "v2"],
            segments("openEHR-EHR-OBSERVATION.blood_pressure.v2", &['.'])
        );
        assert_eq!(
            vec!["/", "content[", "id0.", "1]/", "value"],
            segments("/content[id0.1]/value", &['/', '.', '['])
        );
        assert_eq!(Vec::<String>::new(), segments("", &['.']));
    }
}
