// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The toast region: short notices a page arrives with or a module raises.
//!
//! A toast is a title, an optional line under it and an optional action
//! under the text, beside its tone's icon, with a Dismiss button. A screen
//! reader reads it as it arrives: the region is a polite live region, and a
//! toast of the bad tone is an alert (WAI-ARIA 1.2, the `status` and
//! `alert` roles). The server draws the region with the [`Flash`] toasts
//! the console hands it, the outcome of a form post it redirected back
//! from, each with a Dismiss that is a native disclosure, so it closes
//! with no script. The `toast` module then times a toast without an action
//! out after [`DWELL_MS`], the count held while the pointer or the focus is
//! on the region (WCAG 2.2 SC 2.2.1), keeps one with an action until it is
//! dismissed, shows at most [`SHOWN_MAX`] and raises a toast any module asks
//! for with a `fui:toast` event, cloning the region's `<template>`; a link
//! marked [`RAISES`] to a page that answers with a toast raises that toast
//! in place of loading the page. A toast
//! holds no clinical data: it says what happened, never what was in it.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::tone::Tone;
use crate::view::button::Variant;

/// How long a toast without an action stays, in milliseconds.
pub const DWELL_MS: u64 = 6000;

/// The most toasts the region shows at once.
pub const SHOWN_MAX: usize = 3;

/// The attribute of a link whose page answers with a toast, naming the
/// toast's tone (`ok`, `info`, `warn`, `bad`); `data-fui-toast-title`,
/// `-line`, `-action` and `-href` carry its words and its action, and the
/// `toast` module raises it in place of following the link.
pub const RAISES: &str = "data-fui-toast";

/// A toast's action that leads somewhere: a link drawn as a small button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToastLink {
    /// What the action says, such as "Open History".
    pub label: String,
    /// Where it leads.
    pub href: String,
}

/// A toast the server draws as the page arrives, such as what a form post
/// the console redirected back from did (POST, redirect, GET).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Flash {
    /// Its tone; the bad tone is an alert.
    pub tone: Tone,
    /// What happened, in a few words.
    pub title: String,
    /// The line under the title, when there is more to say.
    pub line: Option<String>,
    /// The page to go on to, when there is one.
    pub action: Option<ToastLink>,
}

impl Flash {
    /// A toast of `tone` titled `title`, with no line and no action.
    #[must_use]
    pub fn new(tone: Tone, title: impl Into<String>) -> Self {
        Self {
            tone,
            title: title.into(),
            line: None,
            action: None,
        }
    }

    /// This toast with `line` under its title.
    #[must_use]
    pub fn line(self, line: impl Into<String>) -> Self {
        Self {
            line: Some(line.into()),
            ..self
        }
    }

    /// This toast with `action` under its text.
    #[must_use]
    pub fn action(self, action: ToastLink) -> Self {
        Self {
            action: Some(action),
            ..self
        }
    }
}

/// Takes the oldest toasts that are no refusal and have no action out of
/// `list` until it holds [`SHOWN_MAX`] or none of those is left to take.
fn trim(list: &mut Vec<Flash>) {
    while list.len() > SHOWN_MAX {
        let Some(oldest) = list
            .iter()
            .position(|flash| flash.action.is_none() && flash.tone != Tone::Bad)
        else {
            return;
        };
        list.remove(oldest);
    }
}

/// The icon of a toast of `tone`.
const fn glyph(tone: Tone) -> Glyph {
    match tone {
        Tone::Ok => Glyph::CIRCLE_CHECK,
        Tone::Warn => Glyph::TRIANGLE_ALERT,
        Tone::Bad => Glyph::CIRCLE_ALERT,
        Tone::Info | Tone::Neutral | Tone::Accent => Glyph::INFO,
    }
}

/// The role of a toast of `tone`: an alert for the bad tone, and none for
/// the others, which the polite region around them announces.
#[must_use]
pub const fn role(tone: Tone) -> Option<&'static str> {
    match tone {
        Tone::Bad => Some("alert"),
        Tone::Ok | Tone::Warn | Tone::Info | Tone::Neutral | Tone::Accent => None,
    }
}

struct ToastView {
    tone: &'static str,
    role: Option<&'static str>,
    glyph: Html,
    title: String,
    line: Option<String>,
    action: Option<ToastLink>,
}

#[derive(Template)]
#[template(path = "toast/region.html")]
struct RegionView {
    region: Word,
    dismiss: Word,
    x: Html,
    placeholder: Html,
    action_class: &'static str,
    glyphs: Vec<(&'static str, Html)>,
    toasts: Vec<ToastView>,
}

/// Returns the toast region, showing `flash` as the page arrives and every
/// toast a module raises after. A console draws it once in its shell.
#[must_use]
pub fn toast_region(locale: &Locale, mut flash: Vec<Flash>) -> Html {
    trim(&mut flash);
    RegionView {
        region: tr(locale, Text::ToastRegion),
        dismiss: tr(locale, Text::ToastDismiss),
        x: icon(Glyph::X, "16"),
        placeholder: icon(Glyph::INFO, "16"),
        action_class: Variant::Secondary.small(),
        glyphs: Tone::ALL
            .into_iter()
            .map(|tone| (tone.name(), icon(glyph(tone), "16")))
            .collect(),
        toasts: flash
            .into_iter()
            .map(|flash| ToastView {
                tone: flash.tone.class(),
                role: role(flash.tone),
                glyph: icon(glyph(flash.tone), "16"),
                title: flash.title,
                line: flash.line,
                action: flash.action,
            })
            .collect(),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Flash, SHOWN_MAX, toast_region, trim};
    use crate::i18n::Locale;
    use crate::tone::Tone;

    #[test]
    fn the_oldest_plain_toast_leaves_first_and_a_refusal_stays() {
        let mut list = vec![
            Flash::new(Tone::Bad, "refused"),
            Flash::new(Tone::Ok, "one"),
            Flash::new(Tone::Ok, "two"),
            Flash::new(Tone::Ok, "three"),
            Flash::new(Tone::Ok, "four"),
        ];
        trim(&mut list);
        assert_eq!(SHOWN_MAX, list.len());
        assert_eq!("refused", list[0].title);
        assert_eq!("three", list[1].title);
    }

    #[test]
    fn a_bad_toast_is_an_alert_with_a_native_dismiss() {
        let html = toast_region(&Locale::english(), vec![Flash::new(Tone::Bad, "Nope")]);
        let html = html.as_str();
        assert!(
            html.contains(r#"class="fui-toast fui-tone-bad" role="alert">"#),
            "{html}"
        );
        assert!(
            html.contains("<details open class=\"fui-toast-close\">"),
            "{html}"
        );
        assert!(html.contains("<template>"), "{html}");
    }
}
