// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overlays and the feedback.
//!
//! They are the confirmation dialogs, the modal
//! dialog a signal opens, the side panel, a toast in every tone and the
//! popover form. The user menu, which sets the theme, is in the top bar,
//! and the drawer is the navigation on a narrow screen.

use crate::locale::{Msg, Words, tr, tr_with};
use ferrohealth_ui::i18n::{Arg, Word};
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::island::SidePanel;
use ferrohealth_ui::table::view::{ACTIONS, Body, Column, DataTable, Sorting, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::{ActionGroup, action_group};
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::dialog::{
    Confirmation, DIALOG_FORM, Explanation, confirm_dialog, danger_action_dialog, danger_dialog,
    info_dialog,
};
use ferrohealth_ui::view::form::{INPUT, TEXTAREA, described_by, field, keyed_field};
use ferrohealth_ui::view::kbd::{Shortcut, ShortcutGroup, kbd, shortcut_hint};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::popover::{PopoverForm, popover_form};
use ferrohealth_ui::view::preferences::{Post, language_select};
use ferrohealth_ui::view::surface::{
    card, chip, cluster, definitions, grid, page_header, panel, split, stack,
};
use ferrohealth_ui::widget::action_menu::{
    Act, Action, Choice, Item, Listbox, Menu, Pick, listbox, row_menu,
};
use ferrohealth_ui::widget::dialog::{Modal, ModalWords, Width, modal_dialog};
use ferrohealth_ui::widget::shortcuts::LegendWords;
use ferrohealth_ui::widget::side_panel::{Opener, SidePanelWords};
use ferrohealth_ui::widget::toast::{Toast, ToastAction, ToastLink, raise};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

/// The confirmation dialogs the title row opens with no script: the plain
/// one its primary opens and the danger one More's delete opens, whose
/// confirm, the island [`crate::islands::ConfirmAndToast`], raises a toast.
fn confirmations() -> AnyView {
    let plain = confirm_dialog(
        Confirmation {
            id: "reload-set".into(),
            title: tr(Msg::ReloadTheSet).into(),
            message: tr(Msg::FilesDiskCompileReplaceRunning).into(),
        },
        view! { <crate::islands::ConfirmAndToast dialog="reload-set".to_owned() danger=false words=confirm_words() /> },
    )
    .into_any();
    let danger = danger_dialog(
        Confirmation {
            id: "delete-file".into(),
            title: tr(Msg::DeleteTheFile).into(),
            message: tr(Msg::SyntheticFileGoesGoodRunning).into(),
        },
        view! { <crate::islands::ConfirmAndToast dialog="delete-file".to_owned() danger=true words=confirm_words() /> },
    )
    .into_any();
    let how = info_dialog(
        Explanation {
            id: "how-reloading".into(),
            title: tr(Msg::HowReloadingWorks).into(),
        },
        view! {
            <p>{tr(Msg::ReloadingCompiles)}</p>
            <p class="fui-small fui-muted">{tr(Msg::ReloadingKeeps)}</p>
        },
    )
    .into_any();
    card(
        tr(Msg::ConfirmationDialogs),
        view! {
            <p class="fui-muted">
                {tr(Msg::OpenedTheirTriggerNoScript)}
            </p>
            {plain}
            {danger}
            <button
                type="button"
                class=Variant::Ghost.class()
                command="show-modal"
                commandfor="how-reloading"
                aria-haspopup="dialog"
            >
                {icon(Glyph::INFO, "16")}
                {tr(Msg::HowReloadingWorks)}
            </button>
            {how}
            {member_removals()}
        },
    )
    .into_any()
}

/// A table whose every row's menu ends in Remove…, which opens a danger
/// confirmation of its own beside it in the row's actions cell, each
/// dialog's id made from its row. The
/// confirmation asks for a required Reason inside the `<ActionForm>` of
/// [`crate::acknowledge::remove_member`], which returns naming the member
/// removed, and the list leaves that row out.
fn member_removals() -> AnyView {
    let removed = use_query_map().with_untracked(|query| query.get("removed"));
    let back = page_path("overlays");
    let remove = ServerAction::<crate::acknowledge::RemoveMember>::new();
    let rows = crate::data::MEMBERS
        .into_iter()
        .filter(|member| removed.as_deref() != Some(*member))
        .map(|member| member_row(member, &back, remove))
        .collect_view();
    let said = removed
        .filter(|member| crate::data::MEMBERS.contains(&member.as_str()))
        .map(|member| {
            view! {
                <p class="fui-small fui-muted" id="member-removed">
                    {tr_with(Msg::MemberRemoved, &[("member", Arg::from(member))])}
                </p>
            }
        });
    let columns = [
        Column::text("member", tr(Msg::Member)),
        Column::text("actions", tr(Msg::Actions)),
    ];
    view! {
        <div id="member-removals">
            {data_table(DataTable {
                caption: &tr(Msg::RemoveMembers),
                columns: &columns,
                sorting: Sorting::default(),
                body: Body::Rows(rows.into_any()),
            })}
        </div>
        {said}
    }
    .into_any()
}

/// The row of `member`: its name, and in its actions cell its menu with
/// Remove… last and the confirmation that asks why and runs `remove`,
/// returning to `back`.
fn member_row(
    member: &'static str,
    back: &str,
    remove: ServerAction<crate::acknowledge::RemoveMember>,
) -> AnyView {
    let id = format!("remove-{member}");
    let reason = format!("{id}-reason");
    let named = [("member", Arg::from(member))];
    let menu = Menu::new(
        format!("member-menu-{member}"),
        tr_with(Msg::ActionsFor, &[("name", member.into())]),
        Vec::new(),
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        tr(Msg::Remove),
        Act::Dialog(id.clone()),
    ))]);
    let described = described_by(&reason, true, false);
    let fields = view! {
        <input type="hidden" name="member" value=member />
        <input type="hidden" name="back" value=back.to_owned() />
        {keyed_field(
            reason.clone(),
            tr(Msg::Reason),
            Some(tr(Msg::ReasonKept)),
            view! {
                <textarea id=reason name="reason" class=TEXTAREA required aria-describedby=described></textarea>
            },
        )}
    }
    .into_any();
    let dialog = danger_action_dialog(
        Confirmation {
            id: id.into(),
            title: tr_with(Msg::RemoveMemberTitle, &named).into(),
            message: tr_with(Msg::RemoveMemberBody, &named).into(),
        },
        fields,
        move |parts| {
            view! {
                <ActionForm action=remove attr:class=DIALOG_FORM>
                    {parts}
                </ActionForm>
            }
            .into_any()
        },
        Signal::derive(move || remove.value().with(|value| matches!(value, Some(Ok(()))))),
        view! {
            <button type="submit" class=Variant::Destructive.class()>
                {label(Some(Glyph::TRASH), tr_with(Msg::RemoveMember, &named))}
            </button>
        },
    );
    view! {
        <tr>
            <th scope="row"><code>{member}</code></th>
            <td data-label=tr(Msg::Actions) class=ACTIONS>
                {row_menu(menu)}
                {dialog}
            </td>
        </tr>
    }
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
            command="close"
            commandfor=dialog
            on:click=move |_| raise(Toast::new(Tone::Ok, said.clone()))
        >
            {icon(glyph, "16")}
            {words}
        </button>
    }
    .into_any()
}

/// The path of this brand's page `page`.
fn page_path(page: &str) -> String {
    crate::app::route_brand().map_or_else(
        || String::from("/"),
        |brand| format!("/{}/{page}", brand.key()),
    )
}

/// A file's menu `id`, named `label`: a link, a disabled item with its
/// reason, and the destructive delete last.
fn file_menu(id: &str, label: String) -> Menu {
    Menu::new(
        id,
        label,
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                tr(Msg::OpenTheDocument),
                Act::Link(page_path("code")),
            )),
            Item::new(Action::new(
                Some(Glyph::DOWNLOAD),
                tr(Msg::DownloadTheFile),
                Act::Link(page_path("code")),
            ))
            .disabled(tr(Msg::ProvisionedReadOnly)),
        ],
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        tr(Msg::DeleteFile),
        Act::Dialog(String::from("delete-file")),
    ))])
}

/// The title row's actions: Reload the set, which a confirmation follows,
/// and More with the file's other actions, its delete last.
fn overlays_actions() -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::REFRESH_CW),
        tr(Msg::ReloadSet),
        Act::Dialog(String::from("reload-set")),
    ))
    .more(file_menu("overlays-more", tr(Msg::MoreActionsFile)))
}

/// The action group's other shapes: a primary alone, a split primary with
/// its variants, and More alone.
fn action_groups() -> AnyView {
    let alone = ActionGroup::primary(Action::new(
        Some(Glyph::FILE_CODE),
        tr(Msg::OpenTheDocument),
        Act::Link(page_path("code")),
    ));
    let split = ActionGroup::primary(Action::new(
        Some(Glyph::SAVE),
        tr(Msg::Save),
        Act::Link(page_path("forms")),
    ))
    .variants(Menu::new(
        "save-variants",
        tr(Msg::WaysToSave),
        vec![
            Item::new(Action::new(
                None,
                tr(Msg::SaveAsDraft),
                Act::Link(page_path("forms")),
            )),
            Item::new(Action::new(
                None,
                tr(Msg::SaveAndReload),
                Act::Link(page_path("forms")),
            )),
        ],
    ));
    let more = ActionGroup::more_only(Menu::new(
        "alone-more",
        tr(Msg::MoreActionsSet),
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                tr(Msg::OpenTheDocument),
                Act::Link(page_path("code")),
            )),
            Item::new(Action::new(
                Some(Glyph::COPY),
                tr(Msg::CopyTheSetName),
                Act::Copy(String::from(crate::data::SET_NAME)),
            )),
        ],
    ));
    let beside = ActionGroup::primary(Action::new(
        Some(Glyph::PLAY),
        tr(Msg::RunQuery),
        Act::Link(page_path("forms")),
    ))
    .popover(PopoverForm {
        id: "save-query-beside".into(),
        variant: Variant::Secondary,
        glyph: Glyph::SAVE,
        label: tr(Msg::SaveTheQuery).into(),
        title: tr(Msg::SaveTheQueryToTheServer).into(),
        body: view! {
            <form method="get" action="#popover-form" class="fui-form">
                {field(
                    "beside-query-name",
                    tr(Msg::Name),
                    None,
                    view! { <input id="beside-query-name" name="name" type="text" class=INPUT value="synthetic-vitals" /> },
                )}
                <div class="fui-dialog-actions">
                    <button type="submit" class=Variant::Primary.class()>
                        {label(Some(Glyph::SAVE), tr(Msg::Save))}
                    </button>
                </div>
            </form>
        }
        .into_any(),
    });
    card(
        tr(Msg::ActionGroups),
        definitions(vec![
            (tr(Msg::APrimaryAlone), action_group(alone)),
            (tr(Msg::APrimaryPopover), action_group(beside)),
            (tr(Msg::ASplitPrimary), action_group(split)),
            (tr(Msg::MoreAlone), action_group(more)),
        ]),
    )
    .into_any()
}

/// A row menu per file, and the select buttons: the language, which posts,
/// and the version, whose choices are links.
fn menus() -> AnyView {
    let rows = ["synthetic-007.model.yml", "synthetic-011.extension.yml"]
        .into_iter()
        .enumerate()
        .map(|(at, row)| {
            let label = tr_with(Msg::ActionsFor, &[("name", row.into())]);
            split(
                view! { <code>{row}</code> },
                row_menu(file_menu(&format!("row-menu-{at}"), label)),
            )
            .into_any()
        })
        .collect();
    let here = page_path("overlays");
    let version = listbox(Listbox {
        id: String::from("version-list"),
        label: tr(Msg::Version),
        value: tr_with(
            Msg::VersionLatestOf,
            &[("version", "1.1.0".into()), ("count", 2.into())],
        )
        .into(),
        choices: vec![
            Choice {
                label: String::from("1.1.0").into(),
                note: Some(tr(Msg::Latest).into()),
                lang: None,
                selected: true,
                pick: Pick::Link(format!("{here}?version=1.1.0")),
            },
            Choice {
                label: String::from("1.0.0").into(),
                note: None,
                lang: None,
                selected: false,
                pick: Pick::Link(format!("{here}?version=1.0.0")),
            },
        ],
    });
    let language = language_select(
        "language-list",
        &crate::app::preferences_of(&crate::cookies::cookies_requested()).language,
        &crate::locale::choices(),
        &ferrohealth_ui::i18n::Locale::english(),
        &Post {
            action: String::from(crate::app::PREFERENCE_ROUTE),
            back: here.clone(),
        },
    );
    card(
        tr(Msg::MenusAndSelects),
        stack(view! {
            {divided_list(Some(tr(Msg::Files).into()), rows)}
            {definitions(vec![
                (tr(Msg::Language), language),
                (tr(Msg::Version), version),
            ])}
        }),
    )
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
                    Word::from(words.fill(Msg::DeleteRow, &[("row", &row)]))
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
            title: Signal::stored(Word::from(words.get(Msg::RenameFile))),
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

/// The session's access, which the side panel holds and its own page
/// shows.
fn access_body() -> AnyView {
    stack(view! {
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
    .into_any()
}

/// The route of the access page of the brand whose page is drawn.
fn access_route() -> String {
    let root =
        crate::app::route_brand().map_or_else(String::new, |brand| format!("/{}", brand.key()));
    format!("{root}/overlays/access")
}

/// The side panel a button opens, holding the session's access; with no
/// WebAssembly its opener leads to the access page.
fn side_panel_card() -> AnyView {
    let opener = Opener {
        label: tr(Msg::ViewAccess),
        glyph: Some(Glyph::SHIELD_CHECK),
        variant: Variant::Secondary,
        href: access_route(),
    };
    card(
        tr(Msg::SidePanel),
        view! {
            <p class="fui-muted">{tr(Msg::ContentNoNavigationFromEnd)}</p>
            <SidePanel
                id=String::from("access")
                title=tr(Msg::Access)
                opener=opener
                words=SidePanelWords::of_page()
            >
                {access_body()}
            </SidePanel>
        },
    )
    .into_any()
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
                id: "save-query".into(),
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

/// The page's shortcuts, as the legend lists them: the ones the gallery
/// binds, and a group of one.
fn shortcut_groups() -> Vec<ShortcutGroup> {
    let shortcut = |keys: &str, does: Msg| Shortcut {
        keys: String::from(keys),
        does: tr(does),
    };
    vec![
        ShortcutGroup {
            title: tr(Msg::ShortcutsEverywhere),
            shortcuts: vec![
                shortcut("/", Msg::ShortcutFind),
                shortcut("?", Msg::ShortcutShow),
                shortcut("Esc", Msg::ShortcutClose),
            ],
        },
        ShortcutGroup {
            title: tr(Msg::ShortcutsSearch),
            shortcuts: vec![shortcut("Enter", Msg::ShortcutOpenFirst)],
        },
    ]
}

/// The keyboard shortcut legend, opened by the island
/// [`crate::islands::ShortcutLegend`], and the hint a listing shows at its
/// foot.
fn shortcuts() -> AnyView {
    let hint = [
        Shortcut {
            keys: String::from("/"),
            does: tr(Msg::ShortcutFind),
        },
        Shortcut {
            keys: String::from("?"),
            does: tr(Msg::ShortcutShow),
        },
    ];
    card(
        tr(Msg::KeyboardShortcuts),
        view! {
            <p class="fui-muted">{tr(Msg::ShortcutsCardIntro)}</p>
            <crate::islands::ShortcutLegend
                groups=shortcut_groups()
                words=LegendWords::of_page()
                opener=tr(Msg::KeyboardShortcuts)
            />
            {shortcut_hint(&hint)}
        },
    )
    .into_any()
}

/// The legend of `groups` in the words `words`, and the button `opener`
/// that opens it: the library's legend opens on `?` outside a field and
/// from a control marked to open it, the button among them, never in the
/// phone layout. The button shows once the page has hydrated and from 40
/// rem up.
pub(crate) fn shortcut_legend_with(
    groups: Vec<ShortcutGroup>,
    words: LegendWords,
    opener: String,
) -> AnyView {
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));
    view! {
        <button
            type="button"
            id="open-shortcuts"
            class="fui-button fui-button-secondary fui-wide-only"
            aria-keyshortcuts="?"
            data-ferroui-shortcuts=""
            hidden=move || !hydrated.get()
        >
            {opener}
            {kbd("?")}
        </button>
        <ferrohealth_ui::island::ShortcutLegend groups words />
    }
    .into_any()
}

/// The access page: the side panel's content as a page of its own, where
/// the panel's opener leads with no WebAssembly.
#[component]
#[expect(
    clippy::must_use_candidate,
    reason = "the #[component] macro drops a #[must_use] from the function it writes"
)]
pub fn AccessPage() -> impl IntoView {
    let header = page_header(tr(Msg::Access), None, ActionGroup::none()).into_any();
    view! {
        {super::title(tr(Msg::Access))}
        {header}
        {panel(access_body())}
    }
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
        overlays_actions(),
    )
    .into_any();
    let sections = (
        confirmations(),
        modals(),
        side_panel_card(),
        toasts(),
        popover(),
        shortcuts(),
    );
    let shapes = (action_groups(), menus());
    view! {
        {super::title(tr(Msg::OverlaysFeedback))}
        {header}
        {stack(view! {
            {grid(view! {
                {sections.0}
                {sections.2}
            })}
            {grid(view! {
                {shapes.0}
                {shapes.1}
            })}
            {sections.1}
            {grid(view! {
                {sections.3}
                {sections.4}
            })}
            {sections.5}
        })}
    }
}
