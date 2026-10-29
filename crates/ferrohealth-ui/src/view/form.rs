// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The fields: a label above its control, the help under it, and the
//! checkbox with its label on one row.
//!
//! The control itself is the console's, because its value and listeners
//! are: a view here gives it the classes and the label and help their
//! associations. Every control carries an explicit, stable `id` and `name`
//! (no random id differs between the server pass and hydration), and a
//! control with help names it with `aria-describedby` set to [`help_id`].

use leptos::prelude::*;

use crate::i18n::Word;
use crate::island::HoldToShow;
use crate::widget::masked::HoldWords;

/// The classes of a text input.
pub const INPUT: &str = "fui-input";

/// The classes of a select.
pub const SELECT: &str = "fui-select";

/// The classes of a text area in the code face.
pub const TEXTAREA: &str = "fui-textarea";

/// The classes of a text area for prose, such as a definition: the body
/// face, two rows high unless the console sets `rows`.
pub const PROSE: &str = "fui-textarea fui-textarea-prose";

/// The width class of a control that holds a number or a short code, which
/// never takes the full row; add it to the control's classes.
pub const NARROW: &str = "fui-control-narrow";

/// The width class of a control that holds a name or a version.
pub const MEDIUM: &str = "fui-control-medium";

/// The classes of a checkbox drawn as a switch.
///
/// The knob's place says on or off as well as its colour (WCAG 2.2
/// SC 1.4.1); give the input `role="switch"` and use it where
/// [`check_field`] takes a checkbox.
pub const SWITCH: &str = "fui-switch";

/// The classes of a form laid out as one column of fields.
pub const FORM: &str = "fui-form";

/// The classes of a row of fields and their buttons.
///
/// The fields share the row's width, each growing from 10rem so a
/// placeholder is not cut, the buttons keep their own, and below 30rem each
/// field and button takes the whole row.
pub const FIELD_FLOW: &str = "fui-field-flow";

/// The id of the help line of the control `id`, which the control names in
/// `aria-describedby`.
#[must_use]
pub fn help_id(id: &str) -> String {
    format!("{id}-help")
}

/// The id of the error line of the control `id`, which the control names in
/// `aria-describedby` after its help.
#[must_use]
pub fn error_id(id: &str) -> String {
    format!("{id}-error")
}

/// The value of the `aria-describedby` of the control `id`: its help line
/// ([`help_id`]) when `help`, then its error line ([`error_id`]) when
/// `error`, or nothing when it has neither.
///
/// The order is the reading order, the help before the refusal
/// (WAI-ARIA 1.2, `aria-describedby`).
#[must_use]
pub fn described_by(id: &str, help: bool, error: bool) -> Option<String> {
    match (help, error) {
        (true, true) => Some(format!("{} {}", help_id(id), error_id(id))),
        (true, false) => Some(help_id(id)),
        (false, true) => Some(error_id(id)),
        (false, false) => None,
    }
}

/// The id of the unit suffix of the control `id` ([`Inner::Unit`]), which
/// the control names in `aria-describedby`.
#[must_use]
pub fn unit_id(id: &str) -> String {
    format!("{id}-unit")
}

/// The value of the `aria-describedby` of the control `id` that has a unit
/// suffix: the unit ([`unit_id`]) first, then its help and its error line
/// as [`described_by`] names them.
///
/// The unit is read with the value it qualifies, so it comes first
/// (WAI-ARIA 1.2, `aria-describedby`; WCAG 2.2 SC 1.3.1).
#[must_use]
pub fn described_with_unit(id: &str, help: bool, error: bool) -> String {
    match described_by(id, help, error) {
        Some(rest) => format!("{} {rest}", unit_id(id)),
        None => unit_id(id),
    }
}

/// The classes of a button inside a field's edge ([`Inner::Button`]): a
/// square 30 px target with an icon and its name in `aria-label`
/// (WCAG 2.2 SC 2.5.8).
pub const INNER_BUTTON: &str = "fui-icon-button fui-input-toggle";

/// A part inside a field's edge, before or after the typed text (R10): the
/// input and its parts share one border and one focus ring, and the text
/// never runs under a part.
#[derive(Debug)]
pub enum Inner {
    /// Words before or after the typed text that frame it, such as a
    /// scheme before an address.
    Text(String),
    /// The unit of a quantity, after it, such as "ms" or "mm Hg". It has
    /// the id [`unit_id`], and the control names it in `aria-describedby`
    /// ([`described_with_unit`]), so the unit is read with the value.
    Unit(String),
    /// A button, such as a clear or the date picker's: give it
    /// [`INNER_BUTTON`] and a name.
    Button(AnyView),
    /// The eye of a masked input ([`masked_input`]), after it: held by the
    /// pointer, by touch or by Space or Enter, it shows what was typed, and
    /// released, it masks it again ([`crate::island::HoldToShow`]). Until
    /// the page's WebAssembly runs it is disabled and says the field stays
    /// masked.
    HoldToShow(HoldWords),
}

impl Inner {
    /// Returns the part, its words in the prefix's or the suffix's place.
    fn view(self, id: &str, leading: bool) -> AnyView {
        let class = if leading {
            "fui-input-prefix"
        } else {
            "fui-input-suffix"
        };
        match self {
            Self::Text(words) => view! { <span class=class>{words}</span> }.into_any(),
            Self::Unit(words) => view! {
                <span id=unit_id(id) class=class>
                    {words}
                </span>
            }
            .into_any(),
            Self::Button(button) => button,
            Self::HoldToShow(words) => {
                view! { <HoldToShow input=id.to_owned() words /> }.into_any()
            }
        }
    }
}

/// Returns the masked text input `id` named `name` (R10), described by
/// `described_by` when it has help or an error line.
///
/// It is `type="password"`, so what is typed shows as dots, with no
/// autocomplete, no capitals and no spelling check, as for an identifier.
///
/// It carries no value: what is typed stays in the input and posts with its
/// form. Put [`Inner::HoldToShow`] after it to show the text while held. In
/// an answer form ([`crate::widget::action::answer_field`]), draw
/// [`crate::widget::action::secret_mark`] for it too, so the page that
/// answers the post never carries the value back.
///
/// ```
/// use ferrohealth_ui::view::form::{Field, Inner, described_by, masked_input};
/// use ferrohealth_ui::widget::masked::HoldWords;
/// use leptos::prelude::*;
///
/// let html = Owner::new().with(|| {
///     Field::new("identifier", "Identifier")
///         .help("Nine digits.")
///         .trailing(Inner::HoldToShow(HoldWords::of_page().named("Show the identifier while held")))
///         .view(masked_input("identifier", "identifier", described_by("identifier", true, false)))
///         .to_html()
/// });
/// assert!(html.contains(r#"type="password""#));
/// assert!(html.contains("Show the text") || html.contains("stays masked"));
/// ```
#[must_use]
pub fn masked_input(id: &str, name: &str, described_by: Option<String>) -> AnyView {
    view! {
        <input
            id=id.to_owned()
            name=name.to_owned()
            type="password"
            class=INPUT
            autocomplete="off"
            autocapitalize="off"
            spellcheck="false"
            aria-describedby=described_by
        />
    }
    .into_any()
}

/// The classes of a field's error line: the help's size in the refusal's
/// colour.
///
/// The line is the console's, because what fills it and when are: give it
/// the id [`error_id`], put it in the field's trailing slot
/// ([`Field::after`]), and name it in the control's `aria-describedby`
/// ([`described_by`]).
pub const ERROR: &str = "fui-help fui-error";

/// A field drawn piece by piece: its label, the control, its help line and
/// a trailing line after the help.
///
/// The label shows above the control or is visually hidden, the control
/// can hold a part inside its edge before or after the typed text
/// ([`Inner`]), and the trailing line is such as the one a refusal shows
/// in.
///
/// [`field`] and [`keyed_field`] draw the common case; this is the shape
/// they share.
///
/// ```
/// use ferrohealth_ui::view::form::{ERROR, Field, INPUT, described_by, error_id};
/// use leptos::prelude::*;
///
/// /// The base URL field of the member `member`.
/// fn base_url(member: &str) -> AnyView {
///     let id = String::from("member-base");
///     let input = view! {
///         <input id=id.clone() name="base" class=INPUT aria-describedby=described_by(&id, true, true) />
///     };
///     Field::new(id.clone(), format!("Base URL of {member}"))
///         .help("Where the gateway reaches the member.")
///         .after(view! { <p id=error_id(&id) class=ERROR aria-live="polite"></p> })
///         .view(input)
/// }
///
/// let html = Owner::new().with(|| base_url("node-a").to_html());
/// assert!(html.contains("Base URL of node-a"));
/// ```
#[derive(Debug)]
#[must_use]
pub struct Field {
    id: String,
    label: Word,
    label_hidden: bool,
    holds_id: bool,
    help: Option<Word>,
    after: Option<AnyView>,
    leading: Option<Inner>,
    trailing: Option<Inner>,
}

impl Field {
    /// The field of the control `id`, labelled `label`, with no help.
    pub fn new(id: impl Into<String>, label: impl Into<Word>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            label_hidden: false,
            holds_id: false,
            help: None,
            after: None,
            leading: None,
            trailing: None,
        }
    }

    /// Adds the help line `help` under the control, its id [`help_id`].
    pub fn help(mut self, help: impl Into<Word>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Adds the help line `help` when there is one.
    pub fn help_if(mut self, help: Option<impl Into<Word>>) -> Self {
        self.help = help.map(Into::into);
        self
    }

    /// Hides the label from sight and keeps it for assistive technology,
    /// still bound to the control with `for` (WCAG 2.2 SC 1.3.1 and
    /// SC 4.1.2, technique C7): a row of a list of values, each an input
    /// and its remove button, where a visible label over every row is noise.
    pub fn label_hidden(mut self) -> Self {
        self.label_hidden = true;
        self
    }

    /// Widens the field for a whole id in an operation's options row
    /// (`is-id`, [`crate::view::transform`]): it starts at 24 rem, wide
    /// enough for a uuid or a versioned object uid, and takes the rest of
    /// the row. A field for another long token, such as a list of scopes,
    /// takes it too.
    pub fn holds_id(mut self) -> Self {
        self.holds_id = true;
        self
    }

    /// Adds `line` after the help, such as the control's error line
    /// ([`ERROR`]).
    pub fn after(mut self, line: impl IntoView + 'static) -> Self {
        self.after = Some(line.into_any());
        self
    }

    /// Puts `part` inside the field's edge before the typed text (R10).
    pub fn leading(mut self, part: Inner) -> Self {
        self.leading = Some(part);
        self
    }

    /// Puts `part` inside the field's edge after the typed text (R10).
    pub fn trailing(mut self, part: Inner) -> Self {
        self.trailing = Some(part);
        self
    }

    /// Returns the field around `control`.
    pub fn view(self, control: impl IntoView + 'static) -> AnyView {
        let Self {
            id,
            label,
            label_hidden,
            holds_id,
            help,
            after,
            leading,
            trailing,
        } = self;
        let control = if leading.is_none() && trailing.is_none() {
            control.into_any()
        } else {
            let leading = leading.map(|part| part.view(&id, true));
            let trailing = trailing.map(|part| part.view(&id, false));
            view! { <div class="fui-input-group">{leading}{control}{trailing}</div> }.into_any()
        };
        let help = help.map(|help| view! { <p id=help_id(&id) class="fui-help">{help}</p> });
        let class = if label_hidden {
            "fui-sr-only"
        } else {
            "fui-label"
        };
        view! {
            <div class="fui-field" class:is-id=holds_id>
                <label for=id class=class>
                    {label}
                </label>
                {control}
                {help}
                {after}
            </div>
        }
        .into_any()
    }
}

/// Returns the field of the control `id`: the label `label` above
/// `control`, and the line `help` under it when there is one.
///
/// The label and the help are a `&'static str` or a `String` made at run
/// time alike. A
/// field drawn once per row of a list takes its id from [`keyed_field`];
/// a hidden label or a trailing line is [`Field`]'s.
#[must_use]
pub fn field(
    id: &'static str,
    label: impl Into<Word>,
    help: Option<String>,
    control: impl IntoView + 'static,
) -> impl IntoView {
    keyed_field(id.to_owned(), label, help, control)
}

/// Returns [`field`] for a control whose id is made at run time, such as one
/// per row of a list, so each label names its own control and the help id
/// stays in step with it.
#[must_use]
pub fn keyed_field(
    id: String,
    label: impl Into<Word>,
    help: Option<String>,
    control: impl IntoView + 'static,
) -> impl IntoView {
    Field::new(id, label).help_if(help).view(control)
}

/// Returns the action `button` with the line `error` under it where its
/// refusal shows: a field with no label, such as the button that adds a
/// row to a list.
///
/// The button names the line in `aria-describedby` ([`error_id`]), and the
/// line takes [`ERROR`].
#[must_use]
pub fn action_field(button: impl IntoView + 'static, error: impl IntoView + 'static) -> AnyView {
    let button = button.into_any();
    let error = error.into_any();
    view! {
        <div class="fui-field">
            <div class="fui-cluster">{button}</div>
            {error}
        </div>
    }
    .into_any()
}

/// Returns the related controls `body` under the legend `legend`, the
/// browser's fieldset frame reset to the card's look (HTML Living Standard,
/// the `fieldset` element; WCAG 2.2 SC 1.3.1).
#[must_use]
pub fn fieldset(legend: impl Into<String>, body: impl IntoView + 'static) -> impl IntoView {
    let legend = legend.into();
    let body = body.into_any();
    view! {
        <fieldset class="fui-fieldset">
            <legend>{legend}</legend>
            {body}
        </fieldset>
    }
}

/// Returns the fields `body` in pairs from 40 rem up, one above the other
/// below it.
#[must_use]
pub fn columns(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class="fui-columns">{body}</div> }
}

/// Returns the fields and buttons `body` in one row that shares its width
/// ([`FIELD_FLOW`]).
#[must_use]
pub fn field_flow(body: impl IntoView + 'static) -> impl IntoView {
    let body = body.into_any();
    view! { <div class=FIELD_FLOW>{body}</div> }
}

/// Returns the checkbox `checkbox`, whose id is `id`, beside its label
/// `label` on one row, the box clear of the words.
#[must_use]
pub fn check(id: &'static str, label: impl Into<Word>, checkbox: impl IntoView) -> impl IntoView {
    keyed_check(id.to_owned(), label, checkbox)
}

/// Returns [`check`] for a checkbox whose id is made at run time, such as
/// one per row of a list.
#[must_use]
pub fn keyed_check(id: String, label: impl Into<Word>, checkbox: impl IntoView) -> impl IntoView {
    let label: Word = label.into();
    view! {
        <div class="fui-check">
            {checkbox} <label for=id>{label}</label>
        </div>
    }
}

/// Returns the checkbox `checkbox` beside its label with the line `help`
/// under the words.
///
/// The row is [`check`]'s; the checkbox, whose id is `id`, names the help
/// with `aria-describedby` set to [`help_id`].
#[must_use]
pub fn check_field(
    id: &'static str,
    label: impl Into<Word>,
    help: impl Into<Word>,
    checkbox: impl IntoView + 'static,
) -> AnyView {
    keyed_check_field(id.to_owned(), label, help, checkbox)
}

/// Returns [`check_field`] for a checkbox whose id is made at run time, such
/// as one per row of a list, the help id kept in step with it.
#[must_use]
pub fn keyed_check_field(
    id: String,
    label: impl Into<Word>,
    help: impl Into<Word>,
    checkbox: impl IntoView + 'static,
) -> AnyView {
    let (label, help): (Word, Word) = (label.into(), help.into());
    let checkbox = checkbox.into_any();
    let help_id = help_id(&id);
    view! {
        <div class="fui-check">
            {checkbox}
            <span class="fui-check-text">
                <label for=id>{label}</label>
                <span id=help_id class="fui-help">
                    {help}
                </span>
            </span>
        </div>
    }
    .into_any()
}

/// Returns a group of checkboxes named `name` under the legend `legend`,
/// each drawn as a chip.
///
/// A ticked chip takes the accent's edge, so the tick and the edge both say
/// so: a choice of several, such as the nodes a query asks.
#[must_use]
pub fn chip_choices(
    name: &'static str,
    legend: impl Into<String>,
    options: Vec<(String, String)>,
    checked: &[String],
) -> AnyView {
    let legend = legend.into();
    let chips = options
        .into_iter()
        .map(|(value, words)| {
            let id = format!("{name}-{value}");
            let label_for = id.clone();
            let on = checked.contains(&value);
            view! {
                <label for=label_for class="fui-chip-check">
                    <input type="checkbox" id=id name=name value=value checked=on />
                    {words}
                </label>
            }
        })
        .collect_view();
    view! {
        <fieldset class="fui-fieldset fui-choice">
            <legend class="fui-label">{legend}</legend>
            <div class="fui-chip-checks">{chips}</div>
        </fieldset>
    }
    .into_any()
}
