// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's two actions answered on the page: the note, a plain
//! post the page draws itself again for (#485), and the archetype id, the
//! same post answered in place by the `answer` module, as you type and on
//! Check (#546).

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Arg;
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::view::answer::Answer;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::form::{ERROR, FORM, INPUT, Posted, field, help_id};
use ferrohealth_ui::view::surface::card;

use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The field the archetype id check posts.
const ARCHETYPE: &str = "archetype";

/// The id of the archetype id check's answer region.
const CHECK_ANSWER: &str = "check-answer";

/// The rest before the archetype id check posts while you type.
const SETTLE_MS: u32 = 300;

/// The prefix of an archetype id of the openEHR EHR reference model.
const EHR_PREFIX: &str = "openEHR-EHR-";

/// What the note of the action answered on its page says once posted: its
/// word count, or the refusal of an empty note.
fn counted(v: &Visit, note: &str) -> Result<String, String> {
    let words = note.split_whitespace().count();
    if words == 0 {
        return Err(v.tr(Msg::AcknowledgeEmpty));
    }
    let count = u32::try_from(words).unwrap_or(u32::MAX);
    Ok(v.tr_with(Msg::NoteWords, &[("count", Arg::from(count))]))
}

/// An action whose answer is drawn on its page: the form posts to the page,
/// which draws itself again with the answer and the posted note kept in its
/// field (#485).
pub(super) fn answered_action(v: &Visit, posted: Option<&Posted>) -> Html {
    let posted = posted.filter(|posted| posted.field("note").is_some());
    let note = posted.and_then(|posted| posted.field("note"));
    let answer = note
        .map(|note| match counted(v, note) {
            Ok(count) => h(format!(
                "<p id=\"note-answer\" class=\"fui-help\">{}</p>",
                t(count)
            )),
            Err(refusal) => h(format!(
                "<p id=\"note-answer\" class=\"{ERROR}\">{}</p>",
                t(refusal)
            )),
        })
        .unwrap_or_default();
    // The form's other fields come back as posted, the secrets left out.
    let kept: Html = posted
        .map(Posted::fields)
        .unwrap_or_default()
        .iter()
        .filter(|(name, _)| name != "note")
        .map(|(name, value)| {
            h(format!(
                "<input type=\"hidden\" name=\"{}\" value=\"{}\">",
                t(name),
                t(value)
            ))
        })
        .collect();
    let input = h(format!(
        "<input id=\"answer-note\" name=\"note\" type=\"text\" class=\"{INPUT}\" value=\"{}\">",
        t(note.unwrap_or_default())
    ));
    card(
        v.tr(Msg::ActionAnsweredOnPage),
        h(format!(
            "<form method=\"post\" class=\"{FORM}\" id=\"answer-form\">{kept}{}<button type=\"submit\" class=\"{}\">{}</button></form><div aria-live=\"polite\">{answer}</div>",
            field("answer-note", v.tr(Msg::NoteToCheck), None, input),
            Variant::Primary.class(),
            label(Some(Glyph::CHECK), v.tr(Msg::Check)),
        )),
    )
}

/// The parts of the archetype id `id` of the openEHR EHR reference model,
/// `openEHR-EHR-CLASS.concept.vN`: its class, concept and version.
fn archetype_parts(id: &str) -> Option<(&str, &str, &str)> {
    let rest = id.strip_prefix(EHR_PREFIX)?;
    let (class, rest) = rest.split_once('.')?;
    let (concept, version) = rest.rsplit_once(".v")?;
    let class_ok = !class.is_empty() && class.chars().all(|c| c.is_ascii_uppercase() || c == '_');
    let concept_ok = !concept.is_empty()
        && concept
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    let version_ok = !version.is_empty() && version.chars().all(|c| c.is_ascii_digit());
    (class_ok && concept_ok && version_ok).then_some((class, concept, version))
}

/// The answer to the archetype id `id`: what it names, or its refusal.
fn checked(v: &Visit, id: &str) -> Result<String, String> {
    let id = id.trim();
    if id.is_empty() {
        return Err(v.tr(Msg::ArchetypeIdEmpty));
    }
    let (class, concept, version) =
        archetype_parts(id).ok_or_else(|| v.tr(Msg::ArchetypeIdRefused))?;
    Ok(v.tr_with(
        Msg::ArchetypeIdWellFormed,
        &[
            ("kind", Arg::from(class)),
            ("name", Arg::from(concept)),
            ("version", Arg::from(version)),
        ],
    ))
}

/// An action answered in place (#546): the archetype id posts to the page
/// as you type and on Check, and the `answer` module moves the answer the
/// page draws into its region here; with no script the page answers whole.
pub(super) fn answered_in_place(v: &Visit, posted: Option<&Posted>) -> Html {
    let id = posted.and_then(|posted| posted.field(ARCHETYPE));
    let answer = id
        .map(|id| match checked(v, id) {
            Ok(names) => h(format!("<p class=\"fui-help\">{}</p>", t(names))),
            Err(refusal) => h(format!("<p class=\"{ERROR}\">{}</p>", t(refusal))),
        })
        .unwrap_or_default();
    let input = h(format!(
        "<input id=\"check-id\" name=\"{ARCHETYPE}\" type=\"text\" class=\"{INPUT}\" autocomplete=\"off\" spellcheck=\"false\" aria-describedby=\"{} {CHECK_ANSWER}\" value=\"{}\">",
        help_id("check-id"),
        t(id.unwrap_or_default())
    ));
    card(
        v.tr(Msg::CheckArchetypeId),
        h(format!(
            "<form method=\"post\" class=\"{FORM}\" id=\"check-form\"{}>{}<button type=\"submit\" class=\"{}\">{}</button></form><div id=\"{CHECK_ANSWER}\" aria-live=\"polite\">{answer}</div>",
            Answer::new(CHECK_ANSWER)
                .with_settle(SETTLE_MS)
                .attributes(),
            field(
                "check-id",
                v.tr(Msg::ArchetypeId),
                Some(v.tr(Msg::ArchetypeIdCheckedAsYouType)),
                input
            ),
            Variant::Primary.class(),
            label(Some(Glyph::CHECK), v.tr(Msg::Check)),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::archetype_parts;

    #[test]
    fn an_archetype_id_reads_as_its_class_concept_and_version() {
        assert_eq!(
            Some(("OBSERVATION", "blood_pressure", "2")),
            archetype_parts("openEHR-EHR-OBSERVATION.blood_pressure.v2")
        );
        assert_eq!(
            Some(("CLUSTER", "device-detail", "10")),
            archetype_parts("openEHR-EHR-CLUSTER.device-detail.v10")
        );
        for refused in [
            "",
            "blood_pressure",
            "openEHR-EHR-OBSERVATION.blood_pressure",
            "openEHR-EHR-observation.blood_pressure.v1",
            "openEHR-EHR-OBSERVATION..v1",
            "openEHR-EHR-OBSERVATION.blood pressure.v1",
            "openEHR-EHR-OBSERVATION.blood_pressure.vx",
        ] {
            assert_eq!(None, archetype_parts(refused), "{refused}");
        }
    }
}
