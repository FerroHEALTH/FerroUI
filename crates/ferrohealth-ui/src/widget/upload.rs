// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The upload field: a file dropped or chosen, or its text pasted, read in
//! the browser into the highlighted editor, where it can be read and
//! corrected before the console sends it.
//!
//! The drop zone takes a file dragged onto it and reads "Drop to read the
//! file" while one is over it; the button beside its words opens the
//! browser's file chooser. A file is refused, with the reason shown and
//! announced, when its name ends in none of the accepted extensions or it
//! is larger than the limit. A file read replaces the text, and the field
//! names it. The library sends nothing: the console posts the text, and
//! shows its in-flight state and its refusal in the slots here. The file
//! is read with `Blob.text()` in a handler (W3C File API), so the server
//! draws the field ready and nothing runs before hydration.

use std::borrow::Cow;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::highlight::Language;
use crate::i18n::{Phrase, Text, tr, tr_with};
use crate::icon::{Glyph, icon};
use crate::widget::editor::{Drawn, Grammar, drawn};

/// What an upload field accepts and says.
#[derive(Debug, Clone)]
pub struct Upload {
    /// The field's id: the file input is `id` and `-file`, the text area
    /// `id` and `-text`, the drop zone `id` and `-drop`.
    pub id: &'static str,
    /// The field's label, such as "Operational template".
    pub label: Cow<'static, str>,
    /// The chooser's words, such as "Choose OPT file".
    pub choose: Cow<'static, str>,
    /// The extensions accepted, with their dots, such as `[".opt", ".xml"]`.
    pub accept: &'static [&'static str],
    /// The largest file read, in bytes.
    pub max_bytes: u64,
    /// The text area's placeholder: the format's first line.
    pub placeholder: Cow<'static, str>,
    /// The words the field draws itself ([`UploadWords::of_page`] over the
    /// same `accept` and `max_bytes`).
    pub words: UploadWords,
}

/// A size in words, resolved on the server in the page's locale with the
/// count left open, so an island can name the size of a file it reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SizeWords {
    /// One byte, whole.
    pub one_byte: String,
    /// A count of bytes other than one, around the open `count`.
    pub bytes: Phrase,
    /// A count of kibibytes, around the open `count`.
    pub kib: Phrase,
    /// A count of mebibytes, around the open `count`.
    pub mib: Phrase,
}

impl SizeWords {
    /// The size words in the locale of the page being drawn.
    #[must_use]
    pub fn of_page() -> Self {
        let open = |text| Phrase::new(tr_with(text, &[("count", Phrase::open("count"))]));
        Self {
            one_byte: tr_with(Text::SizeBytes, &[("count", 1_u32.into())]),
            bytes: open(Text::SizeBytes),
            kib: open(Text::SizeKib),
            mib: open(Text::SizeMib),
        }
    }

    /// `bytes` in the largest unit that keeps it above one.
    #[must_use]
    pub fn size(&self, bytes: u64) -> String {
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
        unit.fill(&[("count", &count.to_string())])
    }
}

/// The words an upload field draws itself, resolved on the server in the
/// page's locale, so an island that draws it carries no catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadWords {
    /// The drop zone's words.
    pub drop_invite: String,
    /// The drop zone's words while a file is over it.
    pub drop_over: String,
    /// The kinds of file and the limit the field takes.
    pub rule: String,
    /// The line while the console sends.
    pub busy: String,
    /// The line once a file is read, around the open `file`.
    pub read: Phrase,
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
    /// `max_bytes`, in the locale of the page being drawn.
    #[must_use]
    pub fn of_page(accept: &[&str], max_bytes: u64) -> Self {
        let sizes = SizeWords::of_page();
        let kinds = accept.join(", ");
        let limit = sizes.size(max_bytes);
        let file = || ("file", Phrase::open("file"));
        Self {
            drop_invite: tr(Text::UploadDropInvite),
            drop_over: tr(Text::UploadDropOver),
            rule: tr_with(
                Text::UploadRule,
                &[
                    ("kinds", kinds.clone().into()),
                    ("limit", limit.clone().into()),
                ],
            ),
            busy: tr(Text::UploadBusy),
            read: Phrase::new(tr_with(Text::UploadRead, &[file()])),
            refused_kind: Phrase::new(tr_with(
                Text::UploadRefusedKind,
                &[file(), ("kinds", kinds.into())],
            )),
            refused_size: Phrase::new(tr_with(
                Text::UploadRefusedSize,
                &[
                    file(),
                    ("size", Phrase::open("size")),
                    ("limit", limit.into()),
                ],
            )),
            unreadable: Phrase::new(tr_with(Text::UploadUnreadable, &[file()])),
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
    pub fn sentence(&self, words: &UploadWords) -> String {
        match self {
            Self::Kind { name } => words.refused_kind.fill(&[("file", name)]),
            Self::Size { name, bytes } => words
                .refused_size
                .fill(&[("file", name), ("size", &words.sizes.size(*bytes))]),
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
/// handlers that read a file take with them.
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
        if !upload
            .accept
            .iter()
            .any(|extension| lower.ends_with(extension))
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

/// What the field is doing.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "the browser half alone reads a file")
)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum Picked {
    /// No file read.
    #[default]
    None,
    /// The file named was read into the text.
    Read(String),
    /// The file was refused, with the sentence that says why.
    Refused(String),
}

/// Returns the upload field `upload`, its text in `source`; `busy` reads
/// true while the console sends, and `refusal` holds what the console's
/// upstream refused, shown verbatim.
#[must_use]
pub fn upload_field(
    upload: Upload,
    source: RwSignal<String>,
    busy: Signal<bool>,
    refusal: Signal<Option<String>>,
) -> AnyView {
    let picked = RwSignal::new(Picked::None);
    let words = StoredValue::new(upload.words.clone());
    let uploading = upload.words.busy.clone();
    let text_id = format!("{}-text", upload.id);
    let label_for = text_id.clone();
    let state = move || match picked.get() {
        Picked::None => None,
        Picked::Read(name) => Some(
            view! {
                <p class="fui-drop-state" role="status">
                    {icon(Glyph::FILE_TEXT, "14")}
                    {words.with_value(|words| words.read.fill(&[("file", &name)]))}
                </p>
            }
            .into_any(),
        ),
        Picked::Refused(sentence) => Some(
            view! {
                <p class="fui-drop-state fui-drop-state-refused" role="alert">
                    {icon(Glyph::CIRCLE_ALERT, "14")}
                    {sentence}
                </p>
            }
            .into_any(),
        ),
    };
    let zone = drop_zone(&upload, words, source, picked);
    // The text area is the highlighted editor, its grammar read from the
    // text: a pasted or read OPT shows as XML, a JSON document as JSON.
    let editor = drawn(
        Drawn {
            id: text_id,
            language: Grammar::Following(Signal::derive(move || {
                source.with(|text| Language::detect(text))
            })),
            text: source.into(),
            marks: Signal::stored(Vec::new()),
            editable: Signal::stored(true),
            placeholder: Some(upload.placeholder.clone()),
        },
        move |text| source.set(text),
    );
    view! {
        <div class="fui-upload">
            {zone} {state} <div class="fui-field">
                <label for=label_for class="fui-label">
                    {upload.label}
                </label>
                {editor}
            </div>
            {move || {
                busy.get()
                    .then(|| {
                        view! {
                            <p class="fui-spinner" role="status">
                                <span class="fui-spinner-ring" aria-hidden="true"></span>
                                {uploading.clone()}
                            </p>
                        }
                    })
            }}
            {move || {
                refusal
                    .get()
                    .map(|text| {
                        view! {
                            <div role="alert" class="fui-alert">
                                {text}
                            </div>
                        }
                    })
            }}
        </div>
    }
    .into_any()
}

/// Returns the drop zone of `upload`: the words, the chooser and the rule,
/// reading a file dropped or chosen into `source` and saying so in
/// `picked`.
fn drop_zone(
    upload: &Upload,
    words: StoredValue<UploadWords>,
    source: RwSignal<String>,
    picked: RwSignal<Picked>,
) -> impl IntoView + use<> {
    let over = RwSignal::new(false);
    let release = upload.words.drop_over.clone();
    let invite = upload.words.drop_invite.clone();
    let rule = upload.words.rule.clone();
    let file_id = format!("{}-file", upload.id);
    let drop_id = format!("{}-drop", upload.id);
    let accept = upload.accept.join(",");
    let kinds = Rule::of(upload);
    let choose = upload.choose.clone();
    let input_for = file_id.clone();
    let zone_class = move || {
        let mut class = String::from("fui-drop");
        if over.get() {
            class.push_str(" fui-drop-over");
        }
        if matches!(picked.get(), Picked::Refused(_)) {
            class.push_str(" fui-drop-refused");
        }
        class
    };
    view! {
        <div
                id=drop_id
                class=zone_class
                on:dragover=move |event| {
                    event.prevent_default();
                    over.set(true);
                }
                on:dragleave=move |_| over.set(false)
                on:drop=move |event| {
                    event.prevent_default();
                    over.set(false);
                    dropped(kinds, words, &event, source, picked);
                }
            >
                <span class="fui-drop-glyph">{icon(Glyph::UPLOAD, "20")}</span>
                <p class="fui-drop-words">
                    {move || if over.get() { release.clone() } else { invite.clone() }}
                </p>
                <label for=input_for class="fui-button fui-button-secondary fui-button-small">
                    {icon(Glyph::FOLDER_OPEN, "14")}
                    {choose}
                </label>
                <input
                    id=file_id
                    type="file"
                    class="fui-sr-only"
                    accept=accept
                    on:change=move |event| chosen(kinds, words, &event, source, picked)
                />
                <p class="fui-small">{rule}</p>
            </div>
    }
}

/// Reads the file `event` chose into `source`, or refuses it.
fn chosen(
    upload: Rule,
    words: StoredValue<UploadWords>,
    event: &leptos::ev::Event,
    source: RwSignal<String>,
    picked: RwSignal<Picked>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let file = event
            .target()
            .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
            .and_then(|input| input.files())
            .and_then(|files| files.get(0));
        if let Some(file) = file {
            read(upload, words, &file, source, picked);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        Rule,
        StoredValue<UploadWords>,
        &leptos::ev::Event,
        RwSignal<String>,
        RwSignal<Picked>,
    ) = (upload, words, event, source, picked);
}

/// Reads the file `event` dropped into `source`, or refuses it.
fn dropped(
    upload: Rule,
    words: StoredValue<UploadWords>,
    event: &leptos::ev::DragEvent,
    source: RwSignal<String>,
    picked: RwSignal<Picked>,
) {
    #[cfg(target_arch = "wasm32")]
    {
        let file = event
            .data_transfer()
            .and_then(|transfer| transfer.files())
            .and_then(|files| files.get(0));
        if let Some(file) = file {
            read(upload, words, &file, source, picked);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        Rule,
        StoredValue<UploadWords>,
        &leptos::ev::DragEvent,
        RwSignal<String>,
        RwSignal<Picked>,
    ) = (upload, words, event, source, picked);
}

/// Reads `file` into `source` when [`admit`] lets it, and says so in
/// `picked`.
#[cfg(target_arch = "wasm32")]
fn read(
    upload: Rule,
    words: StoredValue<UploadWords>,
    file: &web_sys::File,
    source: RwSignal<String>,
    picked: RwSignal<Picked>,
) {
    let name = file.name();
    if let Err(refused) = upload.admit(&name, whole_bytes(file.size())) {
        picked.set(Picked::Refused(
            words.with_value(|words| refused.sentence(words)),
        ));
        return;
    }
    let unreadable = words.with_value(|words| words.unreadable.fill(&[("file", &name)]));
    let promise = file.text();
    wasm_bindgen_futures::spawn_local(async move {
        let read = wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .ok()
            .and_then(|text| text.as_string());
        match read {
            Some(text) => {
                source.set(text);
                picked.set(Picked::Read(name));
            }
            None => picked.set(Picked::Refused(unreadable)),
        }
    });
}

/// The byte count of a file the browser reports as `size`.
///
/// W3C File API, the `size` attribute: a whole byte count, held as a double
/// below 2^53, so the conversion is exact; a size that is not finite or is
/// negative is no size, and reads as the largest, which the limit refuses.
#[cfg(target_arch = "wasm32")]
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "checked finite and non-negative first, and a File API size is a whole count below 2^53, so the cast is exact"
)]
fn whole_bytes(size: f64) -> u64 {
    if size.is_finite() && size >= 0.0 {
        size as u64
    } else {
        u64::MAX
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::{Refused, SizeWords, Upload, UploadWords, admit};

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
            words: UploadWords::of_page(ACCEPT, MAX_BYTES),
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
    fn a_refusal_names_the_file_and_the_rule() {
        let kind = Refused::Kind {
            name: String::from("notes.txt"),
        };
        let words = UploadWords::of_page(ACCEPT, MAX_BYTES);
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
    fn a_size_takes_the_largest_unit_above_one_and_its_plural() {
        let sizes = SizeWords::of_page();
        assert_eq!("2 MiB", sizes.size(MAX_BYTES));
        assert_eq!("12 bytes", sizes.size(12));
        assert_eq!("1 byte", sizes.size(1));
        assert_eq!("0 bytes", sizes.size(0));
        assert_eq!("2 KiB", sizes.size(1500));
    }
}
