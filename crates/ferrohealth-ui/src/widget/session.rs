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
//! The watch ([`session_watch`]) asks that route when the time is up, when
//! the tab is shown again or regains the focus, and on the user's own input
//! once a minute; only the user's input extends the session, never a timer
//! or a background poll. A [`WARN_AT`] seconds before the idle timeout a
//! modal alert dialog says when the session ends and offers to stay signed
//! in or to sign out (WCAG 2.2 SC 2.2.1, at least 20 seconds to answer;
//! WAI-ARIA Authoring Practices, the alert dialog pattern). When the session
//! has ended the page loads the console's sign-in page with its own address
//! as the return address, so the user comes back to where they were
//! (SC 2.2.5), and says so to every other tab of the console over a
//! `BroadcastChannel`, which [`gone`] does too for a `401` the console's own
//! code meets. With no WebAssembly the console's server redirects the next
//! request to sign-in, as before. No specification governs the timers and
//! the broadcast: our own design, held to WCAG 2.2 and the OWASP Session
//! Management Cheat Sheet (idle and absolute timeouts, logout across tabs).

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::i18n::{Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};

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
    /// The warning's words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        Self {
            title: tr(Text::SessionWarningTitle),
            ends: Phrase::new(tr_with(
                Text::SessionWarningEnds,
                &[("time", Phrase::open("time"))],
            )),
            stay: tr(Text::SessionStay),
            sign_out: tr(Text::SessionSignOut),
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
    /// What the session had left as the server drew the page, carried to the
    /// browser as the server read it (a `SharedValue`).
    pub left: Left,
    /// The seconds before the idle timeout at which the warning opens
    /// ([`WARN_AT`]; never fewer than [`WARN_FLOOR`]).
    pub warn_at: u32,
    /// The warning's words ([`SessionWords::of_page`]).
    pub words: SessionWords,
}

impl Watch {
    /// The watch over the session route `status`, leaving for `sign_in` and
    /// signing out at `sign_out`, from `left` as the server drew the page,
    /// with the warning [`WARN_AT`] seconds ahead and its words in the
    /// page's locale.
    #[must_use]
    pub fn new(
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
            words: SessionWords::of_page(),
        }
    }
}

/// Returns the watch over the session `watch` describes: nothing to see
/// until the warning opens, which the server draws closed.
#[must_use]
pub fn session_watch(watch: Watch) -> AnyView {
    let Watch {
        status,
        sign_in,
        sign_out,
        left,
        warn_at,
        words,
    } = watch;
    let SessionWords {
        title,
        ends,
        stay,
        sign_out: sign_out_words,
    } = words;
    let open = RwSignal::new(false);
    let shown = RwSignal::new(warn_at.max(WARN_FLOOR));
    let node = NodeRef::<leptos::html::Dialog>::new();
    let staying = browser::watch(
        browser::Setup {
            status,
            sign_in,
            left,
            warn_at,
        },
        open,
        shown,
    );
    Effect::new(move |_| sync(node, open.get()));
    let form_id = format!("{WARNING_ID}-sign-out");
    view! {
        <form id=form_id.clone() method="post" action=sign_out data-ferroui-reloads="" hidden></form>
        <dialog
            id=WARNING_ID
            class="fui-dialog fui-modal"
            role="alertdialog"
            aria-labelledby=format!("{WARNING_ID}-title")
            aria-describedby=format!("{WARNING_ID}-ends")
            node_ref=node
            on:cancel=move |event: leptos::ev::Event| {
                event.prevent_default();
                staying.run(());
            }
        >
            <h2 id=format!("{WARNING_ID}-title") class="fui-dialog-title">
                {icon(Glyph::CLOCK, "18")}
                {title}
            </h2>
            <div class="fui-modal-body">
                <p id=format!("{WARNING_ID}-ends")>
                    {move || {
                        ends.fill(&[("time", &crate::widget::countdown::clock(shown.get()))])
                            .into_render()
                    }}
                </p>
            </div>
            <div class="fui-dialog-actions">
                <button type="submit" form=form_id class="fui-button fui-button-secondary">
                    {icon(Glyph::LOG_OUT, "16")}
                    {sign_out_words}
                </button>
                <button
                    type="button"
                    class="fui-button fui-button-primary"
                    autofocus
                    on:click=move |_| staying.run(())
                >
                    {stay}
                </button>
            </div>
        </dialog>
    }
    .into_any()
}

/// Says to every tab of the console, this one included, that the session
/// has ended, so each leaves for sign-in.
///
/// Call it when the console's own code meets a `401` (a server function
/// whose session is gone), and from the signed-out page, which the
/// [`crate::island::SessionEnded`] component does. On the server it does
/// nothing.
pub fn gone() {
    #[cfg(target_arch = "wasm32")]
    browser::tell(Message::Ended);
}

/// Shows the dialog `node` as a modal while `open`, and closes it otherwise.
fn sync(node: NodeRef<leptos::html::Dialog>, open: bool) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(dialog) = node.get() else {
            return;
        };
        if open && !dialog.open() {
            if dialog.show_modal().is_err() {
                leptos::logging::warn!("the browser refused to show the session warning");
            }
        } else if !open && dialog.open() {
            dialog.close();
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (NodeRef<leptos::html::Dialog>, bool) = (node, open);
}

mod browser {
    use leptos::prelude::*;

    use super::Left;
    #[cfg(target_arch = "wasm32")]
    use super::{ACTIVITY_GAP, CHANNEL, Message, plan, sign_in_address};

    /// The console's part of the watch.
    pub(super) struct Setup {
        /// The session route.
        pub(super) status: String,
        /// The sign-in page.
        pub(super) sign_in: String,
        /// What the session had left as the page was drawn.
        pub(super) left: Left,
        /// The seconds of warning.
        pub(super) warn_at: u32,
    }

    /// Starts the watch once the page has hydrated, and returns what Stay
    /// signed in runs.
    pub(super) fn watch(setup: Setup, open: RwSignal<bool>, shown: RwSignal<u32>) -> Callback<()> {
        #[cfg(target_arch = "wasm32")]
        {
            let live = Live::new(setup, open, shown);
            Effect::new(move |_| live.start());
            on_cleanup(move || live.stop());
            Callback::new(move |()| live.ask(Ask::Extend))
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            let Setup {
                status,
                sign_in,
                left,
                warn_at,
            } = setup;
            drop((status, sign_in));
            let _: (Left, u32, RwSignal<bool>, RwSignal<u32>) = (left, warn_at, open, shown);
            Callback::new(|()| {})
        }
    }

    /// Says `message` to every other tab, and to this tab's watch.
    #[cfg(target_arch = "wasm32")]
    pub(super) fn tell(message: Message) {
        if let Ok(channel) = web_sys::BroadcastChannel::new(CHANNEL) {
            if channel.post_message(&message.text().into()).is_err() {
                leptos::logging::warn!("the browser refused the session message");
            }
            channel.close();
        }
    }

    /// Why the session route was asked.
    #[cfg(target_arch = "wasm32")]
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub(super) enum Ask {
        /// To read what is left, which never extends the session.
        Read,
        /// For the user, which extends the idle timeout.
        Extend,
    }

    /// The watch's state in the browser that is no signal.
    #[cfg(target_arch = "wasm32")]
    struct Inner {
        /// The session route.
        status: String,
        /// The sign-in page.
        sign_in: String,
        /// What was left when last learned, and the monotonic and the wall
        /// clock then.
        known: (Left, f64, f64),
        /// The pending timers.
        timers: Vec<TimeoutHandle>,
        /// The warning's ticker.
        ticker: Option<IntervalHandle>,
        /// The listeners, removed when the watch goes.
        hooks: Vec<WindowListenerHandle>,
        /// The channel the other tabs speak on.
        channel: Option<web_sys::BroadcastChannel>,
        /// Whether an extension is on its way.
        asking: bool,
        /// Whether the page is leaving.
        leaving: bool,
    }

    /// The watch in the browser.
    #[cfg(target_arch = "wasm32")]
    #[derive(Clone, Copy)]
    struct Live {
        /// What is no signal.
        inner: StoredValue<Inner, LocalStorage>,
        /// The seconds of warning.
        warn_at: u32,
        /// Whether the warning shows.
        open: RwSignal<bool>,
        /// The seconds the warning says are left.
        shown: RwSignal<u32>,
    }

    #[cfg(target_arch = "wasm32")]
    impl Live {
        /// The state of a watch over `setup`.
        fn new(setup: Setup, open: RwSignal<bool>, shown: RwSignal<u32>) -> Self {
            let (monotonic, wall) = clocks();
            Self {
                inner: StoredValue::new_local(Inner {
                    status: setup.status,
                    sign_in: setup.sign_in,
                    known: (setup.left, monotonic, wall),
                    timers: Vec::new(),
                    ticker: None,
                    hooks: Vec::new(),
                    channel: None,
                    asking: false,
                    leaving: false,
                }),
                warn_at: setup.warn_at,
                open,
                shown,
            }
        }

        /// What is left now, and the milliseconds since it was learned.
        fn left(self) -> (Left, f64) {
            let (left, monotonic, wall) = self
                .inner
                .try_with_value(|inner| inner.known)
                .unwrap_or_default();
            let (now, now_wall) = clocks();
            let spent = crate::widget::countdown::elapsed(now - monotonic, now_wall - wall);
            (
                left.after(crate::widget::countdown::whole_seconds(spent)),
                spent,
            )
        }

        /// Whether the page is leaving, or the watch has gone.
        fn leaving(self) -> bool {
            self.inner.try_with_value(|inner| inner.leaving) != Some(false)
        }

        /// Listens and plans, once the page has hydrated.
        fn start(self) {
            use leptos::wasm_bindgen::JsCast as _;
            use leptos::wasm_bindgen::closure::Closure;
            let mut hooks = Vec::new();
            for kind in ["visibilitychange", "focus", "pageshow"] {
                hooks.push(window_event_listener_untyped(kind, move |_| {
                    if !document().hidden() {
                        self.ask(Ask::Read);
                    }
                }));
            }
            for kind in ["keydown", "pointerdown"] {
                hooks.push(window_event_listener_untyped(kind, move |_| {
                    if !self.open.get_untracked() && self.left().1 >= ACTIVITY_GAP {
                        self.ask(Ask::Extend);
                    }
                }));
            }
            let channel = web_sys::BroadcastChannel::new(CHANNEL).ok();
            if let Some(channel) = &channel {
                let heard = Closure::<dyn Fn(web_sys::MessageEvent)>::new(
                    move |event: web_sys::MessageEvent| match event
                        .data()
                        .as_string()
                        .as_deref()
                        .and_then(Message::parse)
                    {
                        Some(Message::Ended) => self.leave(false),
                        Some(Message::Extended(left)) => self.learn(left),
                        None => {}
                    },
                );
                channel.set_onmessage(Some(heard.as_ref().unchecked_ref()));
                // NOTE: no specification governs this: our own design; the handler lives
                // as long as the page, and `stop` closes the channel it is set on.
                heard.forget();
            }
            self.inner.update_value(|inner| {
                inner.hooks = hooks;
                inner.channel = channel;
            });
            self.plan();
        }

        /// Removes every listener, timer and the channel.
        fn stop(self) {
            self.clear();
            let gone = self
                .inner
                .try_update_value(|inner| (std::mem::take(&mut inner.hooks), inner.channel.take()));
            if let Some((hooks, channel)) = gone {
                for hook in hooks {
                    hook.remove();
                }
                if let Some(channel) = channel {
                    channel.close();
                }
            }
        }

        /// Clears the pending timers and the ticker.
        fn clear(self) {
            let gone = self
                .inner
                .try_update_value(|inner| (std::mem::take(&mut inner.timers), inner.ticker.take()));
            if let Some((timers, ticker)) = gone {
                for timer in timers {
                    timer.clear();
                }
                if let Some(ticker) = ticker {
                    ticker.clear();
                }
            }
        }

        /// Takes `left` as what the session has now, and plans again.
        fn learn(self, left: Left) {
            let (monotonic, wall) = clocks();
            self.inner
                .update_value(|inner| inner.known = (left, monotonic, wall));
            self.plan();
        }

        /// Sets the timers for what is left now: the warning and the end.
        fn plan(self) {
            if self.leaving() {
                return;
            }
            self.clear();
            let next = plan(self.left().0, self.warn_at);
            if next.end_in == 0 {
                self.leave(true);
                return;
            }
            if next.warn_in == Some(0) {
                self.warn();
            } else {
                let _: Option<bool> = self.open.try_set(false);
                if let Some(seconds) = next.warn_in {
                    self.after(seconds);
                }
            }
            self.after(next.end_in);
        }

        /// Reads the session route again in `seconds`.
        fn after(self, seconds: u32) {
            let wait = std::time::Duration::from_secs(u64::from(seconds));
            if let Ok(timer) = set_timeout_with_handle(move || self.ask(Ask::Read), wait) {
                self.inner.update_value(|inner| inner.timers.push(timer));
            }
        }

        /// Opens the warning, its time counting down by the second.
        fn warn(self) {
            let tick = move || {
                let _: Option<u32> = self.shown.try_set(self.left().0.ends_in());
            };
            tick();
            if let Ok(ticker) =
                set_interval_with_handle(tick, std::time::Duration::from_millis(250))
            {
                self.inner.update_value(|inner| inner.ticker = Some(ticker));
            }
            let _: Option<bool> = self.open.try_set(true);
        }

        /// Asks the session route, and acts on its answer.
        pub(super) fn ask(self, ask: Ask) {
            let extend = ask == Ask::Extend;
            let Some(status) = self.inner.try_update_value(|inner| {
                let busy = inner.leaving || (extend && inner.asking);
                inner.asking |= extend;
                (!busy).then(|| inner.status.clone())
            }) else {
                return;
            };
            let Some(status) = status else {
                return;
            };
            wasm_bindgen_futures::spawn_local(async move {
                let answer = fetch(&status, extend).await;
                if extend {
                    self.inner.try_update_value(|inner| inner.asking = false);
                }
                match answer {
                    Some(Ok(left)) => {
                        self.learn(left);
                        if extend {
                            tell(Message::Extended(left));
                        }
                    }
                    Some(Err(())) => self.leave(true),
                    None => self.plan(),
                }
            });
        }

        /// Leaves for the sign-in page with this page as the return address,
        /// telling the other tabs first when `tell_others`.
        fn leave(self, tell_others: bool) {
            let Some(Some(sign_in)) = self.inner.try_update_value(|inner| {
                let first = !inner.leaving;
                inner.leaving = true;
                first.then(|| inner.sign_in.clone())
            }) else {
                return;
            };
            self.clear();
            if tell_others {
                tell(Message::Ended);
            }
            let location = window().location();
            let here = format!(
                "{}{}",
                location.pathname().unwrap_or_default(),
                location.search().unwrap_or_default()
            );
            if location.assign(&sign_in_address(&sign_in, &here)).is_err() {
                leptos::logging::warn!("the browser refused the sign-in page");
            }
        }
    }

    /// The monotonic and the wall clock, in milliseconds.
    #[cfg(target_arch = "wasm32")]
    fn clocks() -> (f64, f64) {
        let monotonic = window()
            .performance()
            .map_or(0.0, |performance| performance.now());
        (monotonic, web_sys::js_sys::Date::now())
    }

    /// Asks the session route `status`, a `POST` to `extend` and a `GET`
    /// otherwise: what is left, `Err` when it answered `401`, and `None`
    /// when it did not answer or answered something else.
    #[cfg(target_arch = "wasm32")]
    async fn fetch(status: &str, extend: bool) -> Option<Result<Left, ()>> {
        use leptos::wasm_bindgen::JsCast as _;
        use wasm_bindgen_futures::JsFuture;
        let init = web_sys::RequestInit::new();
        init.set_credentials(web_sys::RequestCredentials::SameOrigin);
        init.set_method(if extend { "POST" } else { "GET" });
        let response: web_sys::Response =
            JsFuture::from(window().fetch_with_str_and_init(status, &init))
                .await
                .ok()?
                .dyn_into()
                .ok()?;
        if response.status() == 401 {
            return Some(Err(()));
        }
        if !response.ok() {
            leptos::logging::warn!("the session route answered {}", response.status());
            return None;
        }
        let text = JsFuture::from(response.text().ok()?)
            .await
            .ok()?
            .as_string()?;
        match leptos::serde_json::from_str(&text) {
            Ok(left) => Some(Ok(left)),
            Err(error) => {
                let (line, column) = (error.line(), error.column());
                leptos::logging::warn!(
                    "the session route's answer is unreadable at {line}:{column}"
                );
                None
            }
        }
    }
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
