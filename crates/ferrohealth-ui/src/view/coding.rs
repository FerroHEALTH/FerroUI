// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The coding (M2): a coded value as a display over its code and, where it
//! helps, its system; the inactive mark; and a canonical that breaks after
//! a slash.
//!
//! The code is drawn in the code face; the display may wrap. A code breaks
//! only when it is long (more than 24 characters, such as a versioned uid
//! or a UUID), and then only after a colon, a dot, a hyphen, a slash or a
//! vertical bar, so an ordinary code never breaks and a long one never
//! overflows a phone. A canonical breaks after each slash, and a long piece
//! between two slashes (a `urn:uuid:` or `urn:oid:` canonical, a long id)
//! breaks as a long code does. The inline form puts the code after the display on one
//! line, for a coding inside a sentence or a list of codes. An inactive
//! coding is muted and carries the hollow ring and the word Inactive, so
//! the state never rests on colour alone (family rule 12, WCAG 2.2
//! SC 1.4.1). No specification governs the layout: our own design.

use crate::html::Html;
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::view::path::{broken, segments};

/// The length in characters above which a code, or a piece of a canonical
/// between two slashes, offers its finer breaks.
const LONG: usize = 24;

/// The characters a long code or piece offers a break after.
const FINE: [char; 5] = [':', '.', '-', '/', '|'];

/// The pieces of `text`, each ending after one of `breaks`, and a piece of
/// more than [`LONG`] characters split again after each run of [`FINE`],
/// so `::` stays whole.
fn pieces(text: &str, breaks: &[char]) -> Vec<String> {
    segments(text, breaks)
        .into_iter()
        .flat_map(|piece| {
            if piece.chars().count() > LONG {
                fine(&piece)
            } else {
                vec![piece]
            }
        })
        .collect()
}

/// The pieces of `piece`, each ending after a run of [`FINE`] but the last.
fn fine(piece: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut characters = piece.chars().peekable();
    while let Some(character) = characters.next() {
        current.push(character);
        let ends_run =
            FINE.contains(&character) && characters.peek().is_some_and(|next| !FINE.contains(next));
        if ends_run {
            out.push(std::mem::take(&mut current));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// A coded value as a console has it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Coding {
    /// The display, or `None` for a code that has none, which says "no
    /// display" in the muted colour.
    pub display: Option<String>,
    /// The code.
    pub code: String,
    /// The system's name, such as "Synthetic clinical findings", when it
    /// helps the reader; `None` where the page already names it.
    pub system: Option<String>,
    /// Whether the code is inactive in its system.
    pub inactive: bool,
    /// The page of the concept, when the display leads to it.
    pub href: Option<String>,
    /// Whether the concept is the one the page reads, among codings of the
    /// same set such as a search's matches: its link carries
    /// `aria-current="true"` (WAI-ARIA 1.2).
    pub current: bool,
}

impl Coding {
    /// The active coding `code` shown as `display`, with no system and no
    /// link.
    #[must_use]
    pub fn new(code: impl Into<String>, display: impl Into<String>) -> Self {
        Self {
            display: Some(display.into()),
            code: code.into(),
            system: None,
            inactive: false,
            href: None,
            current: false,
        }
    }

    /// This coding, with its system named `system`.
    #[must_use]
    pub fn system(self, system: impl Into<String>) -> Self {
        Self {
            system: Some(system.into()),
            ..self
        }
    }

    /// This coding, inactive.
    #[must_use]
    pub fn inactive(self) -> Self {
        Self {
            inactive: true,
            ..self
        }
    }

    /// This coding, its display a link to `href`.
    #[must_use]
    pub fn href(self, href: impl Into<String>) -> Self {
        Self {
            href: Some(href.into()),
            ..self
        }
    }

    /// This coding, the current one of its set when `current` says so.
    #[must_use]
    pub fn current(self, current: bool) -> Self {
        Self { current, ..self }
    }
}

/// How a coding is laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    /// The display over its code and system: a table cell, a facts value.
    #[default]
    Stacked,
    /// The display, then its code, on one line that wraps between them: in
    /// a sentence or a list of codes.
    Inline,
}

/// Returns the inactive mark: the hollow ring and the word Inactive, in the
/// muted colour.
///
/// The ring is generated by the stylesheet with no text of its own, so a
/// screen reader reads the word alone.
#[must_use]
pub fn inactive_mark(locale: &Locale) -> Html {
    Html::raw(format!(
        "<span class=\"fui-inactive-mark\">{}</span>",
        tr(locale, Text::CodingInactive)
    ))
}

/// Returns `url` as a canonical: in the code face, breaking after each
/// slash, and inside a piece of more than 24 characters after each colon,
/// dot, hyphen or vertical bar.
#[must_use]
pub fn canonical(url: &str) -> Html {
    let parts: Vec<String> = pieces(url, &['/'])
        .into_iter()
        .map(|part| String::from(Html::text(part)))
        .collect();
    Html::raw(format!(
        "<code class=\"fui-canonical\">{}</code>",
        parts.join("<wbr>")
    ))
}

/// Returns the code `code` in the code face: whole, or for a code of more
/// than 24 characters a piece per break it offers.
fn code_face(code: &str) -> String {
    if code.chars().count() > LONG {
        format!(
            "<code class=\"fui-coding-code\">{}</code>",
            broken(pieces(code, &[]))
        )
    } else {
        format!(
            "<code class=\"fui-coding-code\">{}</code>",
            Html::text(code)
        )
    }
}

/// The row that ends a list of codes longer than a page, such as a
/// clause's codes or a coded node's value set: "25 of 4000 codes shown.
/// Show the next 25".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoreCodes {
    /// How many codes the list holds in all.
    pub total: u32,
    /// The sentence that counts the codes shown.
    pub words: Word,
    /// The link's words.
    pub link: Word,
    /// The page with the next codes read.
    pub href: String,
}

impl MoreCodes {
    /// The row of a list that shows `shown` of its `total` codes, reading
    /// `next` more at `href`, in `locale`.
    #[must_use]
    pub fn of(locale: &Locale, shown: u32, total: u32, next: u32, href: impl Into<String>) -> Self {
        let rest = total.saturating_sub(shown);
        let link = if next >= rest {
            tr_with(locale, Text::CodesMoreRest, &[("count", rest.into())])
        } else {
            tr_with(locale, Text::CodesMoreNext, &[("count", next.into())])
        };
        Self {
            total,
            words: tr_with(
                locale,
                Text::CodesMore,
                &[("shown", shown.into()), ("total", total.into())],
            ),
            link,
            href: href.into(),
        }
    }
}

/// Returns the last item of a list of codes longer than a page: how many
/// it shows and the link to the next ones, which the console's server
/// answers, so the list pages with no script.
#[must_use]
pub fn more_codes(more: MoreCodes) -> Html {
    let MoreCodes {
        words, link, href, ..
    } = more;
    Html::raw(format!(
        "<li class=\"fui-code-more\"><span>{words} <a href=\"{}\">{link}</a></span></li>",
        Html::text(href)
    ))
}

/// Returns the coding `coding` laid out as `layout`.
#[must_use]
pub fn coding(locale: &Locale, coding: Coding, layout: Layout) -> Html {
    let Coding {
        display,
        code,
        system,
        inactive,
        href,
        current,
    } = coding;
    let words = match display {
        Some(display) => String::from(Html::text(display)),
        None => format!(
            "<span class=\"fui-muted\">{}</span>",
            tr(locale, Text::CodingNoDisplay)
        ),
    };
    let shown = match href {
        Some(href) => format!(
            "<a href=\"{}\"{} class=\"fui-row-link\">{words}</a>",
            Html::text(href),
            if current {
                " aria-current=\"true\""
            } else {
                ""
            }
        ),
        None => words,
    };
    let system = system
        .map(|system| {
            format!(
                "<span class=\"fui-coding-system\">{}</span>",
                Html::text(system)
            )
        })
        .unwrap_or_default();
    let mark = if inactive {
        String::from(inactive_mark(locale))
    } else {
        String::new()
    };
    let mut class = String::from("fui-coding");
    if layout == Layout::Inline {
        class.push_str(" fui-coding-inline");
    }
    if inactive {
        class.push_str(" is-inactive");
    }
    Html::raw(format!(
        "<span class=\"{class}\"><span class=\"fui-coding-display\">{shown}</span><span class=\"fui-coding-meta\">{}{system}{mark}</span></span>",
        code_face(&code)
    ))
}

#[cfg(test)]
mod tests {
    use super::{Coding, Layout, pieces};

    #[test]
    fn the_builders_set_one_part_each() {
        let coding = Coding::new("SF-1210", "Cough")
            .system("Synthetic clinical findings")
            .inactive()
            .href("/concept")
            .current(true);
        assert_eq!(Some("Cough"), coding.display.as_deref());
        assert_eq!(
            Some("Synthetic clinical findings"),
            coding.system.as_deref()
        );
        assert!(coding.inactive);
        assert_eq!(Some("/concept"), coding.href.as_deref());
        assert!(coding.current);
        assert_eq!(Layout::Stacked, Layout::default());
    }

    #[test]
    fn only_a_long_code_or_piece_offers_its_finer_breaks() {
        assert_eq!(vec!["SF-1210"], pieces("SF-1210", &[]));
        assert_eq!(
            vec![
                "8849182c-",
                "82ad-",
                "4088-",
                "a07f-",
                "48ead4180515::",
                "ferroehr.",
                "example.",
                "org::",
                "12",
            ],
            pieces(
                "8849182c-82ad-4088-a07f-48ead4180515::ferroehr.example.org::12",
                &[]
            )
        );
        assert_eq!(
            vec!["https:/", "/", "example.org/", "fhir/", "ValueSet/", "a-b"],
            pieces("https://example.org/fhir/ValueSet/a-b", &['/'])
        );
        assert_eq!(
            vec![
                "urn:",
                "uuid:",
                "8849182c-",
                "82ad-",
                "4088-",
                "a07f-",
                "48ead4180515"
            ],
            pieces("urn:uuid:8849182c-82ad-4088-a07f-48ead4180515", &['/'])
        );
    }
}
