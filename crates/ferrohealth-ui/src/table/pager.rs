// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The pagers of a listing.
//!
//! A pager states the range, then First, Previous, the position, Next and
//! Last, and the sizes; a step that cannot be taken stays where it is,
//! drawn as a disabled control, and says "(unavailable)" to assistive
//! technology (WCAG 2.2 SC 1.4.1, SC 4.1.2). A listing that keeps its state
//! in the URL pages by links ([`link_pager`]); one that keeps its position
//! out of the URL, because it can show patient data, pages by a `POST` form
//! whose buttons carry the page and the size ([`post_pager`]). Both work
//! with no script.

use std::borrow::Cow;

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::{Locale, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::table::model::{
    Noun, OpenWindow, PAGE_KEY, Paging, SIZE_KEY, SIZES, StateSource, Window, partial_note,
};
use crate::table::view::{hidden_fields, method};
use crate::view::button::Variant;

/// A step of the pager.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The first page.
    First,
    /// The page before.
    Previous,
    /// The page after.
    Next,
    /// The last page.
    Last,
}

impl Step {
    /// Every step, in the order the pager draws them around the position.
    pub const ALL: [Self; 4] = [Self::First, Self::Previous, Self::Next, Self::Last];

    /// The step's `data-page` value, a stable hook for a journey.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::Previous => "prev",
            Self::Next => "next",
            Self::Last => "last",
        }
    }

    /// The step's words in `locale`.
    #[must_use]
    pub fn label(self, locale: &Locale) -> Word {
        tr(
            locale,
            match self {
                Self::First => Text::StepFirst,
                Self::Previous => Text::StepPrevious,
                Self::Next => Text::StepNext,
                Self::Last => Text::StepLast,
            },
        )
    }

    /// The step's icon.
    #[must_use]
    pub const fn glyph(self) -> Glyph {
        match self {
            Self::First => Glyph::CHEVRONS_LEFT,
            Self::Previous => Glyph::CHEVRON_LEFT,
            Self::Next => Glyph::CHEVRON_RIGHT,
            Self::Last => Glyph::CHEVRONS_RIGHT,
        }
    }

    /// The page the step leads to from `window`, or `None` when it cannot be
    /// taken.
    #[must_use]
    pub const fn target(self, window: Window) -> Option<u32> {
        match self {
            Self::First | Self::Previous if !window.has_previous() => None,
            Self::Next | Self::Last if !window.has_next() => None,
            Self::First => Some(1),
            Self::Previous => Some(window.page - 1),
            Self::Next => Some(window.page + 1),
            Self::Last => Some(window.pages),
        }
    }
}

/// Returns the inside of a pager step's control: the icon beside the words,
/// the icon after them for a step forward, and "(unavailable)" as visually
/// hidden text when the step cannot be taken.
#[must_use]
pub fn step_label(locale: &Locale, step: Step, available: bool) -> Html {
    let forward = matches!(step, Step::Next | Step::Last);
    let glyph = icon(step.glyph(), "14");
    let words = Html::raw(format!(
        "<span class=\"fui-step-words\">{}</span>",
        step.label(locale)
    ));
    let unavailable = if available {
        Html::empty()
    } else {
        Html::raw(format!(
            "<span class=\"fui-sr-only\"> {}</span>",
            tr(locale, Text::StepUnavailable)
        ))
    };
    if forward {
        Html::join([words, unavailable, glyph])
    } else {
        Html::join([glyph, words, unavailable])
    }
}

/// What a pager says about the listing it pages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// The pager's accessible name, which tells two pagers on one screen
    /// apart.
    pub label: Cow<'static, str>,
    /// What the rows are, named in the range: "1-25 of 312 value sets".
    pub noun: Noun,
    /// The one-click page sizes; none when the size is a setting.
    pub sizes: &'static [u32],
    /// The rows the server counted, when it stated a count that the rows in
    /// hand may fall short of.
    pub stated: Option<u32>,
}

impl Listing {
    /// A listing named `label` of rows, offering 25, 50 and 100.
    #[must_use]
    pub fn new(label: impl Into<Cow<'static, str>>) -> Self {
        Self {
            label: label.into(),
            noun: Noun::ROWS,
            sizes: &SIZES,
            stated: None,
        }
    }

    /// This listing, its rows named `noun`.
    #[must_use]
    pub fn noun(self, noun: Noun) -> Self {
        Self { noun, ..self }
    }

    /// This listing, offering `sizes` (none when the size is a setting).
    #[must_use]
    pub fn sizes(self, sizes: &'static [u32]) -> Self {
        Self { sizes, ..self }
    }

    /// This listing, its server having counted `stated` rows.
    #[must_use]
    pub fn stated(self, stated: Option<u32>) -> Self {
        Self { stated, ..self }
    }
}

impl From<&'static str> for Listing {
    fn from(label: &'static str) -> Self {
        Self::new(label)
    }
}

impl From<String> for Listing {
    fn from(label: String) -> Self {
        Self::new(label)
    }
}

/// How a step or a size is drawn: a link, or a submit button of a form.
type StepDrawer<'a> = &'a dyn Fn(Step, Option<u32>) -> Html;
type SizeDrawer<'a> = &'a dyn Fn(u32, bool) -> Html;

/// Returns the size choices of `listing` around `current`, or nothing when
/// it offers none.
fn size_choices(locale: &Locale, listing: &Listing, current: u32, size: SizeDrawer<'_>) -> Html {
    if listing.sizes.is_empty() {
        return Html::empty();
    }
    let sizes: Html = listing
        .sizes
        .iter()
        .map(|choice| size(*choice, *choice == current))
        .collect();
    Html::raw(format!(
        "<div class=\"fui-pager-sizes\"><span class=\"fui-muted\">{}</span><span class=\"fui-pager-size-choices\">{sizes}</span></div>",
        tr(locale, Text::RowsPerPage)
    ))
}

#[derive(Template)]
#[template(
    source = r#"<nav class="fui-pager" aria-label="{{ label }}">{{ forms }}<p role="status">{{ range }}</p>{% if let Some(steps) = steps %}<div class="fui-pager-steps">{{ steps }}</div>{% endif %}{{ sizes }}</nav>"#,
    ext = "html"
)]
struct Frame {
    label: Cow<'static, str>,
    forms: Html,
    range: Word,
    steps: Option<Html>,
    sizes: Html,
}

/// Returns a pager over `window`, each step and size drawn by `step` and
/// `size`.
///
/// The range names the rows by the listing's noun, and a second sentence
/// says so when the server counted more than the listing holds.
#[must_use]
pub fn pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: Window,
    step: StepDrawer<'_>,
    size: SizeDrawer<'_>,
) -> Html {
    framed(locale, listing.into(), window, step, size, Html::empty())
}

fn framed(
    locale: &Locale,
    listing: Listing,
    window: Window,
    step: StepDrawer<'_>,
    size: SizeDrawer<'_>,
    forms: Html,
) -> Html {
    let [first, previous, next, last] = Step::ALL.map(|each| step(each, each.target(window)));
    let range = match partial_note(locale, window.total, listing.stated, listing.noun) {
        Some(note) => tr_with(
            locale,
            Text::RangeWithNote,
            &[
                ("range", window.range_of(locale, listing.noun).into()),
                ("note", note.into()),
            ],
        ),
        None => window.range_of(locale, listing.noun),
    };
    let sizes = size_choices(locale, &listing, window.size, size);
    // A listing of one page draws the range and the sizes alone.
    let steps = (window.pages > 1).then(|| {
        let position = tr_with(
            locale,
            Text::PageOf,
            &[("page", window.page.into()), ("pages", window.pages.into())],
        );
        Html::join([
            first,
            previous,
            Html::raw(format!(
                "<span class=\"fui-pager-position\">{position}</span>"
            )),
            next,
            last,
        ])
    });
    Frame {
        label: listing.label,
        forms,
        range,
        steps,
        sizes,
    }
    .html()
}

/// Returns the pager of a listing whose total the server does not report:
/// Previous and Next around the range, which names no total.
#[must_use]
pub fn open_pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: OpenWindow,
    step: StepDrawer<'_>,
    size: SizeDrawer<'_>,
) -> Html {
    framed_open(locale, listing.into(), window, step, size, Html::empty())
}

fn framed_open(
    locale: &Locale,
    listing: Listing,
    window: OpenWindow,
    step: StepDrawer<'_>,
    size: SizeDrawer<'_>,
    forms: Html,
) -> Html {
    let previous = step(
        Step::Previous,
        window.has_previous().then(|| window.page - 1),
    );
    let next = step(
        Step::Next,
        window.has_next().then(|| window.page.saturating_add(1)),
    );
    let sizes = size_choices(locale, &listing, window.size, size);
    Frame {
        label: listing.label,
        forms,
        range: window.range_of(locale, listing.noun),
        steps: Some(Html::join([previous, next])),
        sizes,
    }
    .html()
}

/// Returns the pager over `window` whose steps and sizes are links.
///
/// The href of each comes from `href`: it is the pager of a listing that
/// keeps its state in the URL.
#[must_use]
pub fn link_pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: Window,
    href: &dyn Fn(Paging) -> String,
) -> Html {
    let paging = window.paging();
    pager(
        locale,
        listing,
        window,
        &|each, target| link_step(locale, href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

/// Returns the pager over `window`, a listing with no stated total, whose
/// steps and sizes are links from `href`.
#[must_use]
pub fn link_open_pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: OpenWindow,
    href: &dyn Fn(Paging) -> String,
) -> Html {
    let paging = window.paging();
    open_pager(
        locale,
        listing,
        window,
        &|each, target| link_step(locale, href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

fn link_step(
    locale: &Locale,
    href: &dyn Fn(Paging) -> String,
    paging: Paging,
    each: Step,
    target: Option<u32>,
) -> Html {
    let class = Variant::Secondary.small();
    match target {
        Some(page) => Html::raw(format!(
            "<a href=\"{}\" class=\"{class}\" data-page=\"{}\">{}</a>",
            Html::text(href(paging.at(page))),
            each.key(),
            step_label(locale, each, true)
        )),
        None => Html::raw(format!(
            "<span class=\"{class}\" aria-disabled=\"true\" data-page=\"{}\">{}</span>",
            each.key(),
            step_label(locale, each, false)
        )),
    }
}

fn link_size(href: &dyn Fn(Paging) -> String, choice: u32, current: bool) -> Html {
    let here = if current {
        " aria-current=\"true\""
    } else {
        ""
    };
    Html::raw(format!(
        "<a href=\"{}\" class=\"fui-pager-size\"{here}>{choice}</a>",
        Html::text(href(Paging::sized(choice)))
    ))
}

/// Where a pager whose listing keeps its position out of the URL posts the
/// page and the size.
///
/// The console's route keeps the position in its
/// session and answers with that page. The pager's buttons submit the
/// fields `page` and `size` beside `fields`.
#[derive(Debug, Clone, Copy)]
pub struct PostTo<'a> {
    /// The name the pager's two forms are made from, unique on the page.
    pub id: &'a str,
    /// The URL the forms post to.
    pub action: &'a str,
    /// The other fields each form carries. A `page` or a `size` among them
    /// is left out.
    pub fields: &'a [(String, String)],
}

impl PostTo<'_> {
    fn steps(&self) -> String {
        format!("{}-steps", self.id)
    }

    fn sizes(&self) -> String {
        format!("{}-sizes", self.id)
    }

    /// The two forms the pager's buttons submit by their `form` attribute:
    /// hidden fields alone, drawn with no box.
    fn forms(&self, size: u32) -> Html {
        let carried = || hidden_fields(self.fields, &[PAGE_KEY, SIZE_KEY]);
        let posts = method(StateSource::Session);
        let action = Html::text(self.action);
        Html::raw(format!(
            "<form id=\"{}\" class=\"fui-pager-form\" method=\"{posts}\" action=\"{action}\">{}<input type=\"hidden\" name=\"{SIZE_KEY}\" value=\"{size}\"></form><form id=\"{}\" class=\"fui-pager-form\" method=\"{posts}\" action=\"{action}\">{}<input type=\"hidden\" name=\"{PAGE_KEY}\" value=\"1\"></form>",
            Html::text(self.steps()),
            carried(),
            Html::text(self.sizes()),
            carried()
        ))
    }
}

fn post_step(locale: &Locale, form: &str, each: Step, target: Option<u32>) -> Html {
    let available = target.is_some();
    let value = target.map_or(String::new(), |page| format!(" value=\"{page}\""));
    let disabled = if available { "" } else { " disabled" };
    Html::raw(format!(
        "<button type=\"submit\" form=\"{}\" name=\"{PAGE_KEY}\"{value} class=\"{}\" data-page=\"{}\"{disabled}>{}</button>",
        Html::text(form),
        Variant::Secondary.small(),
        each.key(),
        step_label(locale, each, available)
    ))
}

fn post_size(form: &str, choice: u32, current: bool) -> Html {
    Html::raw(format!(
        "<button type=\"submit\" form=\"{}\" name=\"{SIZE_KEY}\" value=\"{choice}\" class=\"fui-pager-size\" aria-pressed=\"{current}\">{choice}</button>",
        Html::text(form)
    ))
}

/// Returns the pager over `window` of a listing that keeps its position out
/// of the URL.
///
/// Each step and size is a submit button of a `POST` form to `post`, which
/// carries the page and the size, so no position reaches a URL or the
/// history.
#[must_use]
pub fn post_pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: Window,
    post: PostTo<'_>,
) -> Html {
    let (steps, sizes) = (post.steps(), post.sizes());
    framed(
        locale,
        listing.into(),
        window,
        &|each, target| post_step(locale, &steps, each, target),
        &|choice, current| post_size(&sizes, choice, current),
        post.forms(window.size),
    )
}

/// Returns the pager over `window`, a listing with no stated total that
/// keeps its position out of the URL, whose steps and sizes post to `post`.
#[must_use]
pub fn post_open_pager(
    locale: &Locale,
    listing: impl Into<Listing>,
    window: OpenWindow,
    post: PostTo<'_>,
) -> Html {
    let (steps, sizes) = (post.steps(), post.sizes());
    framed_open(
        locale,
        listing.into(),
        window,
        &|each, target| post_step(locale, &steps, each, target),
        &|choice, current| post_size(&sizes, choice, current),
        post.forms(window.size),
    )
}
