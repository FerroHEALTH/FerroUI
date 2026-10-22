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
//! Remove, which stays on the row's first line on a phone. Add a condition
//! and Add a group close each group.
//!
//! Every control is a field of the console's own form, which wraps the
//! groups: the choices post as they stand, and Remove, Add a condition and
//! Add a group are submit buttons naming what they act on, so the builder
//! works with no WebAssembly (`leptos-ui.md` §15.8). No specification
//! governs the builder: our own design; the query it builds is the
//! console's AQL (openEHR Query, AQL, § WHERE).

use leptos::prelude::*;

use crate::i18n::{Arg, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::button::{Variant, label};
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
fn options(choice: Choice) -> AnyView {
    let Choice { options, chosen } = choice;
    options
        .into_iter()
        .map(|(value, words)| {
            let selected = value == chosen;
            view! {
                <option value=value selected=selected>
                    {words}
                </option>
            }
        })
        .collect_view()
        .into_any()
}

/// Returns the select `id` holding `choice`.
fn select(id: String, choice: Choice) -> AnyView {
    let name = id.clone();
    view! {
        <select id=id name=name class=SELECT>
            {options(choice)}
        </select>
    }
    .into_any()
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
fn condition(condition: Condition) -> AnyView {
    let Condition {
        key,
        path,
        archetype_path,
        operator,
        operand,
    } = condition;
    let remove_name = tr_with(
        Text::CondRemove,
        &[("label", Arg::from(chosen_words(&path)))],
    );
    let path_id = format!("{key}-path");
    let path_hint = archetype_path.clone();
    let path_field = Field::new(path_id.clone(), tr(Text::CondPath))
        .after(view! {
            <span class="fui-cond-path" title=path_hint>
                {archetype_path}
            </span>
        })
        .view(select(path_id, path));
    let operator_id = format!("{key}-op");
    let operator_field = keyed_field(
        operator_id.clone(),
        tr(Text::CondOperator),
        None,
        select(operator_id, operator),
    );
    let value_id = format!("{key}-value");
    let value_field = match operand {
        Operand::Typed { value, unit } => {
            let described = unit
                .as_ref()
                .map(|_| described_with_unit(&value_id, false, false));
            let input_id = value_id.clone();
            let input_name = value_id.clone();
            let input = view! {
                <input
                    id=input_id
                    name=input_name
                    class=INPUT
                    value=value
                    inputmode="decimal"
                    aria-describedby=described
                />
            };
            let field = Field::new(value_id, tr(Text::CondValue));
            match unit {
                Some(unit) => field.trailing(Inner::Unit(unit)).view(input),
                None => field.view(input),
            }
        }
        Operand::Coded(choice) => keyed_field(
            value_id.clone(),
            tr(Text::CondValue),
            None,
            select(value_id, choice),
        )
        .into_any(),
    };
    view! {
        <li class="fui-cond">
            {path_field}
            {operator_field}
            {value_field}
            <button
                type="submit"
                class="fui-icon-button"
                name=REMOVE
                value=key
                aria-label=remove_name.clone()
                title=remove_name
            >
                {icon(Glyph::X, "16")}
            </button>
        </li>
    }
    .into_any()
}

/// Returns the group's All or Any choice: the head's phrase, its visible
/// words hidden from the accessibility tree and said once as the radio
/// group's name, so a screen reader hears one phrase.
fn match_choice(key: &str, number: u32, matching: Match) -> AnyView {
    let name = format!("{key}-match");
    let label_id = format!("{key}-match-label");
    let labelled = label_id.clone();
    let segments = [(Match::All, Text::CondAll), (Match::Any, Text::CondAny)]
        .into_iter()
        .map(|(each, words)| {
            let id = format!("{name}-{}", each.key());
            let label_for = id.clone();
            let radio_name = name.clone();
            view! {
                <label for=label_for class="fui-segment">
                    <input
                        type="radio"
                        id=id
                        name=radio_name
                        value=each.key()
                        checked=each == matching
                        class="fui-sr-only"
                    />
                    {tr(words)}
                </label>
            }
        })
        .collect_view();
    let said = tr_with(
        Text::CondGroupLabel,
        &[("label", Arg::from(number.to_string()))],
    );
    view! {
        <span class="fui-cond-phrase" aria-hidden="true">
            {tr(Text::CondMatch)}
        </span>
        <span id=label_id class="fui-sr-only">
            {said}
        </span>
        <div class="fui-segmented" role="radiogroup" aria-labelledby=labelled>
            {segments}
        </div>
        <span class="fui-cond-phrase" aria-hidden="true">
            {tr(Text::CondOfThese)}
        </span>
    }
    .into_any()
}

/// Returns the group `group`, nested `depth` groups deep.
fn group_view(group: ConditionGroup, depth: u32) -> AnyView {
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
    let not_name = not_id.clone();
    let negation = keyed_check(
        not_id.clone(),
        tr(Text::CondNot),
        view! { <input type="checkbox" id=not_id name=not_name value="true" checked=negated /> },
    );
    let remove = nested.then(|| {
        let name = tr_with(
            Text::CondRemoveGroup,
            &[("label", Arg::from(number.to_string()))],
        );
        let value = key.clone();
        view! {
            <div class="fui-actions">
                <button
                    type="submit"
                    class="fui-icon-button"
                    name=REMOVE_GROUP
                    value=value
                    aria-label=name.clone()
                    title=name
                >
                    {icon(Glyph::TRASH, "16")}
                </button>
            </div>
        }
    });
    let rows = conditions.into_iter().map(condition).collect_view();
    let inner = groups
        .into_iter()
        .map(|inner| view! { <li>{group_view(inner, depth + 1)}</li> })
        .collect_view();
    let add_said = nested.then(|| {
        tr_with(
            Text::CondAddConditionTo,
            &[("label", Arg::from(number.to_string()))],
        )
    });
    let condition_value = key.clone();
    let add_group = (!nested).then(|| {
        let value = key.clone();
        view! {
            <button type="submit" class=Variant::Ghost.small() name=ADD_GROUP value=value>
                {label(Some(Glyph::PLUS), tr(Text::CondAddGroup))}
            </button>
        }
    });
    view! {
        <div class="fui-cond-group" class:is-not=negated>
            <div class="fui-cond-head">{match_choice(&key, number, matching)}{negation}{remove}</div>
            <ul class="fui-cond-list">{rows}{inner}</ul>
            <div class="fui-cluster fui-add-row">
                <button
                    type="submit"
                    class=Variant::Ghost.small()
                    name=ADD_CONDITION
                    value=condition_value
                    aria-label=add_said
                >
                    {label(Some(Glyph::PLUS), tr(Text::CondAddCondition))}
                </button>
                {add_group}
            </div>
        </div>
    }
    .into_any()
}

/// Returns the condition groups of a query, `group` the outermost.
///
/// Put them inside the console's form, which posts every choice and the
/// submit button pressed ([`REMOVE`], [`REMOVE_GROUP`], [`ADD_CONDITION`],
/// [`ADD_GROUP`]). Only the outermost group offers Add a group; a nested
/// group offers its own Remove.
#[must_use]
pub fn condition_groups(group: ConditionGroup) -> AnyView {
    group_view(group, 0)
}
