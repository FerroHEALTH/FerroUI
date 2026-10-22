// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overlays and the feedback.
//!
//! They are the confirmation dialogs, the modal
//! dialog a signal opens, the side panel, and a toast in every tone. The
//! user menu, which sets the theme, is in the top bar, and the drawer is
//! the navigation on a narrow screen.

use crate::locale::{Msg, tr, tr_with};
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::dialog::{Confirmation, confirm_dialog, confirm_trigger, danger_dialog};
use ferrohealth_ui::view::form::{INPUT, field};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::surface::{
    card, chip, cluster, definitions, grid, page_header, split, stack,
};
use ferrohealth_ui::widget::dialog::{Modal, Width, modal_dialog};
use ferrohealth_ui::widget::side_panel::{SidePanel, side_panel, side_panel_trigger};
use ferrohealth_ui::widget::toast::raise;
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
        view! { <crate::islands::ConfirmAndToast dialog="reload-set".to_owned() danger=false /> },
    )
    .into_any();
    let danger = danger_dialog(
        Confirmation {
            id: "delete-file",
            title: tr(Msg::DeleteTheFile).into(),
            message: tr(Msg::SyntheticFileGoesGoodRunning).into(),
        },
        view! { <crate::islands::ConfirmAndToast dialog="delete-file".to_owned() danger=true /> },
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

/// The confirm button of the confirmation dialog `dialog`, the danger one
/// when `danger`: it raises a toast and closes the dialog.
pub(crate) fn confirm_and_toast(dialog: String, danger: bool) -> AnyView {
    let (said, class, glyph, words) = if danger {
        (
            tr(Msg::SyntheticFileWasDeleted),
            Variant::Destructive.class(),
            Glyph::TRASH,
            tr(Msg::Delete),
        )
    } else {
        (
            tr(Msg::SetReloaded),
            Variant::Primary.class(),
            Glyph::REFRESH_CW,
            tr(Msg::Reload),
        )
    };
    let id = format!("confirm-{dialog}");
    view! {
        <button
            type="button"
            id=id
            class=class
            on:click=move |_| {
                raise(Tone::Ok, said.clone());
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
        view! { <crate::islands::ModalDialogs /> },
    )
    .into_any()
}

/// The modal dialog a signal opens: a row's Delete names the row, and a
/// wide form.
pub(crate) fn modal_dialogs() -> AnyView {
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
                        {tr(Msg::Delete)}
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
                let row = pending
                    .get()
                    .map_or_else(|| tr(Msg::TheFile), str::to_owned);
                tr_with(Msg::DeleteRow, &[("row", row.into())])
            }),
            danger: true,
            width: Width::Narrow,
            body: view! {
                <p>{tr(Msg::FileGoesGoodRunningSet)}</p>
            }
            .into_any(),
            actions: view! {
                <button
                    type="button"
                    id="confirm-delete-row"
                    class=Variant::Destructive.class()
                    on:click=move |_| {
                        if let Some(row) = pending.get_untracked() {
                            raise(Tone::Ok, tr_with(Msg::RowDeleted, &[("row", row.into())]));
                        }
                        pending.set(None);
                    }
                >
                    {icon(Glyph::TRASH, "16")}
                    {tr(Msg::Delete)}
                </button>
            }
            .into_any(),
        },
        Signal::derive(move || pending.get().is_some()),
        Callback::new(move |()| pending.set(None)),
    )
    .into_any();
    let rename = rename_modal(renaming);
    view! {
        {divided_list(Some(tr(Msg::Files).into()), rows)}
        <button type="button" class=Variant::Secondary.class() id="open-rename" on:click=move |_| renaming.set(true)>
            {label(Some(Glyph::PENCIL), tr(Msg::RenameTheFile))}
        </button>
        {delete}
        {rename}
    }
    .into_any()
}

/// The wide modal form `renaming` opens.
fn rename_modal(renaming: RwSignal<bool>) -> AnyView {
    let name = RwSignal::new(String::from("synthetic-007.model.yml"));
    modal_dialog(
        Modal {
            id: "rename-file",
            title: Signal::derive(|| tr(Msg::RenameFile)),
            danger: false,
            width: Width::Wide,
            body: field(
                "rename-name",
                tr(Msg::NewName),
                Some(tr(Msg::PathBelowFhirconnectOmocl).clone()),
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
                    {tr(Msg::Rename)}
                </button>
            }
            .into_any(),
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
            <crate::islands::AccessPanel>{body}</crate::islands::AccessPanel>
        },
    )
    .into_any()
}

/// The side panel `access`, holding `body`, which the server drew.
pub(crate) fn access_panel(body: AnyView) -> AnyView {
    side_panel(SidePanel {
        id: "access",
        body_id: "access-drawer",
        title: tr(Msg::Access).into(),
        body,
    })
}

/// A button per tone that raises a toast in it, drawn by the island
/// [`crate::islands::ToastButtons`].
fn toasts() -> AnyView {
    card(tr(Msg::Toasts), view! { <crate::islands::ToastButtons /> }).into_any()
}

/// A button per tone that raises a toast in it.
pub(crate) fn toast_buttons() -> AnyView {
    let raise = move |tone: Tone, text: String| move |_| raise(tone, text.clone());
    let buttons = [
        (Tone::Ok, tr(Msg::Success), tr(Msg::SavedSetReloaded)),
        (Tone::Info, tr(Msg::Information), tr(Msg::SetReloadsBackground)),
        (Tone::Warn, tr(Msg::Warning), tr(Msg::SetCompiled2Warnings)),
        (Tone::Bad, tr(Msg::Refusal), tr(Msg::SaveWasRefused412)),
    ]
    .into_iter()
    .map(|(tone, words, text)| {
        view! {
            <button type="button" class=Variant::Secondary.class() data-toast=tone.name() on:click=raise(tone, text)>
                {label(Some(Glyph::BELL), words)}
            </button>
        }
    })
    .collect_view();
    cluster(buttons).into_any()
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
    let sections = (confirmations(), modals(), panel(), toasts());
    view! {
        {super::title(tr(Msg::OverlaysFeedback))}
        {header}
        {stack(view! {
            {grid(view! {
                {sections.0}
                {sections.2}
            })}
            {sections.1}
            {sections.3}
        })}
    }
}
