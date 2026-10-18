// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overlays and the feedback.
//!
//! They are the confirmation dialogs, the modal
//! dialog a signal opens, the side panel, and a toast in every tone. The
//! operator menu and the theme menu are in the top bar, the drawer is the
//! navigation on a narrow screen.

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
use ferrohealth_ui::widget::toast::use_toasts;
use leptos::prelude::*;

/// The confirmation dialogs a trigger opens with no script: the plain one
/// and the danger one, whose confirm raises a toast.
fn confirmations() -> AnyView {
    let toasts = use_toasts();
    let confirm = move |id: &'static str, text: &'static str| {
        move |_| {
            if let Some(toasts) = toasts {
                toasts.push(Tone::Ok, text);
            }
            hide(id);
        }
    };
    let plain = confirm_dialog(
        Confirmation {
            id: "reload-set",
            title: "Reload the set?",
            message: "The files on disk compile and replace the running set when they compile.",
        },
        view! {
            <button type="button" class=Variant::Primary.class() on:click=confirm("reload-set", "The set reloaded.")>
                {icon(Glyph::REFRESH_CW, "16")}
                "Reload"
            </button>
        },
    )
    .into_any();
    let danger = danger_dialog(
        Confirmation {
            id: "delete-file",
            title: "Delete the file?",
            message: "The synthetic file goes for good; the running set reloads without it.",
        },
        view! {
            <button
                type="button"
                id="confirm-delete-file"
                class=Variant::Destructive.class()
                on:click=confirm("delete-file", "The synthetic file was deleted.")
            >
                {icon(Glyph::TRASH, "16")}
                "Delete"
            </button>
        },
    )
    .into_any();
    card(
        "Confirmation dialogs",
        view! {
            <p class="fui-muted">
                "Opened by their trigger with no script; Escape and Cancel close them and the focus returns."
            </p>
            {cluster(
                view! {
                    {confirm_trigger("reload-set", Variant::Secondary, Glyph::REFRESH_CW, "Reload the set")}
                    {confirm_trigger("delete-file", Variant::Danger, Glyph::TRASH, "Delete the file")}
                },
            )}
            {plain}
            {danger}
        },
    )
    .into_any()
}

/// The modal dialog a signal opens: a row's Delete names the row, and a
/// wide form.
fn modals() -> AnyView {
    let toasts = use_toasts();
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
                        "Delete"
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
                format!("Delete {}?", pending.get().unwrap_or("the file"))
            }),
            danger: true,
            width: Width::Narrow,
            body: view! {
                <p>"The file goes for good, and the running set reloads without it."</p>
            }
            .into_any(),
            actions: view! {
                <button
                    type="button"
                    id="confirm-delete-row"
                    class=Variant::Destructive.class()
                    on:click=move |_| {
                        if let (Some(toasts), Some(row)) = (toasts, pending.get_untracked()) {
                            toasts.push(Tone::Ok, format!("{row} was deleted."));
                        }
                        pending.set(None);
                    }
                >
                    {icon(Glyph::TRASH, "16")}
                    "Delete"
                </button>
            }
            .into_any(),
        },
        Signal::derive(move || pending.get().is_some()),
        Callback::new(move |()| pending.set(None)),
    )
    .into_any();
    let rename = rename_modal(renaming);
    card(
        "Modal dialogs",
        view! {
            {divided_list(Some("Files"), rows)}
            <button type="button" class=Variant::Secondary.class() id="open-rename" on:click=move |_| renaming.set(true)>
                {label(Some(Glyph::PENCIL), "Rename a file")}
            </button>
            {delete}
            {rename}
        },
    )
    .into_any()
}

/// The wide modal form `renaming` opens.
fn rename_modal(renaming: RwSignal<bool>) -> AnyView {
    let name = RwSignal::new(String::from("synthetic-007.model.yml"));
    modal_dialog(
        Modal {
            id: "rename-file",
            title: Signal::derive(|| String::from("Rename the file")),
            danger: false,
            width: Width::Wide,
            body: field(
                "rename-name",
                "New name",
                Some("A path below fhirconnect/ or omocl/."),
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
                    "Rename"
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
            ("Signed in as", "Synthetic operator".into_any()),
            ("Method", "OpenID Connect".into_any()),
            ("Policy", "The operator scope decides what the console may do.".into_any()),
        ])}
        <h3>"Scopes"</h3>
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
        "Side panel",
        view! {
            <p class="fui-muted">"Content that is no navigation, from the end of the page."</p>
            {side_panel_trigger("access", Variant::Secondary.class(), Glyph::SHIELD_CHECK, "View access")}
            {side_panel(SidePanel {
                id: "access",
                body_id: "access-drawer",
                title: "Access",
                body,
            })}
        },
    )
    .into_any()
}

/// A button per tone that raises a toast in it.
fn toasts() -> AnyView {
    let toasts = use_toasts();
    let raise = move |tone: Tone, text: &'static str| {
        move |_| {
            if let Some(toasts) = toasts {
                toasts.push(tone, text);
            }
        }
    };
    let buttons = [
        (Tone::Ok, "Success", "Saved, and the set reloaded."),
        (Tone::Info, "Information", "The set reloads in the background."),
        (Tone::Warn, "Warning", "The set compiled with 2 warnings."),
        (Tone::Bad, "Refusal", "The save was refused: 412."),
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
    card("Toasts", cluster(buttons)).into_any()
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
        "Overlays and feedback",
        Some(String::from("Dialogs, modals, the side panel and toasts.")),
        (),
    )
    .into_any();
    let sections = (confirmations(), modals(), panel(), toasts());
    view! {
        {super::title("Overlays and feedback")}
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
