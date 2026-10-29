// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The masked identifier: its domain, a mask with the last characters left
//! visible, and an audited reveal with a countdown (R13).
//!
//! An identifier belongs to any national or local system: the domain is
//! data ([`Domain`], "NHS number", "NIR", "MRN · Hospital X"), and nothing
//! here assumes a length, a character set, a check digit or a format. The
//! server masks the value with [`Masked::new`], so a page carries only the
//! domain and the few characters [`KEEP`] leaves visible, never the whole
//! value: the mask always draws [`DOTS`] dots whatever the length, so it
//! never tells the length either, and a short value is never shown whole.
//!
//! [`identifier`] is the interactive form. Reveal is a link to the
//! console's own reveal page, which asks the reason, makes the audited
//! round trip and draws the page again with the value in [`Reveal::Shown`];
//! the `identifier` module then counts the seconds down and drops the value
//! from the page when they run out or Hide is pressed, so the value stays
//! in the page no longer than the countdown. The value is a [`Revealed`],
//! which never serializes and whose `Debug` withholds it, so it reaches no
//! log. The countdown speaks through a polite live region at
//! [`ANNOUNCE_AT`] seconds and at zero (WAI-ARIA 1.2, `status`), Hide is a
//! toggle with `aria-pressed`, and the faces (masked, revealed, busy)
//! share one reserved slot, so revealing, counting down and hiding move
//! nothing around it. When the value is dropped with the keyboard focus in
//! it, the focus moves to Reveal, so it is never dropped to the page (WCAG
//! 2.2 SC 2.4.3). [`identifier_static`] is the listing form, with no
//! control.
//!
//! No specification governs the component: our own design.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::number::Grouping;
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::copy::{CopySource, CopyWords, copy_from};
use crate::view::tag::Tag;

/// How many characters at the end of a value the mask leaves visible by
/// default.
pub const KEEP: usize = 3;

/// How many dots the mask draws, whatever the value's length.
pub const DOTS: usize = 6;

/// The seconds left at which the live region says how many remain.
pub const ANNOUNCE_AT: u32 = 10;

/// The dot the mask draws for each hidden character.
const DOT: char = '\u{2022}';

/// An identifier system: what the identifier is, as a reader names it.
///
/// `label` is the domain's short name in the console's words ("BSN", "NHS
/// number", "Personnummer", "MRN · Hospital X"); `system` is the system's
/// own URI or OID, which the domain tag carries as its tooltip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain {
    /// The domain's short name.
    pub label: String,
    /// The system's URI or OID, when the console knows it.
    pub system: Option<String>,
}

impl Domain {
    /// The domain `label`, with no system.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            system: None,
        }
    }

    /// This domain, its system `system`.
    #[must_use]
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Returns the domain as a tag, its system as the tooltip.
    fn tag(&self) -> Html {
        let tag = Tag::new(self.label.clone());
        match &self.system {
            Some(system) => tag.title(system.clone()).view(),
            None => tag.view(),
        }
    }
}

/// The part of an identifier a page may show: its domain and the
/// characters the mask leaves visible.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Masked {
    /// The identifier's domain.
    pub domain: Domain,
    /// The characters the mask leaves visible.
    pub tail: String,
}

impl Masked {
    /// Masks `value` of `domain` on the server, leaving at most `keep`
    /// characters of its end visible ([`KEEP`] is the default).
    ///
    /// The visible part is clamped to half the value's characters, rounded
    /// down, so a short value is never shown whole, and loses any leading
    /// separator ("-CD" shows as "CD"). Any characters count: letters,
    /// digits, separators, any script.
    #[must_use]
    pub fn new(domain: Domain, value: &str, keep: usize) -> Self {
        let count = value.chars().count();
        let shown = kept(count, keep);
        let tail: String = value.chars().skip(count - shown).collect();
        Self {
            domain,
            tail: tail
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .to_owned(),
        }
    }

    /// The mask as it is drawn: [`DOTS`] dots and the visible tail.
    #[must_use]
    pub fn drawn(&self) -> String {
        let mut drawn: String = std::iter::repeat_n(DOT, DOTS).collect();
        drawn.push_str(&self.tail);
        drawn
    }
}

/// How many of `count` characters a mask asked to keep `keep` leaves
/// visible: never more than half, so a short value is never shown whole.
#[must_use]
#[expect(
    clippy::integer_division,
    reason = "half of the characters, rounded down, is the rule"
)]
pub const fn kept(count: usize, keep: usize) -> usize {
    let half = count / 2;
    if keep < half { keep } else { half }
}

/// A value the console's server revealed, held only while it is shown.
///
/// It never serializes and its `Debug` withholds the value, so it reaches
/// no log and no error message.
#[derive(Clone, PartialEq, Eq)]
pub struct Revealed {
    /// The whole value.
    value: String,
    /// The seconds it stays shown.
    seconds: u32,
}

impl Revealed {
    /// The revealed `value`, shown for `seconds` (the console's server caps
    /// it); a zero shows it for one second.
    #[must_use]
    pub fn new(value: String, seconds: u32) -> Self {
        Self {
            value,
            seconds: seconds.max(1),
        }
    }

    /// The seconds left.
    #[must_use]
    pub const fn seconds(&self) -> u32 {
        self.seconds
    }
}

impl std::fmt::Debug for Revealed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Revealed")
            .field("value", &"<withheld>")
            .field("seconds", &self.seconds)
            .finish()
    }
}

/// Where a reveal stands.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Reveal {
    /// Masked, the state the server draws.
    #[default]
    Masked,
    /// The round trip to the server is under way.
    Busy,
    /// The value is shown, counting down.
    Shown(Revealed),
    /// The server refused; the console's words say why.
    Refused(String),
    /// The countdown ran out and the value is gone.
    Expired,
}

/// The state after one second of `reveal`: a shown value counts down and
/// is dropped at zero.
#[must_use]
pub fn ticked(reveal: Reveal) -> Reveal {
    match reveal {
        Reveal::Shown(revealed) if revealed.seconds > 1 => Reveal::Shown(Revealed {
            seconds: revealed.seconds - 1,
            ..revealed
        }),
        Reveal::Shown(_) => Reveal::Expired,
        other => other,
    }
}

/// The words the masked identifier draws, in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifierWords {
    /// The Reveal button.
    pub reveal: Word,
    /// The busy Reveal button.
    pub revealing: Word,
    /// The Hide button.
    pub hide: Word,
    /// The Copy button.
    pub copy: Word,
    /// The copy button's own words.
    pub copied: CopyWords,
    /// The hidden words a mask is heard as, around the open `value`.
    pub masked: Phrase,
    /// The hidden words before a revealed value, around the open `label`.
    pub revealed: Phrase,
    /// The visible countdown, around the open `count`.
    pub countdown: Phrase,
    /// The live region once revealed, around the open `count`.
    pub revealed_said: Phrase,
    /// The live region at [`ANNOUNCE_AT`] seconds.
    pub left_said: Word,
    /// The live region once hidden again.
    pub hidden_said: Word,
    /// The note once hidden again.
    pub expired: Word,
    /// The refusal line, around the open `why`.
    pub refused: Phrase,
    /// The mark the countdown's seconds are grouped by.
    pub grouping: Grouping,
}

impl IdentifierWords {
    /// The masked identifier's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            reveal: tr(locale, Text::IdentifierReveal),
            revealing: tr(locale, Text::IdentifierRevealing),
            hide: tr(locale, Text::IdentifierHide),
            copy: tr(locale, Text::IdentifierCopy),
            copied: CopyWords::of(locale),
            masked: Phrase::new(tr_with(
                locale,
                Text::IdentifierMasked,
                &[("value", Phrase::open("value"))],
            )),
            revealed: Phrase::new(tr_with(
                locale,
                Text::IdentifierRevealed,
                &[("label", Phrase::open("label"))],
            )),
            countdown: Phrase::new(tr_with(
                locale,
                Text::IdentifierCountdown,
                &[("count", Phrase::open("count"))],
            )),
            revealed_said: Phrase::new(tr_with(
                locale,
                Text::IdentifierRevealedSaid,
                &[("count", Phrase::open("count"))],
            )),
            left_said: tr_with(
                locale,
                Text::IdentifierLeftSaid,
                &[("count", ANNOUNCE_AT.into())],
            ),
            hidden_said: tr(locale, Text::IdentifierHiddenSaid),
            expired: tr(locale, Text::IdentifierExpired),
            refused: Phrase::new(tr_with(
                locale,
                Text::IdentifierRefused,
                &[("why", Phrase::open("why"))],
            )),
            grouping: Grouping::of(locale),
        }
    }
}

/// A masked identifier with its reveal.
#[derive(Debug)]
pub struct Identifier {
    /// The id of the identifier's element, unique on the page.
    pub id: String,
    /// The domain and the visible tail.
    pub masked: Masked,
    /// Whether the domain tag shows; `false` where the row already names
    /// the domain, which a reader then hears before the mask.
    pub show_domain: bool,
    /// Where the reveal stands: [`Reveal::Masked`], or what the console's
    /// reveal page answered ([`Reveal::Shown`], [`Reveal::Refused`]); the
    /// `identifier` module counts a shown value down and drops it.
    pub state: Reveal,
    /// The console's page that asks the reason and reveals with a plain
    /// form, which the Reveal control leads to.
    pub fallback: String,
    /// The words, in the page's locale.
    pub words: IdentifierWords,
}

#[derive(Template)]
#[template(path = "identifier/identifier.html", block = "mask")]
struct MaskView<'a> {
    dots: String,
    tail: &'a str,
    words: &'a Word,
}

/// Returns the hidden words that name a mask and the mask itself.
fn mask(masked: &Masked, words: &Word) -> Html {
    // NOTE: WAI-ARIA 1.2, `generic`: naming is prohibited on a span, so the mask is
    // hidden and its words are visually hidden text beside it.
    MaskView {
        dots: std::iter::repeat_n(DOT, DOTS).collect(),
        tail: &masked.tail,
        words,
    }
    .html()
}

/// Returns the words a mask is heard as.
fn masked_words(locale: &Locale, masked: &Masked) -> Word {
    tr_with(
        locale,
        Text::IdentifierMasked,
        &[("value", masked.tail.clone().into())],
    )
}

/// Returns the domain as a reader meets it: the tag, or only its name in
/// visually hidden text where the row already shows it.
fn domain_part(domain: &Domain, show: bool) -> Html {
    if show {
        domain.tag()
    } else {
        Html::raw(format!(
            "<span class=\"fui-sr-only\">{} </span>",
            Html::text(&domain.label)
        ))
    }
}

/// Returns the listing form of a masked identifier: the domain tag (or its
/// hidden name, `show_domain` false) and the mask, with no control.
#[must_use]
pub fn identifier_static(locale: &Locale, masked: &Masked, show_domain: bool) -> Html {
    let words = masked_words(locale, masked);
    Html::raw(format!(
        "<span class=\"fui-identifier fui-identifier-static\">{}{}</span>",
        domain_part(&masked.domain, show_domain),
        mask(masked, &words)
    ))
}

/// Returns a record's identifiers in the order given, each in its listing
/// form with its domain tag, any number of them.
#[must_use]
pub fn identifier_list(locale: &Locale, identifiers: &[Masked]) -> Html {
    let items: String = identifiers
        .iter()
        .map(|masked| format!("<li>{}</li>", identifier_static(locale, masked, true)))
        .collect::<Vec<_>>()
        .concat();
    Html::raw(format!("<ul class=\"fui-identifier-list\">{items}</ul>"))
}

/// The countdown ring's circumference, for a radius of 5.5 on a 14 grid.
pub const RING: f64 = 34.56;

/// The words the `identifier` module says as it counts down.
#[derive(Serialize)]
struct LiveWords<'a> {
    countdown: &'a Phrase,
    revealed_said: &'a Phrase,
    left_said: &'a Word,
    hidden_said: &'a Word,
    group: String,
    announce_at: u32,
}

#[derive(Template)]
#[template(path = "identifier/identifier.html", block = "live")]
struct LiveView {
    id: String,
    seconds: Option<u32>,
    words: String,
    domain: Html,
    shown: bool,
    mask: Html,
    busy: bool,
    small: &'static str,
    revealing: Word,
    fallback: String,
    eye: Html,
    reveal: Word,
    expired: bool,
    expired_words: Word,
    revealed_words: Word,
    value: String,
    ring: f64,
    countdown: Word,
    eye_off: Html,
    hide: Word,
    copy: Html,
    said: Word,
    refused: Option<Word>,
    alert: Html,
}

/// Returns the masked identifier `identifier`.
///
/// Reveal is a link to the console's own reveal page
/// ([`Identifier::fallback`]), which draws the page again with the value
/// shown. A shown value counts down by the second once the `identifier`
/// module runs, drops at zero, and Hide drops it at once; Copy is
/// reachable only while the value shows. A refusal is a line in words
/// beside the identifier, which a toast may repeat.
#[must_use]
pub fn identifier(identifier: Identifier) -> Html {
    let Identifier {
        id,
        masked,
        show_domain,
        state,
        fallback,
        words,
    } = identifier;
    let IdentifierWords {
        reveal,
        revealing,
        hide,
        copy,
        copied,
        masked: masked_heard,
        revealed,
        countdown,
        revealed_said,
        left_said,
        hidden_said,
        expired,
        refused,
        grouping,
    } = words;
    let (value, seconds) = match &state {
        Reveal::Shown(shown) => (Some(shown.value.clone()), Some(shown.seconds)),
        _ => (None, None),
    };
    let shown = value.is_some();
    let count = grouping.write(u64::from(seconds.unwrap_or(0)));
    let said = match (&state, seconds) {
        (Reveal::Shown(_), Some(left)) if left <= ANNOUNCE_AT => left_said.clone(),
        (Reveal::Shown(_), _) => revealed_said.fill(&[("count", &count)]),
        (Reveal::Expired, _) => hidden_said.clone(),
        _ => Word::default(),
    };
    let live = serde_json::to_string(&LiveWords {
        countdown: &countdown,
        revealed_said: &revealed_said,
        left_said: &left_said,
        hidden_said: &hidden_said,
        group: grouping
            .write(1000)
            .chars()
            .nth(1)
            .map(String::from)
            .unwrap_or_default(),
        announce_at: ANNOUNCE_AT,
    })
    // NOTE: serde_json `to_string` fails only for a map with a key that is no
    // string, and these words hold none.
    .unwrap_or_default();
    let copy = match &value {
        Some(value) => copy_from(
            format!("{id}-copy"),
            copy,
            CopySource::Text(value.clone()),
            copied,
        ),
        None => Html::raw(format!(
            "<button type=\"button\" disabled class=\"{}\">{}{}</button>",
            Variant::Secondary.small(),
            icon(Glyph::COPY, "14"),
            copy
        )),
    };
    LiveView {
        mask: mask(&masked, &masked_heard.fill(&[("value", &masked.tail)])),
        domain: domain_part(&masked.domain, show_domain),
        revealed_words: revealed.fill(&[("label", &masked.domain.label)]),
        id,
        seconds,
        words: live,
        shown,
        busy: state == Reveal::Busy,
        small: Variant::Secondary.small(),
        revealing,
        fallback,
        eye: icon(Glyph::EYE, "14"),
        reveal,
        expired: state == Reveal::Expired,
        expired_words: expired,
        value: value.unwrap_or_else(|| std::iter::repeat_n(DOT, DOTS).collect()),
        ring: RING,
        countdown: countdown.fill(&[("count", &count)]),
        eye_off: icon(Glyph::EYE_OFF, "14"),
        hide,
        copy,
        said,
        refused: match &state {
            Reveal::Refused(why) => Some(refused.fill(&[("why", why)])),
            _ => None,
        },
        alert: icon(Glyph::CIRCLE_ALERT, "14"),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Domain, Masked, Reveal, Revealed, kept, ticked};

    #[test]
    fn the_mask_keeps_at_most_half_of_a_value() {
        assert_eq!(3, kept(9, 3));
        assert_eq!(2, kept(4, 3));
        assert_eq!(1, kept(3, 3));
        assert_eq!(0, kept(1, 3));
        assert_eq!(0, kept(0, 3));
        assert_eq!(5, kept(20, 5));
    }

    #[test]
    fn any_characters_mask_and_a_leading_separator_is_dropped() {
        let domain = Domain::new("MRN · Hospital X");
        assert_eq!("12", Masked::new(domain.clone(), "AB-00-12", 3).tail);
        assert_eq!("ÅÖ", Masked::new(domain.clone(), "xyzÄÅÖ", 2).tail);
        assert_eq!("", Masked::new(domain.clone(), "7", 3).tail);
        assert_eq!("••••••12", Masked::new(domain, "AB 12", 3).drawn());
    }

    #[test]
    fn the_mask_never_tells_the_length() {
        let domain = Domain::new("NIR");
        let short = Masked::new(domain.clone(), "000000", 3).drawn();
        let long = Masked::new(domain, "000000000000000", 3).drawn();
        assert_eq!(short.chars().count(), long.chars().count());
    }

    #[test]
    fn a_shown_value_counts_down_and_is_dropped_at_zero() {
        let shown = Reveal::Shown(Revealed::new(String::from("synthetic"), 2));
        let once = ticked(shown);
        assert!(matches!(&once, Reveal::Shown(revealed) if revealed.seconds() == 1));
        assert_eq!(Reveal::Expired, ticked(once));
        assert_eq!(Reveal::Masked, ticked(Reveal::Masked));
    }

    #[test]
    fn a_revealed_value_never_reaches_debug_output() {
        let revealed = Revealed::new(String::from("synthetic-007"), 30);
        let debug = format!("{:?}", Reveal::Shown(revealed));
        assert!(!debug.contains("synthetic-007"), "{debug}");
        assert!(debug.contains("withheld"), "{debug}");
    }
}
