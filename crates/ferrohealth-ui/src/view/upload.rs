// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The upload field: a file dropped or chosen, or its text pasted, read in
//! the browser into the highlighted editor, where it can be read and
//! corrected before the console sends it.
//!
//! The server draws the drop zone, the chooser, the rule and the editor,
//! and the console's in-flight state and upstream refusal when it passes
//! them. The `upload` module reads a file dragged onto the zone or chosen:
//! the zone reads "Drop to read the file" while one is over it, a file is
//! refused, with the reason shown and announced, when its name ends in none
//! of the accepted extensions (a field that names none takes any) or it is
//! larger than the limit ([`admit`], which the module checks again). A file
//! read replaces the text and the field names it; of several dropped at
//! once it reads the first and says so, and a file that is not UTF-8 text
//! is read with each byte it could not decode as U+FFFD, and the field says
//! that too ([`decode`]). The bytes are read with `Blob.arrayBuffer()` (W3C
//! File API). The library sends nothing: the console's form posts the text.
//! With no script the text area takes pasted text, and the file input is a
//! plain file input a multipart form posts.

use std::borrow::Cow;

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::highlight::Language;
use crate::html::{Html, Render as _};
use crate::i18n::number::Grouping;
use crate::i18n::{Locale, Phrase, Text, Word, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::view::editor::{Drawn, Editor, drawn};

/// What an upload field accepts and says.
#[derive(Debug, Clone)]
pub struct Upload {
    /// The field's id: the file input is `id` and `-file`, the text area
    /// `id` and `-text` (the name the form posts the text under), the drop
    /// zone `id` and `-drop`.
    pub id: &'static str,
    /// The field's label, such as "Operational template".
    pub label: Cow<'static, str>,
    /// The chooser's words, such as "Choose OPT file".
    pub choose: Cow<'static, str>,
    /// The extensions accepted, with their dots, such as `[".opt", ".xml"]`;
    /// none takes a file of any kind.
    pub accept: &'static [&'static str],
    /// The largest file read, in bytes.
    pub max_bytes: u64,
    /// The text area's placeholder: the format's first line.
    pub placeholder: Cow<'static, str>,
    /// The words the field draws itself ([`UploadWords::of`] over the same
    /// `accept` and `max_bytes`).
    pub words: UploadWords,
}

/// A size in words, resolved on the server in the page's locale with the
/// count left open, so the `upload` module can name the size of a file it
/// reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeWords {
    /// One byte, whole.
    pub one_byte: Word,
    /// A count of bytes other than one, around the open `count`.
    pub bytes: Phrase,
    /// A count of kibibytes, around the open `count`.
    pub kib: Phrase,
    /// A count of mebibytes, around the open `count`.
    pub mib: Phrase,
    /// The mark the count is grouped by.
    pub grouping: Grouping,
}

impl SizeWords {
    /// The size words in `locale`.
    #[must_use]
    pub fn of(locale: &Locale) -> Self {
        let open = |text| Phrase::new(tr_with(locale, text, &[("count", Phrase::open("count"))]));
        Self {
            one_byte: tr_with(locale, Text::SizeBytes, &[("count", 1_u32.into())]),
            bytes: open(Text::SizeBytes),
            kib: open(Text::SizeKib),
            mib: open(Text::SizeMib),
            grouping: Grouping::of(locale),
        }
    }

    /// `bytes` in the largest unit that keeps it above one, its count's
    /// digits grouped as the page's language groups them.
    #[must_use]
    pub fn size(&self, bytes: u64) -> Word {
        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * 1024;
        let (unit, count) = if bytes >= MIB {
            (&self.mib, bytes.div_ceil(MIB))
        } else if bytes >= KIB {
            (&self.kib, bytes.div_ceil(KIB))
        } else if bytes == 1 {
            return self.one_byte.clone();
        } else {
            (&self.bytes, bytes)
        };
        unit.fill(&[("count", &self.grouping.write(count))])
    }
}

/// The words an upload field draws itself, resolved on the server in the
/// page's locale, so the `upload` module that draws them carries no
/// catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadWords {
    /// The drop zone's words.
    pub drop_invite: Word,
    /// The drop zone's words while a file is over it.
    pub drop_over: Word,
    /// The kinds of file and the limit the field takes.
    pub rule: Word,
    /// The line while the console sends.
    pub busy: Word,
    /// The line once a file is read, around the open `file`.
    pub read: Phrase,
    /// The line once the first of several files dropped at once is read,
    /// around the open `file`.
    pub read_one: Phrase,
    /// The line under it when the file was not UTF-8 text, around the open
    /// `file`.
    pub read_lossy: Phrase,
    /// A file of a kind the field does not take, around the open `file`.
    pub refused_kind: Phrase,
    /// A file over the limit, around the open `file` and `size`.
    pub refused_size: Phrase,
    /// A file the browser could not read, around the open `file`.
    pub unreadable: Phrase,
    /// The words of a size.
    pub sizes: SizeWords,
}

impl UploadWords {
    /// The words of a field that takes the extensions `accept` up to
    /// `max_bytes`, in `locale`.
    #[must_use]
    pub fn of(locale: &Locale, accept: &[&str], max_bytes: u64) -> Self {
        let sizes = SizeWords::of(locale);
        let kinds = accept.join(", ");
        let limit = sizes.size(max_bytes);
        let file = || ("file", Phrase::open("file"));
        let rule = if accept.is_empty() {
            tr_with(
                locale,
                Text::UploadRuleAny,
                &[("limit", limit.clone().into())],
            )
        } else {
            tr_with(
                locale,
                Text::UploadRule,
                &[
                    ("kinds", kinds.clone().into()),
                    ("limit", limit.clone().into()),
                ],
            )
        };
        Self {
            drop_invite: tr(locale, Text::UploadDropInvite),
            drop_over: tr(locale, Text::UploadDropOver),
            rule,
            busy: tr(locale, Text::UploadBusy),
            read: Phrase::new(tr_with(locale, Text::UploadRead, &[file()])),
            read_one: Phrase::new(tr_with(locale, Text::UploadReadOne, &[file()])),
            read_lossy: Phrase::new(tr_with(locale, Text::UploadReadLossy, &[file()])),
            refused_kind: Phrase::new(tr_with(
                locale,
                Text::UploadRefusedKind,
                &[file(), ("kinds", kinds.into())],
            )),
            refused_size: Phrase::new(tr_with(
                locale,
                Text::UploadRefusedSize,
                &[
                    file(),
                    ("size", Phrase::open("size")),
                    ("limit", limit.into()),
                ],
            )),
            unreadable: Phrase::new(tr_with(locale, Text::UploadUnreadable, &[file()])),
            sizes,
        }
    }
}

/// Why a file was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    /// Its name ends in none of the accepted extensions.
    Kind {
        /// The file's name.
        name: String,
    },
    /// It is larger than the limit.
    Size {
        /// The file's name.
        name: String,
        /// Its size, in bytes.
        bytes: u64,
    },
}

impl Refused {
    /// The sentence the field shows for the refusal, in `words`.
    #[must_use]
    pub fn sentence(&self, words: &UploadWords) -> Word {
        match self {
            Self::Kind { name } => words.refused_kind.fill(&[("file", name)]),
            Self::Size { name, bytes } => words
                .refused_size
                .fill(&[("file", name), ("size", words.sizes.size(*bytes).text())]),
        }
    }
}

/// Whether a file `name` of `bytes` may be read under `upload`, or why not.
///
/// # Errors
/// Returns the [`Refused`] reason of a file of another kind or too large.
pub fn admit(upload: &Upload, name: &str, bytes: u64) -> Result<(), Refused> {
    Rule::of(upload).admit(name, bytes)
}

/// The kinds of file and the limit of an upload field: the part of it the
/// `upload` module checks again in the browser.
#[derive(Debug, Clone, Copy)]
struct Rule {
    /// The extensions accepted, with their dots.
    accept: &'static [&'static str],
    /// The largest file read, in bytes.
    max_bytes: u64,
}

impl Rule {
    /// The rule of `upload`.
    const fn of(upload: &Upload) -> Self {
        Self {
            accept: upload.accept,
            max_bytes: upload.max_bytes,
        }
    }

    /// Whether a file `name` of `bytes` may be read, or why not.
    fn admit(self, name: &str, bytes: u64) -> Result<(), Refused> {
        let upload = self;
        let lower = name.to_ascii_lowercase();
        // NOTE: HTML Living Standard, the `accept` attribute: absent or empty, the
        // chooser offers a file of any type, and the drop zone takes the same.
        if !upload.accept.is_empty()
            && !upload
                .accept
                .iter()
                .any(|extension| lower.ends_with(&extension.to_ascii_lowercase()))
        {
            return Err(Refused::Kind {
                name: name.to_owned(),
            });
        }
        if bytes > upload.max_bytes {
            return Err(Refused::Size {
                name: name.to_owned(),
                bytes,
            });
        }
        Ok(())
    }
}

/// Returns the text of a file whose bytes are `bytes`, decoded as UTF-8
/// with a leading byte order mark dropped, and whether any byte was no
/// UTF-8, each such run written as U+FFFD.
///
/// It decodes as `Blob.text()` does (W3C File API; the Encoding Standard,
/// "UTF-8 decode"), and says when the decoding lost bytes, which
/// `Blob.text()` cannot.
#[must_use]
pub fn decode(bytes: &[u8]) -> (String, bool) {
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(bytes);
    match std::str::from_utf8(bytes) {
        Ok(text) => (text.to_owned(), false),
        Err(_) => (String::from_utf8_lossy(bytes).into_owned(), true),
    }
}

/// What the console shows in an upload field as it draws it.
#[derive(Debug, Clone, Copy, Default)]
pub struct Sending<'a> {
    /// The text, as the form last posted it or as the console opens it.
    pub text: &'a str,
    /// Whether the console is sending, which shows the in-flight line.
    pub busy: bool,
    /// What the console's upstream refused, shown verbatim.
    pub refusal: Option<&'a str>,
}

#[derive(Template)]
#[template(path = "upload/upload.html")]
struct UploadView<'a> {
    id: &'a str,
    words: String,
    accept: String,
    max_bytes: u64,
    label: &'a str,
    choose: &'a str,
    invite: &'a Word,
    rule: &'a Word,
    busy: Option<&'a Word>,
    refusal: Option<&'a str>,
    editor: Html,
    glyph: Html,
    folder: Html,
    file: Html,
    warn: Html,
    alert: Html,
}

/// Returns the upload field `upload`, drawn with what `sending` holds.
#[must_use]
pub fn upload_field(upload: &Upload, sending: Sending<'_>) -> Html {
    let text_id = format!("{}-text", upload.id);
    // The text area is the highlighted editor, its grammar read from the
    // text: a pasted or read OPT shows as XML, a JSON document as JSON.
    let editor = Editor {
        id: Cow::Owned(text_id),
        language: Language::detect(sending.text),
        text: sending.text.to_owned(),
        marks: Vec::new(),
        editable: true,
    };
    UploadView {
        id: upload.id,
        // NOTE: serde_json `to_string` fails only for a map with a key that is no
        // string, and the words hold none.
        words: serde_json::to_string(&upload.words).unwrap_or_default(),
        accept: upload.accept.join(","),
        max_bytes: upload.max_bytes,
        label: &upload.label,
        choose: &upload.choose,
        invite: &upload.words.drop_invite,
        rule: &upload.words.rule,
        busy: sending.busy.then_some(&upload.words.busy),
        refusal: sending.refusal,
        editor: drawn(Drawn {
            editor: &editor,
            placeholder: Some(upload.placeholder.to_string()),
            label: None,
            current: None,
            detect: true,
        }),
        glyph: icon(Glyph::UPLOAD, "20"),
        folder: icon(Glyph::FOLDER_OPEN, "14"),
        file: icon(Glyph::FILE_TEXT, "14"),
        warn: icon(Glyph::TRIANGLE_ALERT, "14"),
        alert: icon(Glyph::CIRCLE_ALERT, "14"),
    }
    .html()
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{Refused, SizeWords, Upload, UploadWords, admit, decode, upload_field};
    use crate::i18n::Locale;

    fn en() -> Locale {
        Locale::english()
    }

    const ACCEPT: &[&str] = &[".opt", ".xml"];
    const MAX_BYTES: u64 = 2 * 1024 * 1024;

    fn upload() -> Upload {
        Upload {
            id: "opt",
            label: Cow::Borrowed("Operational template"),
            choose: Cow::Borrowed("Choose OPT file"),
            accept: ACCEPT,
            max_bytes: MAX_BYTES,
            placeholder: Cow::Borrowed("<?xml version=\"1.0\"?>"),
            words: UploadWords::of(&en(), ACCEPT, MAX_BYTES),
        }
    }

    #[test]
    fn a_file_is_read_only_of_an_accepted_kind_and_within_the_limit() {
        assert_eq!(Ok(()), admit(&upload(), "Vital signs.OPT", 1024));
        assert_eq!(
            Err(Refused::Kind {
                name: String::from("notes.txt")
            }),
            admit(&upload(), "notes.txt", 10)
        );
        assert_eq!(
            Err(Refused::Size {
                name: String::from("big.xml"),
                bytes: 3 * 1024 * 1024
            }),
            admit(&upload(), "big.xml", 3 * 1024 * 1024)
        );
    }

    #[test]
    fn a_field_that_names_no_kind_takes_any_file() {
        let any = Upload {
            accept: &[],
            words: UploadWords::of(&en(), &[], MAX_BYTES),
            ..upload()
        };
        assert_eq!(Ok(()), admit(&any, "notes.txt", 10));
        assert_eq!(Ok(()), admit(&any, "no extension", 10));
        assert!(
            admit(&any, "big.bin", MAX_BYTES + 1).is_err(),
            "the limit holds"
        );
        assert_eq!("Any file, up to 2 MiB.", any.words.rule);
    }

    #[test]
    fn a_file_is_decoded_as_utf8_and_a_lossy_decoding_is_flagged() {
        assert_eq!(
            (String::from("caf\u{e9}"), false),
            decode("caf\u{e9}".as_bytes())
        );
        assert_eq!(
            (String::from("<x/>"), false),
            decode(b"\xEF\xBB\xBF<x/>"),
            "the byte order mark is dropped, as Blob.text() drops it"
        );
        assert_eq!(
            (String::from("caf\u{FFFD}"), true),
            decode(b"caf\xE9"),
            "Latin-1 bytes are no UTF-8"
        );
    }

    #[test]
    fn a_refusal_names_the_file_and_the_rule() {
        let kind = Refused::Kind {
            name: String::from("notes.txt"),
        };
        let words = UploadWords::of(&en(), ACCEPT, MAX_BYTES);
        assert_eq!(
            "notes.txt was not read: the field takes .opt, .xml files.",
            kind.sentence(&words)
        );
        let size = Refused::Size {
            name: String::from("big.xml"),
            bytes: 3 * 1024 * 1024,
        };
        assert_eq!(
            "big.xml was not read: it is 3 MiB and the field takes up to 2 MiB.",
            size.sentence(&words)
        );
        assert_eq!(".opt, .xml files, up to 2 MiB.", words.rule);
    }

    #[test]
    fn the_server_draws_the_zone_the_chooser_the_rule_and_the_editor() {
        let html = upload_field(&upload(), super::Sending::default()).to_string();
        for part in [
            "id=\"opt-drop\"",
            "data-fui=\"upload\"",
            "id=\"opt-file\" type=\"file\"",
            "accept=\".opt,.xml\"",
            "<label for=\"opt-file\"",
            "<label for=\"opt-text\" class=\"fui-label\">Operational template</label>",
            "name=\"opt-text\"",
            "data-fui-detect",
            ".opt, .xml files, up to 2 MiB.",
        ] {
            assert!(html.contains(part), "{part}: {html}");
        }
        assert!(!html.contains("fui-spinner"), "nothing in flight");
        assert!(!html.contains("fui-alert"), "nothing refused");
    }

    #[test]
    fn the_console_shows_its_send_and_its_upstream_refusal() {
        let html = upload_field(
            &upload(),
            super::Sending {
                text: "{\"a\": 1}",
                busy: true,
                refusal: Some("the server refused <it>"),
            },
        )
        .to_string();
        assert!(html.contains("fui-spinner"), "{html}");
        assert!(
            html.contains(
                "<div role=\"alert\" class=\"fui-alert\">the server refused &#60;it&#62;</div>"
            ),
            "{html}"
        );
        assert!(html.contains("data-language=\"json\""), "{html}");
    }

    #[test]
    fn a_size_takes_the_largest_unit_above_one_and_its_plural() {
        let sizes = SizeWords::of(&en());
        assert_eq!("2 MiB", sizes.size(MAX_BYTES));
        assert_eq!("12 bytes", sizes.size(12));
        assert_eq!("1 byte", sizes.size(1));
        assert_eq!("0 bytes", sizes.size(0));
        assert_eq!("2 KiB", sizes.size(1500));
    }
}
