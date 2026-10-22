// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overlays and the feedback.
//!
//! They are the confirmation dialogs, the modal
//! dialog a signal opens, the side panel, a toast in every tone and the
//! popover form. The user menu, which sets the theme, is in the top bar,
//! and the drawer is the navigation on a narrow screen.

use crate::locale::{Msg, Words, tr};
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog, confirm_trigger, danger_dialog};
use ferrohealth_ui::view::form::{INPUT, field};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::popover::{PopoverForm, popover_form};
use ferrohealth_ui::view::surface::{
    card, chip, cluster, definitions, grid, page_header, split, stack,
};
use ferrohealth_ui::widget::dialog::{Modal, ModalWords, Width, modal_dialog};
use ferrohealth_ui::widget::side_panel::{
    SidePanel, SidePanelWords, side_panel, side_panel_trigger,
};
use ferrohealth_ui::widget::toast::{Toast, ToastAction, ToastLink, raise};
use leptos::prelude::*;

/// The confirmation dialogs a trigger opens with no script: the plain one
/// and the danger one, whose confirm, the island
/// [`crate::islands::ConfirmAndToast`], raises a toast.
fn confirmations() -> AnyView {
    let plain = confirm_dialog(
        Confirmation {
            id: "reload-set",
            title: tr(Msg::ReloadTheSet).into(),
            message: tr(Msg::FilesDiskCompileReplaceRunning).into(),
        },
        view! { <crate::islands::ConfirmAndToast dialog="reload-set".to_owned() danger=false words=confirm_words() /> },
    )
    .into_any();
    let danger = danger_dialog(
        Confirmation {
            id: "delete-file",
            title: tr(Msg::DeleteTheFile).into(),
            message: tr(Msg::SyntheticFileGoesGoodRunning).into(),
        },
        view! { <crate::islands::ConfirmAndToast dialog="delete-file".to_owned() danger=true words=confirm_words() /> },
    )
    .into_any();
    card(
        tr(Msg::ConfirmationDialogs),
        view! {
            <p class="fui-muted">
                {tr(Msg::OpenedTheirTriggerNoScript)}
            </p>
            {cluster(
                view! {
                    {confirm_trigger("reload-set", Variant::Secondary, Glyph::REFRESH_CW, tr(Msg::ReloadSet))}
                    {confirm_trigger("delete-file", Variant::Danger, Glyph::TRASH, tr(Msg::DeleteFile))}
                },
            )}
            {plain}
            {danger}
        },
    )
    .into_any()
}

/// The words of a confirm button's island.
fn confirm_words() -> Words {
    Words::of(&[
        Msg::SyntheticFileWasDeleted,
        Msg::Delete,
        Msg::SetReloaded,
        Msg::Reload,
    ])
}

/// The confirm button of the confirmation dialog `dialog` in `words`, the
/// danger one when `danger`: it raises a toast and closes the dialog.
pub(crate) fn confirm_and_toast(dialog: String, danger: bool, words: &Words) -> AnyView {
    let (said, class, glyph, words) = if danger {
        (
            words.get(Msg::SyntheticFileWasDeleted),
            Variant::Destructive.class(),
            Glyph::TRASH,
            words.get(Msg::Delete),
        )
    } else {
        (
            words.get(Msg::SetReloaded),
            Variant::Primary.class(),
            Glyph::REFRESH_CW,
            words.get(Msg::Reload),
        )
    };
    let id = format!("confirm-{dialog}");
    view! {
        <button
            type="button"
            id=id
            class=class
            on:click=move |_| {
                raise(Toast::new(Tone::Ok, said.clone()));
                hide(&dialog);
            }
        >
            {icon(glyph, "16")}
            {words}
        </button>
    }
    .into_any()
}

/// The modal dialogs a signal opens, drawn by the island
/// [`crate::islands::ModalDialogs`].
fn modals() -> AnyView {
    card(
        tr(Msg::ModalDialogs),
        view! { <crate::islands::ModalDialogs words=modal_words() modal=ModalWords::of_page() /> },
    )
    .into_any()
}

/// The words of the modal dialogs' island.
fn modal_words() -> Words {
    Words::of(&[
        Msg::Delete,
        Msg::TheFile,
        Msg::FileGoesGoodRunningSet,
        Msg::Files,
        Msg::RenameTheFile,
        Msg::RenameFile,
        Msg::NewName,
        Msg::PathBelowFhirconnectOmocl,
        Msg::Rename,
    ])
    .open(Msg::DeleteRow, &["row"])
    .open(Msg::RowDeleted, &["row"])
}

/// The modal dialog a signal opens, in `words` and the dialogs' own
/// `modal`: a row's Delete names the row, and a wide form.
pub(crate) fn modal_dialogs(words: Words, modal: ModalWords) -> AnyView {
    let words = StoredValue::new(words);
    let pending = RwSignal::new(None::<&'static str>);
    let renaming = RwSignal::new(false);
    let rows = ["synthetic-007.model.yml", "synthetic-011.extension.yml"]
        .into_iter()
        .map(|row| {
            split(
                view! { <code>{row}</code> },
                view! {
                    <button
                        type="button"
                        class=Variant::Danger.small()
                        data-delete=row
                        on:click=move |_| pending.set(Some(row))
                    >
                        {icon(Glyph::TRASH, "14")}
                        {words.with_value(|words| words.get(Msg::Delete))}
                    </button>
                },
            )
            .into_any()
        })
        .collect();
    let delete = modal_dialog(
        Modal {
            id: "delete-row",
            title: Signal::derive(move || {
                words.with_value(|words| {
                    let row = pending
                        .get()
                        .map_or_else(|| words.get(Msg::TheFile), str::to_owned);
                    words.fill(Msg::DeleteRow, &[("row", &row)])
                })
            }),
            danger: true,
            width: Width::Narrow,
            body: view! {
                <p>{words.with_value(|words| words.get(Msg::FileGoesGoodRunningSet))}</p>
            }
            .into_any(),
            actions: view! {
                <button
                    type="button"
                    id="confirm-delete-row"
                    class=Variant::Destructive.class()
                    on:click=move |_| {
                        if let Some(row) = pending.get_untracked() {
                            let said = words.with_value(|words| words.fill(Msg::RowDeleted, &[("row", row)]));
                            raise(Toast::new(Tone::Ok, said));
                        }
                        pending.set(None);
                    }
                >
                    {icon(Glyph::TRASH, "16")}
                    {words.with_value(|words| words.get(Msg::Delete))}
                </button>
            }
            .into_any(),
            words: modal.clone(),
        },
        Signal::derive(move || pending.get().is_some()),
        Callback::new(move |()| pending.set(None)),
    )
    .into_any();
    let rename = words.with_value(|words| rename_modal(renaming, words, modal));
    let (files, rename_the_file) =
        words.with_value(|words| (words.get(Msg::Files), words.get(Msg::RenameTheFile)));
    view! {
        {divided_list(Some(files.into()), rows)}
        <button type="button" class=Variant::Secondary.class() id="open-rename" on:click=move |_| renaming.set(true)>
            {label(Some(Glyph::PENCIL), rename_the_file)}
        </button>
        {delete}
        {rename}
    }
    .into_any()
}

/// The wide modal form `renaming` opens, in `words` and `modal`.
fn rename_modal(renaming: RwSignal<bool>, words: &Words, modal: ModalWords) -> AnyView {
    let name = RwSignal::new(String::from("synthetic-007.model.yml"));
    modal_dialog(
        Modal {
            id: "rename-file",
            title: Signal::stored(words.get(Msg::RenameFile)),
            danger: false,
            width: Width::Wide,
            body: field(
                "rename-name",
                words.get(Msg::NewName),
                Some(words.get(Msg::PathBelowFhirconnectOmocl)),
                view! {
                    <input
                        id="rename-name"
                        type="text"
                        class=INPUT
                        prop:value=move || name.get()
                        on:input:target=move |event| name.set(event.target().value())
                    />
                },
            )
            .into_any(),
            actions: view! {
                <button type="button" class=Variant::Primary.class() on:click=move |_| renaming.set(false)>
                    {icon(Glyph::SAVE, "16")}
                    {words.get(Msg::Rename)}
                </button>
            }
            .into_any(),
            words: modal,
        },
        renaming.into(),
        Callback::new(move |()| renaming.set(false)),
    )
    .into_any()
}

/// The side panel a button opens, holding the session's access.
fn panel() -> AnyView {
    let body = stack(view! {
        {definitions(vec![
            (tr(Msg::Signed), tr(Msg::SyntheticOperator).into_any()),
            (tr(Msg::Method), tr(Msg::OpenidConnect).into_any()),
            (tr(Msg::Policy), tr(Msg::OperatorScopeDecidesWhatConsole).into_any()),
        ])}
        <h3>{tr(Msg::Scopes)}</h3>
        {cluster(
            view! {
                {chip("ferrobridge.operator")}
                {chip("openid")}
                {chip("profile")}
            },
        )}
    })
    .into_any();
    card(
        tr(Msg::SidePanel),
        view! {
            <p class="fui-muted">{tr(Msg::ContentNoNavigationFromEnd)}</p>
            {side_panel_trigger("access", Variant::Secondary.class(), Glyph::SHIELD_CHECK, tr(Msg::ViewAccess))}
            <crate::islands::AccessPanel title=tr(Msg::Access) words=SidePanelWords::of_page()>
                {body}
            </crate::islands::AccessPanel>
        },
    )
    .into_any()
}

/// The side panel `access` titled `title`, holding `body`, which the
/// server drew, in the panel's own `words`.
pub(crate) fn access_panel(title: String, words: SidePanelWords, body: AnyView) -> AnyView {
    side_panel(SidePanel {
        id: "access",
        body_id: "access-drawer",
        title: title.into(),
        body,
        words,
    })
}

/// A button per tone that raises a toast in it, drawn by the island
/// [`crate::islands::ToastButtons`].
fn toasts() -> AnyView {
    let words = Words::of(&[
        Msg::Success,
        Msg::SavedSetReloaded,
        Msg::ThreeFilesCompiledFacadeServes,
        Msg::OpenTheTables,
        Msg::Information,
        Msg::SetReloadsBackground,
        Msg::Warning,
        Msg::SetCompiled2Warnings,
        Msg::TwoMappingsNameMissingPath,
        Msg::Refusal,
        Msg::SaveWasRefused412,
        Msg::SomeoneSavedTheFileFirst,
        Msg::TryAgain,
    ]);
    let tables = super::overview::brand_page("tables").unwrap_or_default();
    card(
        tr(Msg::Toasts),
        view! { <crate::islands::ToastButtons words tables /> },
    )
    .into_any()
}

/// A button per tone that raises a toast in it, in `words`: a title, a line
/// under it where there is more to say, and an action, the success toast's
/// leading to `tables` and the refusal's trying again.
pub(crate) fn toast_buttons(words: &Words, tables: String) -> AnyView {
    let saved = words.get(Msg::SavedSetReloaded);
    let again = Toast::new(Tone::Ok, saved.clone());
    let toasts = [
        (
            words.get(Msg::Success),
            Toast::new(Tone::Ok, saved)
                .line(words.get(Msg::ThreeFilesCompiledFacadeServes))
                .action(ToastAction::Link(ToastLink {
                    label: words.get(Msg::OpenTheTables),
                    href: tables,
                })),
        ),
        (
            words.get(Msg::Information),
            Toast::new(Tone::Info, words.get(Msg::SetReloadsBackground)),
        ),
        (
            words.get(Msg::Warning),
            Toast::new(Tone::Warn, words.get(Msg::SetCompiled2Warnings))
                .line(words.get(Msg::TwoMappingsNameMissingPath)),
        ),
        (
            words.get(Msg::Refusal),
            Toast::new(Tone::Bad, words.get(Msg::SaveWasRefused412))
                .line(words.get(Msg::SomeoneSavedTheFileFirst))
                .action(ToastAction::Run {
                    label: words.get(Msg::TryAgain),
                    run: Callback::new(move |()| raise(again.clone())),
                }),
        ),
    ];
    let buttons = toasts
        .into_iter()
        .map(|(label_words, toast)| {
            let tone = toast.tone;
            view! {
                <button
                    type="button"
                    class=Variant::Secondary.class()
                    data-toast=tone.name()
                    on:click=move |_| raise(toast.clone())
                >
                    {label(Some(Glyph::BELL), label_words)}
                </button>
            }
        })
        .collect_view();
    cluster(buttons).into_any()
}

/// The popover form: Save the query, a small form that opens under its
/// trigger and posts with no script.
fn popover() -> AnyView {
    let body = view! {
        <form method="get" action="#popover-form" class="fui-form">
            {field(
                "query-name",
                tr(Msg::Name),
                None,
                view! { <input id="query-name" name="name" type="text" class=INPUT value="synthetic-vitals" /> },
            )}
            {field(
                "query-version",
                tr(Msg::Version),
                None,
                view! { <input id="query-version" name="version" type="text" class=INPUT value="1.0.0" /> },
            )}
            <div class="fui-dialog-actions">
                <button type="submit" class=Variant::Primary.class()>
                    {label(Some(Glyph::SAVE), tr(Msg::Save))}
                </button>
            </div>
        </form>
    }
    .into_any();
    card(
        tr(Msg::PopoverForm),
        view! {
            <p class="fui-muted" id="popover-form">{tr(Msg::PopoverFormOpensUnderTrigger)}</p>
            {cluster(popover_form(PopoverForm {
                id: "save-query",
                variant: Variant::Secondary,
                glyph: Glyph::SAVE,
                label: tr(Msg::SaveTheQuery).into(),
                title: tr(Msg::SaveTheQueryToTheServer).into(),
                body,
            }))}
        },
    )
    .into_any()
}

/// Closes the popover `id`.
fn hide(id: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::wasm_bindgen::JsCast as _;
        let element = document()
            .get_element_by_id(id)
            .and_then(|element| element.dyn_into::<web_sys::HtmlElement>().ok());
        if let Some(element) = element
            && element.hide_popover().is_err()
        {
            leptos::logging::warn!("the browser refused to close the dialog");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _: &str = id;
}

/// The overlays page.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn OverlaysPage() -> impl IntoView {
    let header = page_header(
        tr(Msg::OverlaysFeedback),
        Some(tr(Msg::DialogsModalsSidePanelToasts)),
        (),
    )
    .into_any();
    let sections = (confirmations(), modals(), panel(), toasts(), popover());
    view! {
        {super::title(tr(Msg::OverlaysFeedback))}
        {header}
        {stack(view! {
            {grid(view! {
                {sections.0}
                {sections.2}
            })}
            {sections.1}
            {grid(view! {
                {sections.3}
                {sections.4}
            })}
        })}
    }
}
