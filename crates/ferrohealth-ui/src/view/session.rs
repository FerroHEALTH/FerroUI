// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The end of a console session, in the page.
//!
//! A protected page leaves for the sign-in page the moment its session
//! ends, warns before an idle timeout with "Stay signed in", and every tab
//! of the console ends together.
//! The console's server tells the page how many seconds its session has
//! left ([`Left`]) as it draws the page, and answers its session route with
//! the same on every ask: `GET` reads them and never counts as activity,
//! `POST` counts as the user's activity and extends the idle timeout, and
//! `401` says the session is gone (`docs/consoles.md` § The session's end).
//! The `session` module, over the watch the server draws
//! ([`session_watch`]), asks that route when the time is up, when
//! the tab is shown again or regains the focus, and on the user's own input
//! once a minute; only the user's input extends the session, never a timer
//! or a background poll. A [`WARN_AT`] seconds before the idle timeout a
//! modal alert dialog says when the session ends and offers to stay signed
//! in or to sign out (WCAG 2.2 SC 2.2.1, at least 20 seconds to answer;
//! WAI-ARIA Authoring Practices, the alert dialog pattern). When the session
//! has ended the page loads the console's sign-in page with its own address
//! as the return address, so the user comes back to where they were
//! (SC 2.2.5), and says so to every other tab of the console over a
//! `BroadcastChannel`, which the signed-out page does too
//! ([`session_ended`]), as does the search box for a `401`. With no script
//! the console's server redirects the next request to sign-in, as before. No specification governs the timers and
//! the broadcast: our own design, held to WCAG 2.2 and the OWASP Session
//! Management Cheat Sheet (idle and absolute timeouts, logout across tabs).

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::countdown::clock;

/// The seconds before the idle timeout at which the warning opens.
pub const WARN_AT: u32 = 120;

/// The fewest seconds the warning gives the user to answer (WCAG 2.2
/// SC 2.2.1, "at least 20 seconds").
pub const WARN_FLOOR: u32 = 20;

/// The fewest milliseconds between two extensions the user's input asks
/// for, so typing extends the session once a minute, not on every key.
pub const ACTIVITY_GAP: f64 = 60_000.0;

/// The name of the `BroadcastChannel` the tabs of a console share; a
/// channel is bound to its origin, so two consoles never hear each other.
pub const CHANNEL: &str = "ferroui-session";

/// The query parameter that carries the return address to the sign-in page.
pub const RETURN: &str = "return";

/// The id of the warning dialog.
pub const WARNING_ID: &str = "ferroui-session-warning";

/// What a session has left, in whole seconds, as the console's server
/// counts it: the wire of its session route (`{"idle": 840, "absolute":
/// 27000}`).
///
/// Seconds left, never a deadline, so a browser clock that is wrong moves
/// nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Left {
    /// The seconds until the idle timeout, unless the user acts.
    pub idle: u32,
    /// The seconds until the session's absolute lifetime ends, which
    /// nothing extends.
    pub absolute: u32,
}

impl Left {
    /// What is left `seconds` later.
    #[must_use]
    pub const fn after(self, seconds: u32) -> Self {
        Self {
            idle: self.idle.saturating_sub(seconds),
            absolute: self.absolute.saturating_sub(seconds),
        }
    }

    /// The seconds until the session ends, by whichever timeout comes first.
    #[must_use]
    pub const fn ends_in(self) -> u32 {
        if self.idle < self.absolute {
            self.idle
        } else {
            self.absolute
        }
    }
}

/// When the watch acts next, in seconds from what it knows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plan {
    /// When the warning opens, `Some(0)` for now; `None` when the absolute
    /// lifetime ends first, which no answer to a warning could extend.
    pub warn_in: Option<u32>,
    /// When the session ends.
    pub end_in: u32,
}

/// The plan for a session with `left`, warning `warn_at` seconds before
/// the idle timeout (never fewer than [`WARN_FLOOR`]).
#[must_use]
pub fn plan(left: Left, warn_at: u32) -> Plan {
    let warn_at = warn_at.max(WARN_FLOOR);
    if left.absolute < left.idle {
        Plan {
            warn_in: None,
            end_in: left.absolute,
        }
    } else {
        Plan {
            warn_in: Some(left.idle.saturating_sub(warn_at)),
            end_in: left.idle,
        }
    }
}

/// What one tab tells the others on [`CHANNEL`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// The session has ended: every tab leaves for sign-in.
    Ended,
    /// The user stayed signed in, so the session has this left.
    Extended(Left),
}

impl Message {
    /// The message as it travels: `ended`, or `extended <idle> <absolute>`.
    #[must_use]
    pub fn text(self) -> String {
        match self {
            Self::Ended => String::from("ended"),
            Self::Extended(left) => format!("extended {} {}", left.idle, left.absolute),
        }
    }

    /// The message `text` carries, `None` for anything else.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut words = text.split(' ');
        let message = match words.next()? {
            "ended" => Self::Ended,
            "extended" => Self::Extended(Left {
                idle: words.next()?.parse().ok()?,
                absolute: words.next()?.parse().ok()?,
            }),
            _ => return None,
        };
        words.next().is_none().then_some(message)
    }
}

/// The address of the sign-in page `sign_in` with `here`, the page left,
/// as its [`RETURN`] parameter, percent-encoded (RFC 3986 § 2.1).
#[must_use]
pub fn sign_in_address(sign_in: &str, here: &str) -> String {
    let joint = if sign_in.contains('?') { '&' } else { '?' };
    let mut address = format!("{sign_in}{joint}{RETURN}=");
    for byte in here.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            address.push(char::from(byte));
        } else {
            const HEX: &[u8; 16] = b"0123456789ABCDEF";
            address.push('%');
            for nibble in [byte >> 4, byte & 0x0f] {
                address.push(char::from(
                    HEX.get(usize::from(nibble)).copied().unwrap_or(b'0'),
                ));
            }
        }
    }
    address
}

/// The words of the warning, resolved on the server in the page's locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionWords {
    /// The dialog's heading.
    pub title: Word,
    /// The sentence that says when the session ends, around the open
    /// `time`.
    pub ends: Phrase,
    /// The button that stays signed in.
    pub stay: Word,
    /// The button that signs out.
    pub sign_out: Word,
}

impl SessionWords {
    /// The warning's words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        Self {
            title: tr(locale, Text::SessionWarningTitle),
            ends: Phrase::new(tr_with(
                locale,
                Text::SessionWarningEnds,
                &[("time", Phrase::open("time"))],
            )),
            stay: tr(locale, Text::SessionStay),
            sign_out: tr(locale, Text::SessionSignOut),
        }
    }
}

/// What the watch needs from the console.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watch {
    /// The console's session route: `GET` and `POST` answer [`Left`] as
    /// JSON, or `401` once the session is gone.
    pub status: String,
    /// The console's sign-in page, which takes the return address as
    /// [`RETURN`].
    pub sign_in: String,
    /// The console's sign-out route, which the warning's Sign out posts to.
    pub sign_out: String,
    /// What the session had left as the server drew the page.
    pub left: Left,
    /// The seconds before the idle timeout at which the warning opens
    /// ([`WARN_AT`]; never fewer than [`WARN_FLOOR`]).
    pub warn_at: u32,
    /// The warning's words ([`SessionWords::of`]).
    pub words: SessionWords,
}

impl Watch {
    /// The watch over the session route `status`, leaving for `sign_in` and
    /// signing out at `sign_out`, from `left` as the server drew the page,
    /// with the warning [`WARN_AT`] seconds ahead and its words in
    /// `locale`.
    #[must_use]
    pub fn new(
        locale: &Locale,
        status: impl Into<String>,
        sign_in: impl Into<String>,
        sign_out: impl Into<String>,
        left: Left,
    ) -> Self {
        Self {
            status: status.into(),
            sign_in: sign_in.into(),
            sign_out: sign_out.into(),
            left,
            warn_at: WARN_AT,
            words: SessionWords::of(locale),
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"<form id="{{ id }}-sign-out" method="post" action="{{ sign_out }}" hidden></form><dialog id="{{ id }}" role="alertdialog" aria-labelledby="{{ id }}-title" aria-describedby="{{ id }}-ends" data-fui="session" data-fui-status="{{ status }}" data-fui-sign-in="{{ sign_in }}" data-fui-idle="{{ idle }}" data-fui-absolute="{{ absolute }}" data-fui-warn-at="{{ warn_at }}" data-fui-ends="{{ ends }}" class="fui-dialog fui-modal"><h2 id="{{ id }}-title" class="fui-dialog-title">{{ clock }}{{ title }}</h2><div class="fui-modal-body"><p id="{{ id }}-ends">{{ said }}</p></div><div class="fui-dialog-actions"><button type="submit" form="{{ id }}-sign-out" class="fui-button fui-button-secondary">{{ log_out }}{{ sign_out_words }}</button><button type="button" autofocus data-fui-stay class="fui-button fui-button-primary">{{ stay }}</button></div></dialog>"#,
    ext = "html"
)]
struct WatchView<'a> {
    id: &'static str,
    sign_out: &'a str,
    status: &'a str,
    sign_in: &'a str,
    idle: u32,
    absolute: u32,
    warn_at: u32,
    ends: String,
    clock: Html,
    title: &'a Word,
    said: Word,
    log_out: Html,
    sign_out_words: &'a Word,
    stay: &'a Word,
}

/// Returns the watch over the session `watch` describes: nothing to see
/// until the warning opens, which the server draws closed and the
/// `session` module opens.
#[must_use]
pub fn session_watch(watch: &Watch) -> Html {
    let warn_at = watch.warn_at.max(WARN_FLOOR);
    WatchView {
        id: WARNING_ID,
        sign_out: &watch.sign_out,
        status: &watch.status,
        sign_in: &watch.sign_in,
        idle: watch.left.idle,
        absolute: watch.left.absolute,
        warn_at,
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and a phrase is a string or an object of one.
        ends: serde_json::to_string(&watch.words.ends).unwrap_or_default(),
        clock: icon(Glyph::CLOCK, "18"),
        title: &watch.words.title,
        said: watch.words.ends.fill(&[("time", &clock(warn_at))]),
        log_out: icon(Glyph::LOG_OUT, "16"),
        sign_out_words: &watch.words.sign_out,
        stay: &watch.words.stay,
    }
    .html()
}

/// Returns the mark a console's signed-out page carries, with which the
/// `session` module says to every tab of the console that the session has
/// ended, so each leaves for sign-in; it draws nothing.
#[must_use]
pub fn session_ended() -> Html {
    Html::raw(String::from(
        "<span data-fui=\"session\" data-fui-ended hidden></span>",
    ))
}

#[cfg(test)]
mod tests {
    use super::{Left, Message, Plan, WARN_FLOOR, plan, sign_in_address};

    #[test]
    fn the_warning_opens_its_seconds_before_the_idle_timeout_and_never_under_twenty() {
        let left = Left {
            idle: 900,
            absolute: 28_800,
        };
        assert_eq!(
            Plan {
                warn_in: Some(780),
                end_in: 900
            },
            plan(left, 120)
        );
        assert_eq!(
            Some(900 - WARN_FLOOR),
            plan(left, 5).warn_in,
            "SC 2.2.1: at least 20 seconds to answer"
        );
        assert_eq!(
            Some(0),
            plan(Left { idle: 15, ..left }, 120).warn_in,
            "a session with less than the warning left warns now"
        );
    }

    #[test]
    fn an_absolute_lifetime_that_ends_first_ends_the_session_with_no_warning() {
        let left = Left {
            idle: 900,
            absolute: 300,
        };
        assert_eq!(
            Plan {
                warn_in: None,
                end_in: 300
            },
            plan(left, 120),
            "staying signed in cannot extend the absolute lifetime"
        );
    }

    #[test]
    fn what_is_left_counts_down_and_stops_at_zero() {
        let left = Left {
            idle: 30,
            absolute: 60,
        };
        assert_eq!(
            Left {
                idle: 0,
                absolute: 15
            },
            left.after(45)
        );
        assert_eq!(30, left.ends_in());
        assert_eq!(0, left.after(90).ends_in());
    }

    #[test]
    fn a_message_reads_back_and_anything_else_is_refused() {
        let extended = Message::Extended(Left {
            idle: 840,
            absolute: 27_000,
        });
        assert_eq!(Some(extended), Message::parse(&extended.text()));
        assert_eq!(Some(Message::Ended), Message::parse(&Message::Ended.text()));
        for text in [
            "",
            "end",
            "extended 1",
            "extended 1 x",
            "extended 1 2 3",
            "ended now",
        ] {
            assert_eq!(None, Message::parse(text), "{text:?}");
        }
    }

    #[test]
    fn the_return_address_rides_the_sign_in_page_percent_encoded() {
        assert_eq!(
            "/sign-in?return=/records%3Fq%3Dr%C3%B6ntgen%26page%3D2",
            sign_in_address("/sign-in", "/records?q=röntgen&page=2")
        );
        assert_eq!(
            "/sign-in?state=expired&return=/",
            sign_in_address("/sign-in?state=expired", "/")
        );
    }
}
