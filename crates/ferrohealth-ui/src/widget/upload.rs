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

use leptos::prelude::*;

use crate::highlight::Language;
use crate::icon::{Glyph, icon};
use crate::widget::editor::{Drawn, Grammar, drawn};

/// What an upload field accepts and says.
#[derive(Debug, Clone, Copy)]
pub struct Upload {
    /// The field's id: the file input is `id` and `-file`, the text area
    /// `id` and `-text`, the drop zone `id` and `-drop`.
    pub id: &'static str,
    /// The field's label, such as "Operational template".
    pub label: &'static str,
    /// The chooser's words, such as "Choose OPT file".
    pub choose: &'static str,
    /// The extensions accepted, with their dots, such as `[".opt", ".xml"]`.
    pub accept: &'static [&'static str],
    /// The largest file read, in bytes.
    pub max_bytes: u64,
    /// The text area's placeholder: the format's first line.
    pub placeholder: &'static str,
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
    /// The sentence the field shows for the refusal, naming `accept` and the
    /// limit `max_bytes`.
    #[must_use]
    pub fn sentence(&self, accept: &[&str], max_bytes: u64) -> String {
        match self {
            Self::Kind { name } => format!(
                "{name} was not read: the field takes {} files.",
                accept.join(", ")
            ),
            Self::Size { name, bytes } => format!(
                "{name} was not read: it is {} and the field takes up to {}.",
                size(*bytes),
                size(max_bytes)
            ),
        }
    }
}

/// `bytes` in the largest unit that keeps it above one, in words.
#[must_use]
pub fn size(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * 1024;
    if bytes >= MIB {
        format!("{} MiB", bytes.div_ceil(MIB))
    } else if bytes >= KIB {
        format!("{} KiB", bytes.div_ceil(KIB))
    } else {
        format!("{bytes} bytes")
    }
}

/// Whether a file `name` of `bytes` may be read under `upload`, or why not.
///
/// # Errors
/// Returns the [`Refused`] reason of a file of another kind or too large.
pub fn admit(upload: &Upload, name: &str, bytes: u64) -> Result<(), Refused> {
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
    let text_id = format!("{}-text", upload.id);
    let label_for = text_id.clone();
    let state = move || match picked.get() {
        Picked::None => None,
        Picked::Read(name) => Some(
            view! {
                <p class="fui-drop-state" role="status">
                    {icon(Glyph::FILE_TEXT, "14")}
                    {format!("{name} read; check the text below before you send it.")}
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
    let zone = drop_zone(upload, source, picked);
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
            placeholder: Some(upload.placeholder),
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
                                "Uploading…"
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
fn drop_zone(upload: Upload, source: RwSignal<String>, picked: RwSignal<Picked>) -> impl IntoView {
    let over = RwSignal::new(false);
    let file_id = format!("{}-file", upload.id);
    let drop_id = format!("{}-drop", upload.id);
    let accept = upload.accept.join(",");
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
                    dropped(upload, &event, source, picked);
                }
            >
                <span class="fui-drop-glyph">{icon(Glyph::UPLOAD, "20")}</span>
                <p class="fui-drop-words">
                    {move || {
                        if over.get() {
                            "Drop to read the file"
                        } else {
                            "Drop a file here, or"
                        }
                    }}
                </p>
                <label for=input_for class="fui-button fui-button-secondary fui-button-small">
                    {icon(Glyph::FOLDER_OPEN, "14")}
                    {upload.choose}
                </label>
                <input
                    id=file_id
                    type="file"
                    class="fui-sr-only"
                    accept=accept
                    on:change=move |event| chosen(upload, &event, source, picked)
                />
                <p class="fui-small">
                    {format!(
                        "{} files, up to {}.",
                        upload.accept.join(", "),
                        size(upload.max_bytes),
                    )}
                </p>
            </div>
    }
}

/// Reads the file `event` chose into `source`, or refuses it.
fn chosen(
    upload: Upload,
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
            read(upload, &file, source, picked);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        Upload,
        &leptos::ev::Event,
        RwSignal<String>,
        RwSignal<Picked>,
    ) = (upload, event, source, picked);
}

/// Reads the file `event` dropped into `source`, or refuses it.
fn dropped(
    upload: Upload,
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
            read(upload, &file, source, picked);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: (
        Upload,
        &leptos::ev::DragEvent,
        RwSignal<String>,
        RwSignal<Picked>,
    ) = (upload, event, source, picked);
}

/// Reads `file` into `source` when [`admit`] lets it, and says so in
/// `picked`.
#[cfg(target_arch = "wasm32")]
fn read(upload: Upload, file: &web_sys::File, source: RwSignal<String>, picked: RwSignal<Picked>) {
    let name = file.name();
    if let Err(refused) = admit(&upload, &name, whole_bytes(file.size())) {
        picked.set(Picked::Refused(
            refused.sentence(upload.accept, upload.max_bytes),
        ));
        return;
    }
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
            None => picked.set(Picked::Refused(format!(
                "{name} was not read: the browser could not read it."
            ))),
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
    use super::{Refused, Upload, admit, size};

    const UPLOAD: Upload = Upload {
        id: "opt",
        label: "Operational template",
        choose: "Choose OPT file",
        accept: &[".opt", ".xml"],
        max_bytes: 2 * 1024 * 1024,
        placeholder: "<?xml version=\"1.0\"?>",
    };

    #[test]
    fn a_file_is_read_only_of_an_accepted_kind_and_within_the_limit() {
        assert_eq!(Ok(()), admit(&UPLOAD, "Vital signs.OPT", 1024));
        assert_eq!(
            Err(Refused::Kind {
                name: String::from("notes.txt")
            }),
            admit(&UPLOAD, "notes.txt", 10)
        );
        assert_eq!(
            Err(Refused::Size {
                name: String::from("big.xml"),
                bytes: 3 * 1024 * 1024
            }),
            admit(&UPLOAD, "big.xml", 3 * 1024 * 1024)
        );
    }

    #[test]
    fn a_refusal_names_the_file_and_the_rule() {
        let kind = Refused::Kind {
            name: String::from("notes.txt"),
        };
        assert_eq!(
            "notes.txt was not read: the field takes .opt, .xml files.",
            kind.sentence(UPLOAD.accept, UPLOAD.max_bytes)
        );
        assert_eq!("2 MiB", size(UPLOAD.max_bytes));
        assert_eq!("12 bytes", size(12));
        assert_eq!("2 KiB", size(1500));
    }
}
