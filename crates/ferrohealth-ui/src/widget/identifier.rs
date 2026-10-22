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
//! [`identifier`] is the interactive form, a widget a console calls inside
//! an island of its own. Reveal runs the console's callback, which asks the
//! reason, makes the audited round trip to its server and puts the answer
//! in the widget's [`Reveal`] signal; the widget then counts the seconds
//! down and drops the value when they run out or Hide is pressed, so the
//! value is held in the browser no longer than the countdown. The value is
//! a [`Revealed`], which never serializes and whose `Debug` withholds it,
//! so it can reach no island prop and no log. The countdown speaks through
//! a polite live region at [`ANNOUNCE_AT`] seconds and at zero (WAI-ARIA
//! 1.2, `status`), the reveal is a toggle with `aria-pressed`, and the
//! three faces (masked, revealed, the busy round trip) share one reserved
//! slot, so revealing, counting down and hiding move nothing around it.
//! [`identifier_static`] is the listing form, with no control.
//!
//! No specification governs the component: our own design.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::tag::Tag;
use crate::widget::copy::{CopyWords, copy_button};

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
    fn tag(&self) -> AnyView {
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
/// no island prop, no log and no error message.
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

/// The words the masked identifier draws, resolved on the server in the
/// page's locale, so the island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IdentifierWords {
    /// The Reveal button.
    pub reveal: String,
    /// The busy Reveal button.
    pub revealing: String,
    /// The Hide button.
    pub hide: String,
    /// The Copy button.
    pub copy: String,
    /// The copy button's own words.
    pub copied: CopyWords,
    /// The hidden words before a revealed value, around the open `label`.
    pub revealed: Phrase,
    /// The visible countdown, around the open `count`.
    pub countdown: Phrase,
    /// The live region once revealed, around the open `count`.
    pub revealed_said: Phrase,
    /// The live region at [`ANNOUNCE_AT`] seconds.
    pub left_said: String,
    /// The live region once hidden again.
    pub hidden_said: String,
    /// The note once hidden again.
    pub expired: String,
    /// The refusal line, around the open `why`.
    pub refused: Phrase,
}

impl IdentifierWords {
    /// The masked identifier's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            reveal: tr(Text::IdentifierReveal),
            revealing: tr(Text::IdentifierRevealing),
            hide: tr(Text::IdentifierHide),
            copy: tr(Text::IdentifierCopy),
            copied: CopyWords::of_page(),
            revealed: Phrase::new(tr_with(
                Text::IdentifierRevealed,
                &[("label", Phrase::open("label"))],
            )),
            countdown: Phrase::new(tr_with(
                Text::IdentifierCountdown,
                &[("count", Phrase::open("count"))],
            )),
            revealed_said: Phrase::new(tr_with(
                Text::IdentifierRevealedSaid,
                &[("count", Phrase::open("count"))],
            )),
            left_said: tr_with(Text::IdentifierLeftSaid, &[("count", ANNOUNCE_AT.into())]),
            hidden_said: tr(Text::IdentifierHiddenSaid),
            expired: tr(Text::IdentifierExpired),
            refused: Phrase::new(tr_with(
                Text::IdentifierRefused,
                &[("why", Phrase::open("why"))],
            )),
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
    /// Where the reveal stands. The server draws [`Reveal::Masked`]; the
    /// console's callback sets [`Reveal::Shown`] or [`Reveal::Refused`],
    /// and the widget counts down and drops the value itself.
    pub state: RwSignal<Reveal>,
    /// The console's page that asks the reason and reveals with a plain
    /// form, which the Reveal control leads to before the page hydrates.
    pub fallback: String,
    /// The words, in the page's locale.
    pub words: IdentifierWords,
}

/// Returns the hidden words that name a mask and the mask itself.
fn mask(masked: &Masked, words: &str) -> AnyView {
    // NOTE: WAI-ARIA 1.2, `generic`: naming is prohibited on a span, so the mask is
    // hidden and its words are visually hidden text beside it.
    view! {
        <span class="fui-identifier-mask">
            <span aria-hidden="true">
                <span class="fui-identifier-dots">
                    {std::iter::repeat_n(DOT, DOTS).collect::<String>()}
                </span>
                {masked.tail.clone()}
            </span>
            <span class="fui-sr-only">{words.to_owned()}</span>
        </span>
    }
    .into_any()
}

/// Returns the words a mask is heard as.
fn masked_words(masked: &Masked) -> String {
    tr_with(
        Text::IdentifierMasked,
        &[("value", masked.tail.clone().into())],
    )
}

/// Returns the domain as a reader meets it: the tag, or only its name in
/// visually hidden text where the row already shows it.
fn domain_part(domain: &Domain, show: bool) -> AnyView {
    if show {
        domain.tag()
    } else {
        let name = format!("{} ", domain.label);
        view! { <span class="fui-sr-only">{name}</span> }.into_any()
    }
}

/// Returns the listing form of a masked identifier: the domain tag (or its
/// hidden name, `show_domain` false) and the mask, with no control.
#[must_use]
pub fn identifier_static(masked: &Masked, show_domain: bool) -> AnyView {
    let words = masked_words(masked);
    view! {
        <span class="fui-identifier fui-identifier-static">
            {domain_part(&masked.domain, show_domain)}
            {mask(masked, &words)}
        </span>
    }
    .into_any()
}

/// Returns a record's identifiers in the order given, each in its listing
/// form with its domain tag, any number of them.
#[must_use]
pub fn identifier_list(identifiers: &[Masked]) -> AnyView {
    let items = identifiers
        .iter()
        .map(|masked| view! { <li>{identifier_static(masked, true)}</li> })
        .collect_view()
        .into_any();
    view! { <ul class="fui-identifier-list">{items}</ul> }.into_any()
}

/// The countdown ring's circumference, for a radius of 5.5 on a 14 grid.
const RING: f64 = 34.56;

/// The ring's dash offset with `left` of `total` seconds remaining.
fn ring_offset(left: u32, total: u32) -> String {
    let share = f64::from(left) / f64::from(total.max(1));
    format!("{:.2}", RING * (1.0 - share.min(1.0)))
}

/// Returns the masked identifier `identifier`, Reveal running `on_reveal`.
///
/// The server draws it masked, and until the page hydrates the Reveal
/// control is a link to the console's own reveal page
/// ([`Identifier::fallback`]). Once hydrated, Reveal sets
/// [`Reveal::Busy`] and runs `on_reveal`, whose answer the console puts in
/// [`Identifier::state`]. A shown value counts down by the second, drops
/// at zero, and Hide drops it at once; Copy is reachable only while the
/// value shows. A refusal is a line in words beside the identifier, which
/// a toast may repeat.
#[must_use]
pub fn identifier(identifier: Identifier, on_reveal: Callback<()>) -> AnyView {
    let Identifier {
        id,
        masked,
        show_domain,
        state,
        fallback,
        words,
    } = identifier;
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    let total = StoredValue::new(0_u32);
    count_down(state, total);
    let IdentifierWords {
        reveal,
        revealing,
        hide,
        copy,
        copied,
        revealed,
        countdown,
        revealed_said,
        left_said,
        hidden_said,
        expired,
        refused,
    } = words;
    let shown = move || matches!(state.get(), Reveal::Shown(_));
    let said = Memo::new(move |_| match state.get() {
        Reveal::Shown(revealed) if revealed.seconds <= ANNOUNCE_AT => left_said.clone(),
        Reveal::Shown(_) => {
            let seconds = total.get_value().to_string();
            revealed_said.fill(&[("count", &seconds)])
        }
        Reveal::Expired => hidden_said.clone(),
        Reveal::Masked | Reveal::Busy | Reveal::Refused(_) => String::new(),
    });
    let refusal = move || match state.get() {
        Reveal::Refused(why) => Some(refused.fill(&[("why", &why)])),
        _ => None,
    };
    let mask_words = masked_words(&masked);
    let shown_words = ShownWords {
        revealed: revealed.fill(&[("label", &masked.domain.label)]),
        countdown,
        hide,
        copy_id: format!("{id}-copy"),
        copy,
        copied,
    };
    view! {
        <span class="fui-identifier" id=id>
            {domain_part(&masked.domain, show_domain)}
            <span class="fui-identifier-slot">
                <span class="fui-identifier-face" class:is-away=shown inert=shown>
                    {mask(&masked, &mask_words)}
                    <a
                        class=format!("{} fui-identifier-reveal", Variant::Secondary.small())
                        href=fallback
                        hidden=move || hydrated.get()
                    >
                        {icon(Glyph::EYE, "14")}
                        {reveal.clone()}
                    </a>
                    {reveal_press(state, on_reveal, reveal, revealing, hydrated)}
                    <span
                        class="fui-identifier-expired"
                        class:is-away=move || state.get() != Reveal::Expired
                    >
                        {expired}
                    </span>
                </span>
                {revealed_face(state, total, shown_words)}
            </span>
            <span class="fui-sr-only" role="status">
                {move || said.get()}
            </span>
            {move || {
                refusal()
                    .map(|line| {
                        view! {
                            <span class="fui-help fui-error fui-identifier-refused" role="status">
                                {icon(Glyph::CIRCLE_ALERT, "14")}
                                {line}
                            </span>
                        }
                    })
            }}
        </span>
    }
    .into_any()
}

/// Returns the Reveal button and its busy twin, which share one place, shown
/// once the page has hydrated.
fn reveal_press(
    state: RwSignal<Reveal>,
    on_reveal: Callback<()>,
    reveal: String,
    revealing: String,
    hydrated: RwSignal<bool>,
) -> AnyView {
    let busy = move || state.get() == Reveal::Busy;
    let press = move |_| {
        state.set(Reveal::Busy);
        on_reveal.run(());
    };
    view! {
        <span class="fui-identifier-press" hidden=move || !hydrated.get()>
            <button
                type="button"
                class=format!("{} fui-identifier-reveal", Variant::Secondary.small())
                class:is-away=busy
                aria-pressed="false"
                inert=busy
                on:click=press
            >
                {icon(Glyph::EYE, "14")}
                {reveal}
            </button>
            <button
                type="button"
                class=Variant::Secondary.small()
                class:is-away=move || !busy()
                aria-busy="true"
                disabled
            >
                <span class="fui-spinner-ring" aria-hidden="true"></span>
                {revealing}
            </button>
        </span>
    }
    .into_any()
}

/// The words of the revealed face.
struct ShownWords {
    /// The hidden words before the value.
    revealed: String,
    /// The visible countdown, around the open `count`.
    countdown: Phrase,
    /// The Hide button.
    hide: String,
    /// The copy button's id.
    copy_id: String,
    /// The Copy button.
    copy: String,
    /// The copy button's own words.
    copied: CopyWords,
}

/// Returns the revealed face of the slot: the value with its countdown,
/// Hide and Copy, away and inert while nothing is revealed.
fn revealed_face(state: RwSignal<Reveal>, total: StoredValue<u32>, words: ShownWords) -> AnyView {
    let ShownWords {
        revealed: revealed_words,
        countdown,
        hide,
        copy_id,
        copy,
        copied,
    } = words;
    let shown = move || matches!(state.get(), Reveal::Shown(_));
    let value = move || match state.get() {
        Reveal::Shown(revealed) => revealed.value,
        _ => masked_placeholder(),
    };
    let left = move || match state.get() {
        Reveal::Shown(revealed) => revealed.seconds,
        _ => 0,
    };
    let text = Signal::derive(move || match state.get() {
        Reveal::Shown(revealed) => revealed.value,
        _ => String::new(),
    });
    view! {
        <span class="fui-identifier-face" class:is-away=move || !shown() inert=move || !shown()>
            <span class="fui-identifier-shown">
                <span class="fui-sr-only">{revealed_words}</span>
                <span class="fui-identifier-value">{value}</span>
                <span class="fui-identifier-countdown" aria-hidden="true">
                    <svg viewBox="0 0 14 14">
                        <circle class="fui-identifier-countdown-track" cx="7" cy="7" r="5.5"></circle>
                        <circle
                            class="fui-identifier-countdown-left"
                            cx="7"
                            cy="7"
                            r="5.5"
                            stroke-dasharray=RING.to_string()
                            stroke-dashoffset=move || ring_offset(left(), total.get_value())
                        ></circle>
                    </svg>
                    {move || countdown.fill(&[("count", &left().to_string())])}
                </span>
            </span>
            <span class="fui-identifier-tools">
                <button
                    type="button"
                    class=format!("{} fui-identifier-reveal", Variant::Secondary.small())
                    aria-pressed="true"
                    on:click=move |_| state.set(Reveal::Masked)
                >
                    {icon(Glyph::EYE_OFF, "14")}
                    {hide}
                </button>
                {copy_button(copy_id, copy, text, copied)}
            </span>
        </span>
    }
    .into_any()
}

/// The text the revealed face holds while nothing is revealed, so the
/// slot keeps the room of a value.
fn masked_placeholder() -> String {
    std::iter::repeat_n(DOT, DOTS).collect()
}

/// Counts a shown value down by the second in the browser, keeping its
/// first count in `total` for the ring and the live region.
fn count_down(state: RwSignal<Reveal>, total: StoredValue<u32>) {
    #[cfg(target_arch = "wasm32")]
    {
        let handle: StoredValue<Option<IntervalHandle>> = StoredValue::new(None);
        Effect::new(move |_| {
            let showing = match state.get() {
                Reveal::Shown(revealed) => Some(revealed.seconds),
                _ => None,
            };
            match showing {
                Some(seconds) => {
                    if handle.get_value().is_none() {
                        total.set_value(seconds);
                        let tick = move || state.update(|reveal| *reveal = ticked(reveal.clone()));
                        match set_interval_with_handle(tick, std::time::Duration::from_secs(1)) {
                            Ok(started) => handle.set_value(Some(started)),
                            Err(_) => state.set(Reveal::Masked),
                        }
                    }
                }
                None => {
                    if let Some(started) = handle.get_value() {
                        started.clear();
                        handle.set_value(None);
                    }
                }
            }
        });
        on_cleanup(move || {
            if let Some(started) = handle.get_value() {
                started.clear();
            }
        });
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (RwSignal<Reveal>, StoredValue<u32>) = (state, total);
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
