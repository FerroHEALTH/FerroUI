// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The pagers of a listing, drawn as static views.
//!
//! A pager states the range, then First, Previous, the position, Next and
//! Last, and the sizes; a step that cannot be taken stays where it is,
//! drawn as a disabled control on the sunken ground, and says
//! "(unavailable)" to assistive technology in visually hidden text
//! (WCAG 2.2 SC 1.4.1, SC 4.1.2). A listing that keeps its state in the URL
//! pages by links ([`link_pager`]); one that keeps its position out of the
//! URL, because it can show patient data, pages by a `POST` form whose
//! buttons carry the page and the size ([`post_pager`]), so no position
//! reaches a URL or the history and the rows stay server HTML. Both work
//! with no script.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, Word, tr, tr_with};
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

    /// The step's words in the page's locale.
    #[must_use]
    pub fn label(self) -> Word {
        tr(match self {
            Self::First => Text::StepFirst,
            Self::Previous => Text::StepPrevious,
            Self::Next => Text::StepNext,
            Self::Last => Text::StepLast,
        })
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
/// the icon after them for a step forward.
///
/// A step that cannot be taken adds "(unavailable)" as visually hidden
/// text; the control's disabled look shows it to the eye.
#[must_use]
pub fn step_label(step: Step, available: bool) -> impl IntoView + use<> {
    let forward = matches!(step, Step::Next | Step::Last);
    view! {
        {(!forward).then(|| icon(step.glyph(), "14"))}
        <span class="fui-step-words">{step.label()}</span>
        {(!available)
            .then(|| view! { <span class="fui-sr-only">{tr(Text::StepUnavailable).map(|words| format!(" {words}"))}</span> })}
        {forward.then(|| icon(step.glyph(), "14"))}
    }
}

/// What a pager says about the listing it pages.
///
/// The label is a fixed word or one the console looks up at run time; a
/// listing is `Clone`, and built where it is drawn.
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
    /// hand may fall short of (a FHIR search's `Bundle.total`).
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

/// Returns the size choices of `listing` around `current`, or nothing when
/// it offers none.
fn size_choices(listing: &Listing, current: u32, size: &dyn Fn(u32, bool) -> AnyView) -> AnyView {
    if listing.sizes.is_empty() {
        return ().into_any();
    }
    let sizes = listing
        .sizes
        .iter()
        .map(|choice| size(*choice, *choice == current))
        .collect_view();
    view! {
        <div class="fui-pager-sizes">
            <span class="fui-muted">{tr(Text::RowsPerPage)}</span>
            <span class="fui-pager-size-choices">{sizes}</span>
        </div>
    }
    .into_any()
}

/// Returns a pager over `window`, each step and size drawn by `step` and
/// `size` (a link for URL state, a submit button for session state).
///
/// The range names the rows by the listing's noun, and a second sentence
/// says so when the server counted more than the listing holds.
#[must_use]
pub fn pager(
    listing: impl Into<Listing>,
    window: Window,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
) -> AnyView {
    framed(listing.into(), window, step, size, ().into_any())
}

/// The pager [`pager`] draws, with `forms` (the forms its buttons submit)
/// inside its frame.
fn framed(
    listing: Listing,
    window: Window,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
    forms: AnyView,
) -> AnyView {
    let [first, previous, next, last] = Step::ALL.map(|each| step(each, each.target(window)));
    let range = match partial_note(window.total, listing.stated, listing.noun) {
        Some(note) => tr_with(
            Text::RangeWithNote,
            &[
                ("range", window.range_of(listing.noun).into()),
                ("note", note.into()),
            ],
        ),
        None => window.range_of(listing.noun),
    };
    let sizes = size_choices(&listing, window.size, size);
    // A listing of one page draws the range and the sizes alone: no step can
    // be taken, so none needs saying.
    let steps = (window.pages > 1).then(|| {
        view! {
            <div class="fui-pager-steps">
                {first} {previous}
                <span class="fui-pager-position">
                    {tr_with(
                        Text::PageOf,
                        &[("page", window.page.into()), ("pages", window.pages.into())],
                    )}
                </span> {next} {last}
            </div>
        }
    });
    view! {
        <nav class="fui-pager" aria-label=listing.label>
            {forms}
            <p role="status">{range}</p>
            {steps}
            {sizes}
        </nav>
    }
    .into_any()
}

/// Returns the pager of a listing whose total the server does not report:
/// Previous and Next around the range, which names no total ("Rows 26 to
/// 50"), each step and size drawn by `step` and `size`.
#[must_use]
pub fn open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
) -> AnyView {
    framed_open(listing.into(), window, step, size, ().into_any())
}

/// The pager [`open_pager`] draws, with `forms` inside its frame.
fn framed_open(
    listing: Listing,
    window: OpenWindow,
    step: &dyn Fn(Step, Option<u32>) -> AnyView,
    size: &dyn Fn(u32, bool) -> AnyView,
    forms: AnyView,
) -> AnyView {
    let previous = step(
        Step::Previous,
        window.has_previous().then(|| window.page - 1),
    );
    let next = step(
        Step::Next,
        window.has_next().then(|| window.page.saturating_add(1)),
    );
    let sizes = size_choices(&listing, window.size, size);
    view! {
        <nav class="fui-pager" aria-label=listing.label>
            {forms}
            <p role="status">{window.range_of(listing.noun)}</p>
            <div class="fui-pager-steps">{previous} {next}</div>
            {sizes}
        </nav>
    }
    .into_any()
}

/// Returns the pager over `window` whose steps and sizes are links, the
/// href of each from `href`: the pager of a listing that keeps its state in
/// the URL, working before the page hydrates.
#[must_use]
pub fn link_pager(
    listing: impl Into<Listing>,
    window: Window,
    href: &dyn Fn(Paging) -> String,
) -> AnyView {
    let paging = window.paging();
    pager(
        listing,
        window,
        &|each, target| link_step(href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

/// Returns the pager over `window`, a listing with no stated total, whose
/// steps and sizes are links from `href`.
#[must_use]
pub fn link_open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    href: &dyn Fn(Paging) -> String,
) -> AnyView {
    let paging = window.paging();
    open_pager(
        listing,
        window,
        &|each, target| link_step(href, paging, each, target),
        &|choice, current| link_size(href, choice, current),
    )
}

/// Returns a pager step as a link to its page, or as a span that says
/// "(unavailable)" when it cannot be taken.
fn link_step(
    href: &dyn Fn(Paging) -> String,
    paging: Paging,
    each: Step,
    target: Option<u32>,
) -> AnyView {
    match target {
        Some(page) => view! {
            <a href=href(paging.at(page)) class=Variant::Secondary.small() data-page=each.key()>
                {step_label(each, true)}
            </a>
        }
        .into_any(),
        None => view! {
            <span class=Variant::Secondary.small() aria-disabled="true" data-page=each.key()>
                {step_label(each, false)}
            </span>
        }
        .into_any(),
    }
}

/// Returns a size choice as a link to the first page at that size.
fn link_size(href: &dyn Fn(Paging) -> String, choice: u32, current: bool) -> AnyView {
    view! {
        <a
            href=href(Paging::sized(choice))
            class="fui-pager-size"
            aria-current=current.then_some("true")
        >
            {choice.to_string()}
        </a>
    }
    .into_any()
}

/// Where a pager whose listing keeps its position out of the URL posts the
/// page and the size: the console's route or server function, which keeps
/// the position (in its session) and answers with that page.
///
/// The library defines no handler. The pager's buttons submit the fields
/// `page` and `size` ([`PAGE_KEY`], [`SIZE_KEY`]) beside `fields`, so the
/// console's handler takes arguments of those names.
#[derive(Debug, Clone, Copy)]
pub struct PostTo<'a> {
    /// The name the pager's two forms are made from, unique on the page.
    pub id: &'a str,
    /// The URL the forms post to.
    pub action: &'a str,
    /// The other fields each form carries, such as which listing it pages
    /// and the page to return to. A `page` or a `size` among them is left
    /// out.
    pub fields: &'a [(String, String)],
}

impl PostTo<'_> {
    /// The id of the form a step submits, which carries the size.
    fn steps(&self) -> String {
        format!("{}-steps", self.id)
    }

    /// The id of the form a size submits, which starts at the first page.
    fn sizes(&self) -> String {
        format!("{}-sizes", self.id)
    }

    /// The two forms the pager's buttons submit by their `form` attribute:
    /// hidden fields alone, drawn with no box, so the pager keeps its layout
    /// (HTML Living Standard, form-associated elements).
    fn forms(&self, size: u32) -> AnyView {
        let carried = || hidden_fields(self.fields, &[PAGE_KEY, SIZE_KEY]);
        let action = self.action.to_owned();
        let posts = method(StateSource::Session);
        view! {
            <form id=self.steps() class="fui-pager-form" method=posts action=action.clone()>
                {carried()}
                <input type="hidden" name=SIZE_KEY value=size.to_string() />
            </form>
            <form id=self.sizes() class="fui-pager-form" method=posts action=action>
                {carried()}
                <input type="hidden" name=PAGE_KEY value="1" />
            </form>
        }
        .into_any()
    }
}

/// Returns a pager step as a submit button of the steps form `form` that
/// carries its page, or as a disabled button that says "(unavailable)"
/// when it cannot be taken.
fn post_step(form: &str, each: Step, target: Option<u32>) -> AnyView {
    let available = target.is_some();
    view! {
        <button
            type="submit"
            form=form.to_owned()
            name=PAGE_KEY
            value=target.map(|page| page.to_string())
            class=Variant::Secondary.small()
            data-page=each.key()
            disabled=!available
        >
            {step_label(each, available)}
        </button>
    }
    .into_any()
}

/// Returns a size choice as a submit button of the sizes form `form` that
/// carries the size, pressed when it is the size in force.
fn post_size(form: &str, choice: u32, current: bool) -> AnyView {
    view! {
        <button
            type="submit"
            form=form.to_owned()
            name=SIZE_KEY
            value=choice.to_string()
            class="fui-pager-size"
            aria-pressed=current.to_string()
        >
            {choice.to_string()}
        </button>
    }
    .into_any()
}

/// Returns the pager over `window` of a listing that keeps its position out
/// of the URL: each step and size a submit button of a `POST` form to
/// `post`, which carries the page and the size.
///
/// The position never reaches a URL or the history, the rows stay server
/// HTML (no row is an island prop), and it pages with no script.
#[must_use]
pub fn post_pager(listing: impl Into<Listing>, window: Window, post: PostTo<'_>) -> AnyView {
    let (steps, sizes) = (post.steps(), post.sizes());
    framed(
        listing.into(),
        window,
        &|each, target| post_step(&steps, each, target),
        &|choice, current| post_size(&sizes, choice, current),
        post.forms(window.size),
    )
}

/// Returns the pager over `window`, a listing with no stated total that
/// keeps its position out of the URL, whose steps and sizes post to `post`.
#[must_use]
pub fn post_open_pager(
    listing: impl Into<Listing>,
    window: OpenWindow,
    post: PostTo<'_>,
) -> AnyView {
    let (steps, sizes) = (post.steps(), post.sizes());
    framed_open(
        listing.into(),
        window,
        &|each, target| post_step(&steps, each, target),
        &|choice, current| post_size(&sizes, choice, current),
        post.forms(window.size),
    )
}
