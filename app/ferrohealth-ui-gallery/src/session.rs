// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The gallery's stand-in for a console session, so the journeys can watch
//! one end.
//!
//! `/session/start/<brand>?idle=…&absolute=…` starts one in a
//! cookie, [`SESSION_ROUTE`] answers what it has left as a console's session
//! route does (`GET` reads, `POST` extends the idle timeout, `401` once it
//! is gone), and a brand's pages draw the library's watch while it lasts.
//!
//! The gallery has no sign-in: a session exists only while a journey has
//! started one, and every other page is drawn with none. Nothing in it is
//! a credential. No specification governs the gallery's session: our own
//! design.

use ferrohealth_ui::island::SessionWatch;
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::widget::session::{Left, WARN_FLOOR, Watch};
use leptos::prelude::*;
use leptos_router::hooks::use_location;

/// The cookie the session is kept in.
pub const SESSION_COOKIE: &str = "gallery-session";

/// The gallery's session route.
pub const SESSION_ROUTE: &str = "/session";

/// The route that starts a session, under which `/<brand>` names the page
/// it leads to.
pub const SESSION_START: &str = "/session/start";

/// A session as its cookie keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Kept {
    /// The idle timeout's length, in seconds.
    pub idle_for: u32,
    /// When the idle timeout ends, in milliseconds since the Unix epoch.
    pub idle_end: u64,
    /// When the absolute lifetime ends, in milliseconds since the Unix epoch.
    pub absolute_end: u64,
}

impl Kept {
    /// A session started at `now` (milliseconds since the Unix epoch) with
    /// an idle timeout of `idle_for` and a lifetime of `absolute_for`
    /// seconds.
    #[must_use]
    pub fn started(now: u64, idle_for: u32, absolute_for: u32) -> Self {
        let absolute_end = now.saturating_add(u64::from(absolute_for).saturating_mul(1000));
        Self {
            idle_for,
            idle_end: now
                .saturating_add(u64::from(idle_for).saturating_mul(1000))
                .min(absolute_end),
            absolute_end,
        }
    }

    /// The session after the user's activity at `now`: the idle timeout
    /// starts again, never past the absolute lifetime.
    #[must_use]
    pub fn extended(self, now: u64) -> Self {
        Self {
            idle_end: now
                .saturating_add(u64::from(self.idle_for).saturating_mul(1000))
                .min(self.absolute_end),
            ..self
        }
    }

    /// The cookie's value: `<idle_for>.<idle_end>.<absolute_end>`.
    #[must_use]
    pub fn value(self) -> String {
        format!("{}.{}.{}", self.idle_for, self.idle_end, self.absolute_end)
    }

    /// The session the `Cookie` header value `cookies` keeps, if any.
    #[must_use]
    pub fn read(cookies: &str) -> Option<Self> {
        let value = cookies.split(';').find_map(|pair| {
            let (name, value) = pair.trim().split_once('=')?;
            (name == SESSION_COOKIE).then_some(value)
        })?;
        let mut parts = value.split('.');
        let kept = Self {
            idle_for: parts.next()?.parse().ok()?,
            idle_end: parts.next()?.parse().ok()?,
            absolute_end: parts.next()?.parse().ok()?,
        };
        parts.next().is_none().then_some(kept)
    }

    /// What the session has left at `now`, `None` once it has ended.
    #[must_use]
    pub fn left(self, now: u64) -> Option<Left> {
        let seconds = |end: u64| u32::try_from(end.saturating_sub(now).div_euclid(1000)).ok();
        let left = Left {
            idle: seconds(self.idle_end)?,
            absolute: seconds(self.absolute_end)?,
        };
        (left.ends_in() > 0).then_some(left)
    }
}

/// The milliseconds since the Unix epoch now, on the server.
#[cfg(not(target_arch = "wasm32"))]
#[must_use]
pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |since| {
            u64::try_from(since.as_millis()).unwrap_or(u64::MAX)
        })
}

/// What the session of the request has left as the server draws the page;
/// in the browser, which reads the value the server serialized, none.
fn left_requested() -> Option<Left> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Kept::read(&crate::cookies::server_cookies()).and_then(|kept| kept.left(now()))
    }
    #[cfg(target_arch = "wasm32")]
    {
        None
    }
}

/// The library's watch over the gallery's session, on a brand's page while
/// a journey's session lasts.
#[component]
pub fn GallerySession() -> impl IntoView {
    // NOTE: the Leptos book, `ssr/22_life_cycle`: what the session had left reaches the
    // hydrating page serialized, so both halves draw the same watch.
    let left = *SharedValue::new(left_requested);
    let brand = use_location()
        .pathname
        .with_untracked(|path| path.split('/').nth(1).and_then(Brand::parse));
    let (Some(left), Some(brand)) = (left, brand) else {
        return ().into_any();
    };
    // The shortest warning the library gives (SC 2.2.1), so a journey's session can be short.
    let watch = Watch {
        warn_at: WARN_FLOOR,
        ..Watch::new(
            SESSION_ROUTE,
            format!("/sign-in/{}/expired", brand.key()),
            format!("{}/{}", crate::app::SIGN_OUT_ROUTE, brand.key()),
            left,
        )
    };
    view! { <SessionWatch watch /> }.into_any()
}

#[cfg(test)]
mod tests {
    use super::{Kept, SESSION_COOKIE};
    use ferrohealth_ui::widget::session::Left;

    #[test]
    fn a_session_reads_back_from_its_cookie_and_anything_else_is_none() {
        let kept = Kept::started(1_000_000, 900, 28_800);
        let cookies = format!("gallery-theme=dark; {SESSION_COOKIE}={}", kept.value());
        assert_eq!(Some(kept), Kept::read(&cookies));
        for cookies in [
            "",
            "gallery-session=1.2",
            "gallery-session=a.b.c",
            "gallery-session=1.2.3.4",
        ] {
            assert_eq!(None, Kept::read(cookies), "{cookies:?}");
        }
    }

    #[test]
    fn activity_starts_the_idle_timeout_again_and_never_past_the_lifetime() {
        let kept = Kept::started(0, 900, 1000);
        assert_eq!(
            Some(Left {
                idle: 900,
                absolute: 1000
            }),
            kept.left(0)
        );
        let later = kept.extended(600_000);
        assert_eq!(1_000_000, later.idle_end, "capped at the lifetime");
        assert_eq!(None, later.left(1_000_000), "the session has ended");
    }
}
