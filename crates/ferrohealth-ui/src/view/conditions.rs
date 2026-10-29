// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The query builder's condition groups (E6).
//!
//! A group's head reads as one phrase, "Match [All | Any] of these
//! conditions", the choice a segmented radio group, then the check box
//! "Match the opposite (NOT)". Each group has an accent edge, dashed when
//! negated, so negation is the word and the edge, never colour alone; a
//! nested group's edge is the strong hairline. A condition is an F7 row:
//! the path as a select with its archetype path as a quiet mono line under
//! it, the operator, a typed value (a unit suffix, or a coded select), and
//! Remove, which stays on the row's first line on a phone and in a group
//! too narrow for the fields side by side. Add a condition
//! and Add a group close each group.
//!
//! Every control is a field of the console's own form, which wraps the
//! groups: the choices post as they stand (a chosen value its select does
//! not list stays chosen, said to be not in the list), and Remove, Add a condition and
//! Add a group are submit buttons naming what they act on, so the builder
//! works with no WebAssembly (`leptos-ui.md` §15.8). No specification
//! governs the builder: our own design; the query it builds is the
//! console's AQL (openEHR Query, AQL, § WHERE).

use crate::html::Html;
use crate::i18n::{Arg, Locale, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::Variant;
use crate::view::form::{
    Field, INPUT, Inner, SELECT, described_with_unit, keyed_check, keyed_field,
};

/// The name the submit button that removes a condition posts, its value
/// the condition's key.
pub const REMOVE: &str = "remove";

/// The name the submit button that removes a nested group posts, its value
/// the group's key.
pub const REMOVE_GROUP: &str = "remove-group";

/// The name Add a condition posts, its value the group's key.
pub const ADD_CONDITION: &str = "add-condition";

/// The name Add a group posts, its value the group's key.
pub const ADD_GROUP: &str = "add-group";

/// How a group combines its conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Match {
    /// Every condition holds.
    All,
    /// At least one condition holds.
    Any,
}

impl Match {
    /// The value the group's choice posts.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Any => "any",
        }
    }
}

/// A select's options and the one chosen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Choice {
    /// The options, each its value and its words.
    pub options: Vec<(String, String)>,
    /// The value of the chosen option.
    pub chosen: String,
}

/// A condition's typed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operand {
    /// Typed text, such as a number, with the unit as a person reads it
    /// after it inside the field's edge when the path is a quantity.
    Typed {
        /// What the field holds.
        value: String,
        /// The unit, such as "mm Hg".
        unit: Option<String>,
    },
    /// A coded value, chosen from the path's value set.
    Coded(Choice),
}

/// One condition of a group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Condition {
    /// The condition's key, unique in the builder: its controls take the
    /// ids and names `<key>-path`, `<key>-op` and `<key>-value`, and Remove
    /// posts it.
    pub key: String,
    /// The paths to choose from, each in words ("Blood pressure ›
    /// Systolic").
    pub path: Choice,
    /// The archetype path of the chosen path, the quiet mono line under
    /// it.
    pub archetype_path: String,
    /// The operators to choose from.
    pub operator: Choice,
    /// The value.
    pub operand: Operand,
}

/// One group of conditions, and the groups nested in it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionGroup {
    /// The group's key, unique in the builder: its choice posts as
    /// `<key>-match`, its negation as `<key>-not`, and its Add and Remove
    /// buttons post it.
    pub key: String,
    /// The group's number as the reader counts it, the first group 1.
    pub number: u32,
    /// How it combines its conditions.
    pub matching: Match,
    /// Whether it matches the opposite.
    pub negated: bool,
    /// Its conditions.
    pub conditions: Vec<Condition>,
    /// The groups nested in it, after its conditions.
    pub groups: Vec<ConditionGroup>,
}

/// Returns the options of `choice` as `<option>`s, the chosen one selected.
///
/// A chosen value no option holds leads as an option of its own, selected
/// and said to be not in the list, so the select shows and posts the value
/// the condition has rather than its first option; an empty one chooses
/// nothing.
fn options(locale: &Locale, choice: Choice) -> String {
    let Choice { options, chosen } = choice;
    let unlisted = if !chosen.is_empty() && !options.iter().any(|(value, _)| *value == chosen) {
        let words = tr_with(
            locale,
            Text::CondNotListed,
            &[("value", Arg::from(chosen.clone()))],
        );
        format!(
            "<option value=\"{}\" selected>{}</option>",
            Html::text(&chosen),
            Html::text(words.text())
        )
    } else {
        String::new()
    };
    let listed: String = options
        .into_iter()
        .map(|(value, words)| {
            format!(
                "<option value=\"{}\"{}>{}</option>",
                Html::text(&value),
                if value == chosen { " selected" } else { "" },
                Html::text(words)
            )
        })
        .collect::<Vec<_>>()
        .concat();
    format!("{unlisted}{listed}")
}

/// Returns the select `id` holding `choice`.
fn select(locale: &Locale, id: &str, choice: Choice) -> Html {
    let id = Html::text(id);
    Html::raw(format!(
        "<select id=\"{id}\" name=\"{id}\" class=\"{SELECT}\">{}</select>",
        options(locale, choice)
    ))
}

/// Returns the words of the chosen option of `choice`, or its value when
/// no option holds it.
fn chosen_words(choice: &Choice) -> String {
    choice
        .options
        .iter()
        .find(|(value, _)| *value == choice.chosen)
        .map_or_else(|| choice.chosen.clone(), |(_, words)| words.clone())
}

/// Returns one condition row.
fn condition(locale: &Locale, condition: Condition) -> String {
    let Condition {
        key,
        path,
        archetype_path,
        operator,
        operand,
    } = condition;
    let remove_name = Html::text(
        tr_with(
            locale,
            Text::CondRemove,
            &[("label", Arg::from(chosen_words(&path)))],
        )
        .text(),
    );
    let path_id = format!("{key}-path");
    let path_field = Field::new(path_id.clone(), tr(locale, Text::CondPath))
        .after(Html::raw(format!(
            "<span title=\"{0}\" class=\"fui-cond-path\">{0}</span>",
            Html::text(&archetype_path)
        )))
        .view(select(locale, &path_id, path));
    let operator_id = format!("{key}-op");
    let operator_field = keyed_field(
        operator_id.clone(),
        tr(locale, Text::CondOperator),
        None,
        select(locale, &operator_id, operator),
    );
    let value_id = format!("{key}-value");
    let value_field = match operand {
        Operand::Typed { value, unit } => {
            let described = unit
                .as_ref()
                .map(|_| {
                    format!(
                        " aria-describedby=\"{}\"",
                        Html::text(described_with_unit(&value_id, false, false))
                    )
                })
                .unwrap_or_default();
            let input = Html::raw(format!(
                "<input id=\"{0}\" name=\"{0}\" value=\"{1}\" inputmode=\"decimal\"{described} class=\"{INPUT}\">",
                Html::text(&value_id),
                Html::text(value)
            ));
            let field = Field::new(value_id, tr(locale, Text::CondValue));
            match unit {
                Some(unit) => field.trailing(Inner::Unit(unit)).view(input),
                None => field.view(input),
            }
        }
        Operand::Coded(choice) => keyed_field(
            value_id.clone(),
            tr(locale, Text::CondValue),
            None,
            select(locale, &value_id, choice),
        ),
    };
    format!(
        "<li class=\"fui-cond\">{path_field}{operator_field}{value_field}<button type=\"submit\" name=\"{REMOVE}\" value=\"{}\" aria-label=\"{remove_name}\" title=\"{remove_name}\" class=\"fui-icon-button\">{}</button></li>",
        Html::text(&key),
        icon(Glyph::X, "16")
    )
}

/// Returns the group's All or Any choice: the head's phrase, its visible
/// words hidden from the accessibility tree and said once as the radio
/// group's name, so a screen reader hears one phrase.
fn match_choice(locale: &Locale, key: &str, number: u32, matching: Match) -> String {
    let name = Html::text(format!("{key}-match"));
    let segments: String = [(Match::All, Text::CondAll), (Match::Any, Text::CondAny)]
        .into_iter()
        .map(|(each, words)| {
            format!(
                "<label for=\"{name}-{key}\" class=\"fui-segment\"><input type=\"radio\" id=\"{name}-{key}\" name=\"{name}\" value=\"{key}\"{} class=\"fui-sr-only\">{}</label>",
                if each == matching { " checked" } else { "" },
                tr(locale, words),
                key = each.key()
            )
        })
        .collect::<Vec<_>>()
        .concat();
    let said = tr_with(
        locale,
        Text::CondGroupLabel,
        &[("label", Arg::from(number.to_string()))],
    );
    format!(
        "<span aria-hidden=\"true\" class=\"fui-cond-phrase\">{}</span><span id=\"{name}-label\" class=\"fui-sr-only\">{said}</span><div role=\"radiogroup\" aria-labelledby=\"{name}-label\" class=\"fui-segmented\">{segments}</div><span aria-hidden=\"true\" class=\"fui-cond-phrase\">{}</span>",
        tr(locale, Text::CondMatch),
        tr(locale, Text::CondOfThese)
    )
}

/// Returns the group `group`, nested `depth` groups deep.
fn group_view(locale: &Locale, group: ConditionGroup, depth: u32) -> String {
    let ConditionGroup {
        key,
        number,
        matching,
        negated,
        conditions,
        groups,
    } = group;
    let nested = depth > 0;
    let not_id = format!("{key}-not");
    let negation = keyed_check(
        &not_id,
        tr(locale, Text::CondNot),
        Html::raw(format!(
            "<input type=\"checkbox\" id=\"{0}\" name=\"{0}\" value=\"true\"{1}>",
            Html::text(&not_id),
            if negated { " checked" } else { "" }
        )),
    );
    let label = || Arg::from(number.to_string());
    let remove = if nested {
        let name = Html::text(tr_with(locale, Text::CondRemoveGroup, &[("label", label())]).text());
        format!(
            "<div class=\"fui-actions\"><button type=\"submit\" name=\"{REMOVE_GROUP}\" value=\"{}\" aria-label=\"{name}\" title=\"{name}\" class=\"fui-icon-button\">{}</button></div>",
            Html::text(&key),
            icon(Glyph::TRASH, "16")
        )
    } else {
        String::new()
    };
    let rows: String = conditions
        .into_iter()
        .map(|each| condition(locale, each))
        .collect::<Vec<_>>()
        .concat();
    let inner: String = groups
        .into_iter()
        .map(|inner| format!("<li>{}</li>", group_view(locale, inner, depth + 1)))
        .collect::<Vec<_>>()
        .concat();
    let add_said = if nested {
        format!(
            " aria-label=\"{}\"",
            Html::text(tr_with(locale, Text::CondAddConditionTo, &[("label", label())]).text())
        )
    } else {
        String::new()
    };
    let add_group = if nested {
        String::new()
    } else {
        format!(
            "<button type=\"submit\" name=\"{ADD_GROUP}\" value=\"{}\" class=\"{}\">{}</button>",
            Html::text(&key),
            Variant::Ghost.small(),
            crate::view::button::label(Some(Glyph::PLUS), tr(locale, Text::CondAddGroup))
        )
    };
    format!(
        "<div class=\"fui-cond-group{}\"><div class=\"fui-cond-head\">{}{negation}{remove}</div><ul class=\"fui-cond-list\">{rows}{inner}</ul><div class=\"fui-cluster fui-add-row\"><button type=\"submit\" name=\"{ADD_CONDITION}\" value=\"{}\"{add_said} class=\"{}\">{}</button>{add_group}</div></div>",
        if negated { " is-not" } else { "" },
        match_choice(locale, &key, number, matching),
        Html::text(&key),
        Variant::Ghost.small(),
        crate::view::button::label(Some(Glyph::PLUS), tr(locale, Text::CondAddCondition))
    )
}

/// Returns the condition groups of a query, `group` the outermost.
///
/// Put them inside the console's form, which posts every choice and the
/// submit button pressed ([`REMOVE`], [`REMOVE_GROUP`], [`ADD_CONDITION`],
/// [`ADD_GROUP`]). Only the outermost group offers Add a group; a nested
/// group offers its own Remove.
#[must_use]
pub fn condition_groups(locale: &Locale, group: ConditionGroup) -> Html {
    Html::raw(group_view(locale, group, 0))
}
