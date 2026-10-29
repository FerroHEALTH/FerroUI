// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The fields: a label above its control, the help under it, and the
//! checkbox with its label on one row.
//!
//! The control itself is the console's, because its value and name are: a
//! view here gives it the classes and the label and help their
//! associations. Every control carries an explicit, stable `id` and `name`,
//! and a control with help names it with `aria-describedby` set to
//! [`help_id`].

use askama::Template;

use crate::html::{Html, Render as _};
use crate::i18n::Word;

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
/// `error`, or nothing when it has neither (WAI-ARIA 1.2,
/// `aria-describedby`).
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
/// as [`described_by`] names them (WCAG 2.2 SC 1.3.1).
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
    Button(Html),
}

impl Inner {
    /// Returns the part, its words in the prefix's or the suffix's place.
    fn view(self, id: &str, leading: bool) -> Html {
        let class = if leading {
            "fui-input-prefix"
        } else {
            "fui-input-suffix"
        };
        match self {
            Self::Text(words) => Html::raw(format!(
                "<span class=\"{class}\">{}</span>",
                Html::text(words)
            )),
            Self::Unit(words) => Html::raw(format!(
                "<span id=\"{}\" class=\"{class}\">{}</span>",
                Html::text(unit_id(id)),
                Html::text(words)
            )),
            Self::Button(button) => button,
        }
    }
}

#[derive(Template)]
#[template(
    source = r#"<input id="{{ id }}" name="{{ name }}" type="password" class="fui-input" autocomplete="off" autocapitalize="off" spellcheck="false"{% if let Some(by) = described_by %} aria-describedby="{{ by }}"{% endif %}>"#,
    ext = "html"
)]
struct Masked {
    id: String,
    name: String,
    described_by: Option<String>,
}

/// Returns the masked text input `id` named `name` (R10).
///
/// It is described by `described_by` when it has help or an error line,
/// and is `type="password"`, so what is typed shows as dots, with no
/// autocomplete, no capitals and no spelling check, as for an identifier.
/// It carries no value: what is typed stays in the input and posts with its
/// form. A page that answers the post draws the field empty again
/// ([`crate::view::posted`]), so the value never comes back to the page.
#[must_use]
pub fn masked_input(id: &str, name: &str, described_by: Option<String>) -> Html {
    Masked {
        id: id.to_owned(),
        name: name.to_owned(),
        described_by,
    }
    .html()
}

/// The classes of a field's error line: the help's size in the refusal's
/// colour.
///
/// The line is the console's: give it the id [`error_id`], put it
/// in the field's trailing slot ([`Field::after`]), and name it in the
/// control's `aria-describedby` ([`described_by`]).
pub const ERROR: &str = "fui-help fui-error";

/// A field drawn piece by piece: its label, the control, its help line and
/// a trailing line after the help.
///
/// ```
/// use ferrohealth_ui::html::Html;
/// use ferrohealth_ui::view::form::{ERROR, Field, INPUT, described_by, error_id};
///
/// let id = "member-base";
/// let input = Html::raw(format!(
///     r#"<input id="{id}" name="base" class="{INPUT}" aria-describedby="{}">"#,
///     described_by(id, true, true).unwrap_or_default()
/// ));
/// let html = Field::new(id, "Base URL of node-a")
///     .help("Where the gateway reaches the member.")
///     .after(Html::raw(format!(r#"<p id="{}" class="{ERROR}" aria-live="polite"></p>"#, error_id(id))))
///     .view(input);
/// assert!(html.as_str().contains("Base URL of node-a"));
/// ```
#[derive(Debug)]
#[must_use]
pub struct Field {
    id: String,
    label: Word,
    label_hidden: bool,
    holds_id: bool,
    help: Option<Word>,
    after: Option<Html>,
    leading: Option<Inner>,
    trailing: Option<Inner>,
}

#[derive(Template)]
#[template(
    source = r#"<div class="fui-field{% if holds_id %} is-id{% endif %}"><label for="{{ id }}" class="{{ label_class }}">{{ label }}</label>{{ control }}{% if let Some(help) = help %}<p id="{{ help_id }}" class="fui-help">{{ help }}</p>{% endif %}{% if let Some(after) = after %}{{ after }}{% endif %}</div>"#,
    ext = "html"
)]
struct FieldView {
    id: String,
    holds_id: bool,
    label_class: &'static str,
    label: Word,
    control: Html,
    help_id: String,
    help: Option<Word>,
    after: Option<Html>,
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
    /// SC 4.1.2, technique C7).
    pub fn label_hidden(mut self) -> Self {
        self.label_hidden = true;
        self
    }

    /// Widens the field for a whole id in an operation's options row
    /// (`is-id`): it starts at 24 rem and takes the rest of the row.
    pub fn holds_id(mut self) -> Self {
        self.holds_id = true;
        self
    }

    /// Adds `line` after the help, such as the control's error line
    /// ([`ERROR`]).
    pub fn after(mut self, line: Html) -> Self {
        self.after = Some(line);
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
    #[must_use]
    pub fn view(self, control: Html) -> Html {
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
            control
        } else {
            let leading = leading.map(|part| part.view(&id, true)).unwrap_or_default();
            let trailing = trailing
                .map(|part| part.view(&id, false))
                .unwrap_or_default();
            Html::raw(format!(
                "<div class=\"fui-input-group\">{leading}{control}{trailing}</div>"
            ))
        };
        FieldView {
            help_id: help_id(&id),
            id,
            holds_id,
            label_class: if label_hidden {
                "fui-sr-only"
            } else {
                "fui-label"
            },
            label,
            control,
            help,
            after,
        }
        .html()
    }
}

/// Returns the field of the control `id`: the label `label` above
/// `control`, and the line `help` under it when there is one.
#[must_use]
pub fn field(
    id: &'static str,
    label: impl Into<Word>,
    help: Option<String>,
    control: Html,
) -> Html {
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
    control: Html,
) -> Html {
    Field::new(id, label).help_if(help).view(control)
}

/// Returns the action `button` with the line `error` under it where its
/// refusal shows.
///
/// It is a field with no label, such as the button that adds a
/// row to a list. The button names the line in `aria-describedby`
/// ([`error_id`]), and the line takes [`ERROR`].
#[must_use]
pub fn action_field(button: Html, error: Html) -> Html {
    let (button, error) = (String::from(button), String::from(error));
    Html::raw(format!(
        "<div class=\"fui-field\"><div class=\"fui-cluster\">{button}</div>{error}</div>"
    ))
}

/// Returns the related controls `body` under the legend `legend`, the
/// browser's fieldset frame reset to the card's look (HTML Living Standard,
/// the `fieldset` element; WCAG 2.2 SC 1.3.1).
#[must_use]
pub fn fieldset(legend: impl Into<String>, body: Html) -> Html {
    let body = String::from(body);
    Html::raw(format!(
        "<fieldset class=\"fui-fieldset\"><legend>{}</legend>{body}</fieldset>",
        Html::text(legend.into())
    ))
}

/// Returns the fields `body` in pairs from 40 rem up, one above the other
/// below it.
#[must_use]
pub fn columns(body: Html) -> Html {
    let body = String::from(body);
    Html::raw(format!("<div class=\"fui-columns\">{body}</div>"))
}

/// Returns the fields and buttons `body` in one row that shares its width
/// ([`FIELD_FLOW`]).
#[must_use]
pub fn field_flow(body: Html) -> Html {
    let body = String::from(body);
    Html::raw(format!("<div class=\"{FIELD_FLOW}\">{body}</div>"))
}

/// Returns the checkbox `checkbox`, whose id is `id`, beside its label
/// `label` on one row, the box clear of the words.
#[must_use]
pub fn check(id: &'static str, label: impl Into<Word>, checkbox: Html) -> Html {
    keyed_check(id, label, checkbox)
}

/// Returns [`check`] for a checkbox whose id is made at run time.
#[must_use]
pub fn keyed_check(id: &str, label: impl Into<Word>, checkbox: Html) -> Html {
    let label: Word = label.into();
    let checkbox = String::from(checkbox);
    Html::raw(format!(
        "<div class=\"fui-check\">{checkbox} <label for=\"{}\">{label}</label></div>",
        Html::text(id)
    ))
}

#[derive(Template)]
#[template(
    source = r#"<div class="fui-check">{{ checkbox }}<span class="fui-check-text"><label for="{{ id }}">{{ label }}</label><span id="{{ help_id }}" class="fui-help">{{ help }}</span></span></div>"#,
    ext = "html"
)]
struct CheckField {
    id: String,
    help_id: String,
    label: Word,
    help: Word,
    checkbox: Html,
}

/// Returns the checkbox `checkbox` beside its label with the line `help`
/// under the words.
///
/// The checkbox, whose id is `id`, names the help with `aria-describedby`
/// set to [`help_id`].
#[must_use]
pub fn check_field(
    id: &'static str,
    label: impl Into<Word>,
    help: impl Into<Word>,
    checkbox: Html,
) -> Html {
    keyed_check_field(id.to_owned(), label, help, checkbox)
}

/// Returns [`check_field`] for a checkbox whose id is made at run time.
#[must_use]
pub fn keyed_check_field(
    id: String,
    label: impl Into<Word>,
    help: impl Into<Word>,
    checkbox: Html,
) -> Html {
    CheckField {
        help_id: help_id(&id),
        id,
        label: label.into(),
        help: help.into(),
        checkbox,
    }
    .html()
}

#[derive(Template)]
#[template(
    source = r#"<fieldset class="fui-fieldset fui-choice"><legend class="fui-label">{{ legend }}</legend><div class="fui-chip-checks">{% for (id, value, words, on) in chips %}<label for="{{ id }}" class="fui-chip-check"><input type="checkbox" id="{{ id }}" name="{{ name }}" value="{{ value }}"{% if *on %} checked{% endif %}>{{ words }}</label>{% endfor %}</div></fieldset>"#,
    ext = "html"
)]
struct ChipChoices {
    name: &'static str,
    legend: String,
    chips: Vec<(String, String, String, bool)>,
}

/// Returns a group of checkboxes named `name` under the legend `legend`,
/// each drawn as a chip.
///
/// A ticked chip takes the accent's edge, so the tick
/// and the edge both say so: a choice of several, such as the nodes a query
/// asks.
#[must_use]
pub fn chip_choices(
    name: &'static str,
    legend: impl Into<String>,
    options: Vec<(String, String)>,
    checked: &[String],
) -> Html {
    ChipChoices {
        name,
        legend: legend.into(),
        chips: options
            .into_iter()
            .map(|(value, words)| {
                let on = checked.contains(&value);
                (format!("{name}-{value}"), value, words, on)
            })
            .collect(),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use super::{Field, Inner, described_by, masked_input};

    #[test]
    fn a_masked_field_posts_dots_and_names_its_help() {
        let html = Field::new("identifier", "Identifier")
            .help("Nine digits.")
            .trailing(Inner::Text(String::from("x")))
            .view(masked_input(
                "identifier",
                "identifier",
                described_by("identifier", true, false),
            ));
        assert!(html.as_str().contains(r#"type="password""#), "{html}");
        assert!(
            html.as_str()
                .contains(r#"aria-describedby="identifier-help""#),
            "{html}"
        );
        assert!(html.as_str().contains("fui-input-group"), "{html}");
    }
}
