// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The code page.
//!
//! It holds the document pane with its format links and its views, the
//! code viewer with a long line, a current line and two findings under
//! their lines, a viewer in each of the four languages, and a revision's
//! changed files with one file's line diff, under a header with a
//! breadcrumb trail and a title in the code face.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::table::change::{Change, change_table};
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::code::{Code, CodeHead, Digest, Gravity, LineDiagnostic, code_card};
use ferrohealth_ui::view::document::{Document, DocumentView, document_pane};
use ferrohealth_ui::view::layout::{Arrangement, Gap, stack_at};
use ferrohealth_ui::view::line_diff::line_diff;
use ferrohealth_ui::view::surface::{Crumb, Header, card, definitions, page_heading, page_meta};

use crate::data::{
    AQL, FHIRCONNECT, FILE_NAMES, FINDING_CODES, JSON, MAPPING, MAPPING_DIGEST, MAPPING_PATH, XML,
    YAML,
};
use crate::locale::Msg;
use crate::page::Page;
use crate::visit::Visit;

/// The id prefix of the viewer's lines.
const LINES: &str = "file-L";

/// The document pane over the JSON and XML samples, in the format `xml`
/// names, with a rendered view, its format links leading back to `here`.
fn document(v: &Visit, here: &str, xml: bool) -> Html {
    let rendered = definitions(vec![
        (v.tr(Msg::Resource), Html::text(v.tr(Msg::Observation))),
        (v.tr(Msg::Status), Html::text("final")),
        (v.tr(Msg::Code), Html::text(v.tr(Msg::SyntheticMeasurement))),
        (v.tr(Msg::Value), Html::text(v.tr(Msg::N725BeatsMin))),
    ]);
    let pane = document_pane(
        &v.locale,
        Document {
            id: String::from("document"),
            label: v.tr(Msg::Observation),
            formats: vec![
                (format!("{here}?format=json"), v.tr(Msg::Json)),
                (format!("{here}?format=xml"), v.tr(Msg::Xml)),
            ],
            format: usize::from(xml),
            language: if xml { Language::Xml } else { Language::Json },
            text: String::from(if xml { XML } else { JSON }),
            opening: DocumentView::Highlighted,
            rendered: Some(rendered),
        },
    );
    card(v.tr(Msg::DocumentPane), pane)
}

/// The code viewer over the mapping file: a long line, the linked line 13
/// current, and an error and a warning under their lines.
fn viewer(v: &Visit) -> Html {
    let [retries, start] = FINDING_CODES;
    let viewer = code_card(
        &v.locale,
        CodeHead {
            title: String::from(MAPPING_PATH),
            tags: vec![String::from(FHIRCONNECT), v.tr(Msg::OwnerConsole)],
            digest: Some(Digest {
                text: String::from(MAPPING_DIGEST),
                title: v.tr(Msg::DigestOfBytes),
            }),
            copy: Some(String::from("file-copy")),
            actions: None,
        },
        &Code {
            label: v.tr(Msg::TheMappingFile),
            language: Language::Yaml,
            text: String::from(MAPPING),
            ids: Some(String::from(LINES)),
            current: Some(13),
            diagnostics: vec![
                LineDiagnostic {
                    line: 18,
                    gravity: Gravity::Error,
                    code: String::from(retries),
                    message: v.tr(Msg::FindingRetries),
                },
                LineDiagnostic {
                    line: 20,
                    gravity: Gravity::Warning,
                    code: String::from(start),
                    message: v.tr(Msg::FindingStartTwice),
                },
            ],
        },
    );
    stack_at(
        Gap::Loose,
        Html::join([
            Html::raw(format!(
                "<p>{} <a href=\"#{LINES}13\">{}</a>.</p>",
                Html::text(v.tr(Msg::DiagnosticLinksLine)),
                Html::text(v.tr(Msg::Line13))
            )),
            viewer,
        ]),
    )
}

/// The tag of a viewer in `language`.
fn language_tag(v: &Visit, language: Language) -> String {
    v.tr(match language {
        Language::Yaml => Msg::Yaml,
        Language::Json => Msg::Json,
        Language::Xml => Msg::Xml,
        Language::Aql | Language::Hl7v2 | Language::Plain => Msg::Aql,
    })
}

/// A viewer of `text` in `language`, its head naming the file `title`,
/// its box named `label`.
fn each(v: &Visit, title: &str, label: Msg, language: Language, text: &str) -> Html {
    code_card(
        &v.locale,
        CodeHead {
            title: title.to_owned(),
            tags: vec![language_tag(v, language)],
            ..CodeHead::default()
        },
        &Code {
            label: v.tr(label),
            language,
            text: text.to_owned(),
            ids: None,
            current: None,
            diagnostics: Vec::new(),
        },
    )
}

/// A viewer in each language, named by its file.
fn languages(v: &Visit) -> Html {
    let [yaml, json, xml, aql] = FILE_NAMES;
    stack_at(
        Gap::Loose,
        Html::join([
            Arrangement::Grid.wrap(Html::join([
                each(v, yaml, Msg::MappingFileYaml, Language::Yaml, YAML),
                each(v, json, Msg::ResourceJson, Language::Json, JSON),
            ])),
            Arrangement::Grid.wrap(Html::join([
                each(v, xml, Msg::CompositionXml, Language::Xml, XML),
                each(v, aql, Msg::QueryAql, Language::Aql, AQL),
            ])),
        ]),
    )
}

/// A synthetic revision of the mapping file against the one that runs, as
/// a unified diff: a changed version, a longer profile URL and one more
/// archetype.
const REVISION_DIFF: &str = "--- a/fhirconnect/gallery.context.yml
+++ b/fhirconnect/gallery.context.yml
@@ -2,6 +2,6 @@ type: context
 type: context
 metadata:
   name: gallery.context
-  version: 1.2.0
+  version: 1.3.0
 spec:
   system: FHIR
@@ -10,3 +10,4 @@ context:
   profile:
-    url: \"http://example.org/fhir/StructureDefinition/gallery-weight\"
+    url: \"http://example.org/fhir/StructureDefinition/gallery-weight-with-device-and-position\"
   archetypes:
+    - \"CLUSTER.body_position.v1\"
";

/// A revision's changed files by their digests, its first column headed
/// File, and the line diff of the one changed in place.
fn revision(v: &Visit) -> Html {
    let changes = [
        Change {
            key: String::from(MAPPING_PATH),
            from: Some(String::from("5712e3f7…f2e0e3fa")),
            to: Some(String::from("9a03c1d2…7b44e810")),
        },
        Change {
            key: String::from("fhirconnect/gallery-device.model.yml"),
            from: None,
            to: Some(String::from("c81f02aa…19e6d3b5")),
        },
    ];
    card(
        v.tr(Msg::RevisionAgainstRunning),
        stack_at(
            Gap::Default,
            Html::join([
                change_table(
                    &v.locale,
                    v.tr(Msg::RevisionFilesCaption),
                    Some(v.tr(Msg::FileColumn)),
                    &changes,
                ),
                line_diff(&v.locale, MAPPING_PATH, REVISION_DIFF),
            ]),
        ),
    )
}

/// The page's subtitle with inline markup: the resource type in bold and
/// the template id as a code link to its page.
fn subtitle(v: &Visit, root: &str) -> Html {
    Html::raw(format!(
        "{} <strong>{}</strong> {} <a href=\"{root}/templates/list\"><code>{}</code></a>{}",
        Html::text(v.tr(Msg::CodeSubtitleLead)),
        Html::text(v.tr(Msg::Observation)),
        Html::text(v.tr(Msg::CodeSubtitleOn)),
        crate::data::TEMPLATES[0],
        Html::text(v.tr(Msg::CodeSubtitleEnd))
    ))
}

/// The code page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let root = format!("/{}", v.brand.key());
    let here = format!("{root}/code");
    let xml = v.param("format") == Some("xml");
    let header = page_heading(
        &v.locale,
        Header {
            title: String::from("gallery.context.v1").into(),
            subtitle: None,
            subtitle_view: Some(subtitle(v, &root)),
            crumbs: vec![
                Crumb::new(v.tr(Msg::Gallery), root.clone()),
                Crumb::new(v.tr(Msg::CodeDocuments), here.clone()),
            ],
            mono: true,
            badge: None,
        },
        ActionGroup::default(),
    );
    let meta = page_meta(Some(v.tr(Msg::AnsweredSyntheticReplicaGallery1)));
    let body = stack_at(
        Gap::Loose,
        Html::join([
            document(v, &here, xml),
            viewer(v),
            languages(v),
            revision(v),
        ]),
    );
    Page::shell(v.tr(Msg::CodeDocuments), Html::join([header, meta, body]))
}
