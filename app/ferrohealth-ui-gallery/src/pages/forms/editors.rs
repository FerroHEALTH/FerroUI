// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The forms page's editors: the upload, the highlighted editor, the mapping
//! editors, the diagnostics, the filter and the editor headers.

use ferrohealth_ui::highlight::Language;
use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::{Arg, Phrase};
use ferrohealth_ui::icon::Glyph;
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::ActionGroup;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::code::Gravity;
use ferrohealth_ui::view::diagnostics::{
    DiagnosticsWords, FileFindings, Finding, Position, diagnostics_by_file,
};
use ferrohealth_ui::view::editor::{
    Editor, EditorWords, LineMark, MarkKind, Opened, Panel, code_editor, editor_diagnostics,
    editor_panel, editor_split, line_id,
};
use ferrohealth_ui::view::filter::{ROW_TEXT, TableFilter, table_filter};
use ferrohealth_ui::view::form::Field;
use ferrohealth_ui::view::layout::{Gap, stack_at};
use ferrohealth_ui::view::status::badge;
use ferrohealth_ui::view::surface::{
    Container, Crumb, Header, Mode, SectionHead, card, editor_heading, panel_section,
    panel_sections_card,
};
use ferrohealth_ui::view::template::list::HeadingCount;
use ferrohealth_ui::view::upload::{Sending, Upload, UploadWords, upload_field};

use crate::data::{
    CLEAN_FILE, CONTEXT_MAPPING, DRAFT_FINDINGS, DRAFT_MAPPING, DRAFT_OPENED, DRAFT_PATH,
    DRAFT_TEXT, FINDING_CODES, LONG_FINDING, MAPPING_PATH, WHOLE_FINDING,
};
use crate::locale::{Msg, message};
use crate::request::DRAFT_ROUTE;
use crate::visit::Visit;

use super::inputs::DRAFT;

/// The extensions the upload field takes.
const UPLOAD_ACCEPT: &[&str] = &[".opt", ".xml"];

/// The largest file the upload field reads.
const UPLOAD_MAX_BYTES: u64 = 2 * 1024 * 1024;

/// The upload field: a file dropped or chosen, read into the text area.
pub(super) fn upload(v: &Visit) -> Html {
    let field = upload_field(
        &Upload {
            id: "gallery-upload",
            label: v.tr(Msg::OperationalTemplate).into(),
            choose: v.tr(Msg::ChooseOptFile).into(),
            accept: UPLOAD_ACCEPT,
            max_bytes: UPLOAD_MAX_BYTES,
            placeholder: "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".into(),
            words: UploadWords::of(&v.locale, UPLOAD_ACCEPT, UPLOAD_MAX_BYTES),
        },
        Sending::default(),
    );
    card(v.tr(Msg::Upload), field)
}

/// The highlighted editor over the synthetic YAML, with two diagnostics
/// marking and linking to their lines, and the bytes it holds.
pub(super) fn editor(v: &Visit) -> Html {
    let text = crate::data::YAML;
    let marks = vec![
        LineMark {
            line: 4,
            kind: MarkKind::Warning,
        },
        LineMark {
            line: 13,
            kind: MarkKind::Error,
        },
    ];
    let field = code_editor(&Editor {
        id: DRAFT.into(),
        language: Language::Yaml,
        text: String::from(text),
        marks,
        editable: true,
    });
    let bytes = u32::try_from(text.len()).unwrap_or(u32::MAX);
    let lines = u32::try_from(text.matches('\n').count())
        .unwrap_or(u32::MAX)
        .saturating_add(1);
    let size = v.tr_with(
        Msg::SizeLines,
        &[("bytes", Arg::from(bytes)), ("lines", Arg::from(lines))],
    );
    let diagnostics: Html = [
        (4, Tone::Warn, "warning", v.tr(Msg::CommentAfterValueKeptBut)),
        (
            13,
            Tone::Bad,
            "error",
            v.tr(Msg::TemplateGalleryTemplateV1Not),
        ),
    ]
    .into_iter()
    .map(|(line, tone, word, text)| {
        Html::raw(format!(
            "<li>{}<span class=\"fui-issue-head\"><a href=\"#{}\">{}</a>:</span><span class=\"fui-issue-text\">{}</span></li>",
            badge(tone, word),
            line_id(DRAFT, line),
            Html::text(v.tr_with(Msg::IssueLine, &[("line", Arg::from(line))])),
            Html::text(text)
        ))
    })
    .collect();
    card(
        v.tr(Msg::HighlightedEditor),
        Html::join([
            Field::new(DRAFT, v.tr(Msg::File))
                .after(Html::raw(format!(
                    "<p class=\"fui-help\"><output>{}</output></p>",
                    Html::text(size)
                )))
                .view(field),
            Html::raw(format!("<ul class=\"fui-issues\">{diagnostics}</ul>")),
        ]),
    )
}

/// The id of the code editor's text area.
pub(super) const MAPPING_DRAFT: &str = "gallery-mapping";

/// The id of the new mapping's text area.
pub(super) const NEW_MAPPING: &str = "gallery-new-mapping";

/// The code editor (B2) beside its diagnostics (B3), in a form that posts
/// its draft: the draft opened with a warning and an error, and the list
/// beside it in step with the cursor.
pub(super) fn mapping_editor(v: &Visit) -> Html {
    let [(note, note_line, note_column), (retries, line, column)] = DRAFT_FINDINGS;
    let marks = vec![
        LineMark {
            line: note_line,
            kind: MarkKind::Warning,
        },
        LineMark {
            line,
            kind: MarkKind::Error,
        },
    ];
    let field = Editor {
        id: MAPPING_DRAFT.into(),
        language: Language::Yaml,
        text: String::from(DRAFT_TEXT),
        marks,
        editable: true,
    };
    let found = |gravity, code: &str, (line, column), message| Finding {
        gravity,
        code: String::from(code),
        at: Some(Position { line, column }),
        message,
        mapping: Some(String::from(DRAFT_MAPPING)),
    };
    let findings = FileFindings {
        file: String::from(DRAFT_PATH),
        href: None,
        lines: format!("#{MAPPING_DRAFT}-L"),
        current: None,
        all: None,
        findings: vec![
            found(
                Gravity::Warning,
                note,
                (note_line, note_column),
                v.tr(Msg::NoteKeptNeverRead),
            ),
            found(
                Gravity::Error,
                retries,
                (line, column),
                v.tr(Msg::RetriesMustBeNumber),
            ),
        ],
    };
    let check = Html::raw(format!(
        "<button type=\"submit\" class=\"{}\">{}</button>",
        Variant::Secondary.small(),
        label(Some(Glyph::LIST_CHECKS), v.tr(Msg::Check))
    ));
    let caret = Position::default();
    let title = v.tr(Msg::Diagnostics);
    let list = editor_diagnostics(
        &title,
        &findings,
        &DiagnosticsWords::of(&v.locale),
        &field,
        caret,
    );
    let panel = editor_panel(Panel {
        editor: field,
        label: v.tr_with(Msg::DraftOf, &[("name", DRAFT_PATH.into())]),
        lead: v.tr(Msg::YamlFhirconnectContext),
        facts: vec![v.tr(Msg::Utf8Lf)],
        opened: Opened::Text(String::from(DRAFT_OPENED)),
        caret,
        tools: Some(check),
        end: None,
        words: EditorWords::of(&v.locale),
    });
    let side = panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title,
                count: Some(HeadingCount::All(2)),
                actions: None,
            }),
            list,
        ),
    );
    editor_split(
        Html::raw(format!(
            "<form method=\"post\" action=\"{DRAFT_ROUTE}\">{panel}</form>"
        )),
        side,
    )
}

/// The code editor of a new file, empty, in a panel section (B10).
pub(super) fn new_mapping(v: &Visit) -> Html {
    let panel = editor_panel(Panel {
        editor: Editor {
            id: NEW_MAPPING.into(),
            language: Language::Yaml,
            text: String::new(),
            marks: Vec::new(),
            editable: true,
        },
        label: v.tr(Msg::NewMappingFile),
        lead: v.tr(Msg::YamlFhirconnectContext),
        facts: vec![v.tr(Msg::Utf8Lf)],
        opened: Opened::New,
        caret: Position::default(),
        tools: None,
        end: None,
        words: EditorWords::of(&v.locale),
    });
    panel_sections_card(
        v.tr(Msg::NewMapping),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: v.tr(Msg::NewMapping),
                count: None,
                actions: None,
            }),
            panel,
        ),
    )
}

/// The diagnostics list of two files, errors first, the current row on the
/// draft's error: the code page's mapping file, and the draft the code
/// editor holds.
pub(super) fn diagnostics_of_files(v: &Visit) -> Html {
    let [(note, note_line, note_column), (retries, line, column)] = DRAFT_FINDINGS;
    let [code_retries, code_start] = FINDING_CODES;
    let found = |gravity, code: &str, at: Option<(u32, u32)>, message, mapping: &str| Finding {
        gravity,
        code: String::from(code),
        at: at.map(|(line, column)| Position { line, column }),
        message,
        mapping: Some(String::from(mapping)),
    };
    let files = [
        FileFindings {
            file: String::from(MAPPING_PATH),
            href: Some(String::from("code")),
            lines: String::from("code#file-L"),
            current: None,
            all: None,
            findings: vec![
                found(
                    Gravity::Warning,
                    code_start,
                    Some((20, 3)),
                    v.tr(Msg::FindingStartTwice),
                    CONTEXT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    code_retries,
                    Some((18, 12)),
                    v.tr(Msg::FindingRetries),
                    CONTEXT_MAPPING,
                ),
            ],
        },
        FileFindings {
            file: String::from(DRAFT_PATH),
            href: None,
            lines: format!("#{MAPPING_DRAFT}-L"),
            current: Some(line),
            all: None,
            findings: vec![
                found(
                    Gravity::Warning,
                    note,
                    Some((note_line, note_column)),
                    v.tr(Msg::NoteKeptNeverRead),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    retries,
                    Some((line, column)),
                    v.tr(Msg::RetriesMustBeNumber),
                    DRAFT_MAPPING,
                ),
                found(
                    Gravity::Error,
                    WHOLE_FINDING,
                    None,
                    v.tr(Msg::ProfileUnknown),
                    DRAFT_MAPPING,
                ),
            ],
        },
    ];
    let title = v.tr(Msg::DiagnosticsByFile);
    panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: v.tr(Msg::Diagnostics),
                count: Some(HeadingCount::All(5)),
                actions: None,
            }),
            diagnostics_by_file(&v.locale, &title, &files),
        ),
    )
}

/// The diagnostics list at real size: a finding whose message and mapping
/// run past a phone's width, under a long path, and a file the check found
/// clean, which says so.
pub(super) fn diagnostics_real_size(v: &Visit) -> Html {
    let [path, code, lines, mapping] = LONG_FINDING;
    let [clean, clean_lines] = CLEAN_FILE;
    let files = [
        FileFindings {
            file: String::from(path),
            href: Some(String::from("code")),
            lines: String::from(lines),
            current: None,
            all: None,
            findings: vec![Finding {
                gravity: Gravity::Error,
                code: String::from(code),
                at: Some(Position {
                    line: 1204,
                    column: 37,
                }),
                message: v.tr(Msg::PathNamesNoNode),
                mapping: Some(String::from(mapping)),
            }],
        },
        FileFindings {
            file: String::from(clean),
            href: None,
            lines: String::from(clean_lines),
            current: None,
            all: None,
            findings: Vec::new(),
        },
    ];
    let title = v.tr(Msg::DiagnosticsRealSize);
    panel_sections_card(
        title.clone(),
        panel_section(
            &v.locale,
            Some(SectionHead {
                title: title.clone(),
                count: Some(HeadingCount::All(1)),
                actions: None,
            }),
            diagnostics_by_file(&v.locale, &title, &files),
        ),
    )
}

/// The filter over a list of synthetic file names.
pub(super) fn filter(v: &Visit) -> Html {
    let names: Vec<String> = crate::data::rows()
        .into_iter()
        .take(12)
        .map(|row| row.name)
        .collect();
    let total = u32::try_from(names.len()).unwrap_or(u32::MAX);
    let items: Html = names
        .iter()
        .map(|name| {
            Html::raw(format!(
                "<li {ROW_TEXT}=\"{0}\"><code>{0}</code></li>",
                Html::text(name)
            ))
        })
        .collect();
    let filter = table_filter(TableFilter {
        id: "gallery-filter",
        label: v.tr(Msg::FilterFiles),
        rows: String::from("gallery-filter-rows"),
        count: v.tr_with(
            Msg::FilesOnPage,
            &[("kept", Arg::from(total)), ("total", Arg::from(total))],
        ),
        phrase: Phrase::new(v.tr_with(
            Msg::FilesOnPage,
            &[
                ("kept", Phrase::open("kept")),
                ("total", Phrase::open("total")),
            ],
        )),
        words: String::new(),
    });
    card(
        v.tr(Msg::Filter),
        Html::raw(format!(
            "{filter}<ul id=\"gallery-filter-rows\" class=\"fui-divided\" aria-label=\"{}\">{items}</ul>",
            Html::text(v.tr(Msg::Files))
        )),
    )
}

/// The header of an editor that creates a file and of one that edits a
/// file: the title says which, and the trail ends with the list a new file
/// joins or with the file edited.
pub(super) fn editor_headers(v: &Visit) -> Html {
    let header = |mode: Mode| {
        let mut crumbs = vec![Crumb::new(v.tr(Msg::MappingFilesCrumb), String::from("#"))];
        if let Mode::Editing(name) = &mode {
            crumbs.push(Crumb::new(name.clone(), String::from("#")));
        }
        Container::Well.wrap(editor_heading(
            &v.locale,
            message(Msg::FileEditor),
            &mode,
            Header {
                crumbs,
                ..Header::default()
            },
            ActionGroup::default(),
        ))
    };
    card(
        v.tr(Msg::EditorHeaders),
        stack_at(
            Gap::Loose,
            Html::raw(format!(
                "<div data-editor-mode=\"new\">{}</div><div data-editor-mode=\"edit\">{}</div>",
                header(Mode::Creating),
                header(Mode::Editing(String::from("synthetic-007.model.yml")))
            )),
        ),
    )
}
