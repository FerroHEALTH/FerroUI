// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The overlays and the feedback.
//!
//! They are the confirmation dialogs, the modal dialog a row's Delete
//! opens, the side panel, a toast in every tone and the popover form. The
//! user menu, which sets the theme, is in the top bar, and the drawer is
//! the navigation on a narrow screen.
//!
//! An outcome arrives as a flash toast on the page drawn again, as a
//! console answers an action: the address's `toast` names it, and a row's
//! `delete` draws the row's confirmation open.

use ferrohealth_ui::html::Html;
use ferrohealth_ui::i18n::Arg;
use ferrohealth_ui::icon::{Glyph, icon};
use ferrohealth_ui::table::view::{ACTIONS, Body, Column, DataTable, Sorting, data_table};
use ferrohealth_ui::tone::Tone;
use ferrohealth_ui::view::action::{ActionGroup, action_group};
use ferrohealth_ui::view::action_menu::{
    Act, Action, Choice, Item, Listbox, Menu, Pick, listbox, row_menu,
};
use ferrohealth_ui::view::button::{Variant, label};
use ferrohealth_ui::view::dialog::{
    ConfirmForm, Confirmation, Explanation, Modal, Width, confirm_dialog, danger_dialog,
    danger_form_dialog, info_dialog, modal_dialog,
};
use ferrohealth_ui::view::form::{INPUT, TEXTAREA, described_by, field, keyed_field};
use ferrohealth_ui::view::kbd::{Shortcut, ShortcutGroup, kbd, shortcut_hint};
use ferrohealth_ui::view::layout::divided_list;
use ferrohealth_ui::view::popover::{PopoverForm, popover_form};
use ferrohealth_ui::view::preferences::{Post, language_select};
use ferrohealth_ui::view::shortcuts::{OPENS_LEGEND, shortcut_legend};
use ferrohealth_ui::view::side_panel::{Opener, SidePanel, SidePanelWords, side_panel};
use ferrohealth_ui::view::surface::{
    card, chip, cluster, definitions, grid, page_header, panel, split, stack,
};
use ferrohealth_ui::view::toast::{Flash, RAISES, ToastLink};

use crate::app::{Page, REMOVE_ROUTE};
use crate::locale::Msg;
use crate::markup::{h, t};
use crate::visit::Visit;

/// The files a row of the modal dialogs' list names.
const FILES: [&str; 2] = ["synthetic-007.model.yml", "synthetic-011.extension.yml"];

/// The attributes with which the `toast` module raises `flash` in place of
/// following its link.
fn raises(flash: &Flash) -> String {
    let line = flash
        .line
        .as_ref()
        .map(|line| format!(" data-fui-toast-line=\"{}\"", t(line)))
        .unwrap_or_default();
    let action = flash
        .action
        .as_ref()
        .map(|action| {
            format!(
                " data-fui-toast-action=\"{}\" data-fui-toast-href=\"{}\"",
                t(&action.label),
                t(&action.href)
            )
        })
        .unwrap_or_default();
    format!(
        " {RAISES}=\"{}\" data-fui-toast-title=\"{}\"{line}{action}",
        flash.tone.name(),
        t(&flash.title)
    )
}

/// The path of this brand's page `page`.
fn page_path(v: &Visit, page: &str) -> String {
    format!("/{}/{page}", v.brand.key())
}

/// The confirm of the confirmation dialog `dialog`, the danger one when
/// `danger`: it leads to the page drawn again with the outcome's toast.
fn confirm_and_toast(v: &Visit, dialog: &str, danger: bool) -> Html {
    let (toast, class, glyph, words) = if danger {
        (
            "deleted",
            Variant::Destructive.class(),
            Glyph::TRASH,
            v.tr(Msg::Delete),
        )
    } else {
        (
            "reloaded",
            Variant::Primary.class(),
            Glyph::REFRESH_CW,
            v.tr(Msg::Reload),
        )
    };
    let raised = told(v, toast)
        .map(|flash| raises(&flash))
        .unwrap_or_default();
    h(format!(
        "<a href=\"{}?toast={toast}\" id=\"confirm-{dialog}\" class=\"{class}\"{raised} data-fui=\"keep\">{}{}</a>",
        t(page_path(v, "overlays")),
        icon(glyph, "16"),
        t(words)
    ))
}

/// The confirmation dialogs the title row opens with no script: the plain
/// one its primary opens and the danger one More's delete opens, each of
/// whose confirms leads to its outcome's toast.
fn confirmations(v: &Visit) -> Html {
    let plain = confirm_dialog(
        &v.locale,
        Confirmation {
            id: "reload-set".into(),
            title: v.tr(Msg::ReloadTheSet).into(),
            message: v.tr(Msg::FilesDiskCompileReplaceRunning).into(),
        },
        confirm_and_toast(v, "reload-set", false),
    );
    let danger = danger_dialog(
        &v.locale,
        Confirmation {
            id: "delete-file".into(),
            title: v.tr(Msg::DeleteTheFile).into(),
            message: v.tr(Msg::SyntheticFileGoesGoodRunning).into(),
        },
        confirm_and_toast(v, "delete-file", true),
    );
    let how = info_dialog(
        &v.locale,
        Explanation {
            id: "how-reloading".into(),
            title: v.tr(Msg::HowReloadingWorks).into(),
        },
        h(format!(
            "<p>{}</p><p class=\"fui-small fui-muted\">{}</p>",
            t(v.tr(Msg::ReloadingCompiles)),
            t(v.tr(Msg::ReloadingKeeps))
        )),
    );
    card(
        v.tr(Msg::ConfirmationDialogs),
        h(format!(
            "<p class=\"fui-muted\">{}</p>{plain}{danger}<button type=\"button\" class=\"{}\" command=\"show-modal\" commandfor=\"how-reloading\" aria-haspopup=\"dialog\">{}{}</button>{how}{}",
            t(v.tr(Msg::OpenedTheirTriggerNoScript)),
            Variant::Ghost.class(),
            icon(Glyph::INFO, "16"),
            t(v.tr(Msg::HowReloadingWorks)),
            member_removals(v)
        )),
    )
}

/// A table whose every row's menu ends in Remove…, which opens a danger
/// confirmation of its own beside it in the row's actions cell, each
/// dialog's id made from its row. The confirmation asks for a required
/// Reason in the form it posts to the gallery's removal route, which
/// returns naming the member removed, and the list leaves that row out.
fn member_removals(v: &Visit) -> Html {
    let removed = v.param("removed");
    let back = page_path(v, "overlays");
    let rows = crate::data::MEMBERS
        .into_iter()
        .filter(|member| removed != Some(*member))
        .map(|member| member_row(v, member, &back))
        .collect();
    let said = removed
        .filter(|member| crate::data::MEMBERS.contains(member))
        .map(|member| {
            h(format!(
                "<p class=\"fui-small fui-muted\" id=\"member-removed\">{}</p>",
                t(v.tr_with(Msg::MemberRemoved, &[("member", Arg::from(member))]))
            ))
        })
        .unwrap_or_default();
    let columns = [
        Column::text("member", v.tr(Msg::Member)),
        Column::text("actions", v.tr(Msg::Actions)),
    ];
    let table = data_table(DataTable {
        caption: &v.tr(Msg::RemoveMembers),
        columns: &columns,
        sorting: Sorting::default(),
        body: Body::Rows(rows),
    });
    h(format!("<div id=\"member-removals\">{table}</div>{said}"))
}

/// The row of `member`: its name, and in its actions cell its menu with
/// Remove… last and the confirmation that asks why and posts the removal,
/// returning to `back`.
fn member_row(v: &Visit, member: &'static str, back: &str) -> Html {
    let id = format!("remove-{member}");
    let reason = format!("{id}-reason");
    let named = [("member", Arg::from(member))];
    let menu = Menu::new(
        format!("member-menu-{member}"),
        v.tr_with(Msg::ActionsFor, &[("name", member.into())]),
        Vec::new(),
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        v.tr(Msg::Remove),
        Act::Dialog(id.clone()),
    ))]);
    let described = described_by(&reason, true, false).unwrap_or_default();
    let fields = Html::join([
        h(format!(
            "<input type=\"hidden\" name=\"member\" value=\"{member}\"><input type=\"hidden\" name=\"back\" value=\"{}\">",
            t(back)
        )),
        keyed_field(
            reason.clone(),
            v.tr(Msg::Reason),
            Some(v.tr(Msg::ReasonKept)),
            h(format!(
                "<textarea id=\"{reason}\" name=\"reason\" class=\"{TEXTAREA}\" required aria-describedby=\"{described}\"></textarea>"
            )),
        ),
    ]);
    let dialog = danger_form_dialog(
        &v.locale,
        Confirmation {
            id: id.into(),
            title: v.tr_with(Msg::RemoveMemberTitle, &named).into(),
            message: v.tr_with(Msg::RemoveMemberBody, &named).into(),
        },
        ConfirmForm {
            action: String::from(REMOVE_ROUTE),
            fields,
        },
        h(format!(
            "<button type=\"submit\" class=\"{}\">{}</button>",
            Variant::Destructive.class(),
            label(Some(Glyph::TRASH), v.tr_with(Msg::RemoveMember, &named))
        )),
    );
    h(format!(
        "<tr><th scope=\"row\"><code>{member}</code></th><td data-label=\"{}\" class=\"{ACTIONS}\">{}{dialog}</td></tr>",
        t(v.tr(Msg::Actions)),
        row_menu(&v.locale, &menu)
    ))
}

/// A file's menu `id`, named `label`: a link, a disabled item with its
/// reason, and the destructive delete last.
fn file_menu(v: &Visit, id: &str, label: String) -> Menu {
    Menu::new(
        id,
        label,
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                v.tr(Msg::OpenTheDocument),
                Act::Link(page_path(v, "code")),
            )),
            Item::new(Action::new(
                Some(Glyph::DOWNLOAD),
                v.tr(Msg::DownloadTheFile),
                Act::Link(page_path(v, "code")),
            ))
            .disabled(v.tr(Msg::ProvisionedReadOnly)),
        ],
    )
    .destructive(vec![Item::new(Action::new(
        Some(Glyph::TRASH),
        v.tr(Msg::DeleteFile),
        Act::Dialog(String::from("delete-file")),
    ))])
}

/// The title row's actions: Reload the set, which a confirmation follows,
/// and More with the file's other actions, its delete last.
fn overlays_actions(v: &Visit) -> ActionGroup {
    ActionGroup::primary(Action::new(
        Some(Glyph::REFRESH_CW),
        v.tr(Msg::ReloadSet),
        Act::Dialog(String::from("reload-set")),
    ))
    .more(file_menu(v, "overlays-more", v.tr(Msg::MoreActionsFile)))
}

/// The action group's other shapes: a primary alone, a split primary with
/// its variants, and More alone.
fn action_groups(v: &Visit) -> Html {
    let alone = ActionGroup::primary(Action::new(
        Some(Glyph::FILE_CODE),
        v.tr(Msg::OpenTheDocument),
        Act::Link(page_path(v, "code")),
    ));
    let split = ActionGroup::primary(Action::new(
        Some(Glyph::SAVE),
        v.tr(Msg::Save),
        Act::Link(page_path(v, "forms")),
    ))
    .variants(Menu::new(
        "save-variants",
        v.tr(Msg::WaysToSave),
        vec![
            Item::new(Action::new(
                None,
                v.tr(Msg::SaveAsDraft),
                Act::Link(page_path(v, "forms")),
            )),
            Item::new(Action::new(
                None,
                v.tr(Msg::SaveAndReload),
                Act::Link(page_path(v, "forms")),
            )),
        ],
    ));
    let more = ActionGroup::more_only(Menu::new(
        "alone-more",
        v.tr(Msg::MoreActionsSet),
        vec![
            Item::new(Action::new(
                Some(Glyph::FILE_CODE),
                v.tr(Msg::OpenTheDocument),
                Act::Link(page_path(v, "code")),
            )),
            Item::new(Action::new(
                Some(Glyph::COPY),
                v.tr(Msg::CopyTheSetName),
                Act::Copy(String::from(crate::data::SET_NAME)),
            )),
        ],
    ));
    let beside = ActionGroup::primary(Action::new(
        Some(Glyph::PLAY),
        v.tr(Msg::RunQuery),
        Act::Link(page_path(v, "forms")),
    ))
    .popover(PopoverForm {
        id: "save-query-beside".into(),
        variant: Variant::Secondary,
        glyph: Glyph::SAVE,
        label: v.tr(Msg::SaveTheQuery).into(),
        title: v.tr(Msg::SaveTheQueryToTheServer).into(),
        body: h(format!(
            "<form method=\"get\" action=\"#popover-form\" class=\"fui-form\">{}<div class=\"fui-dialog-actions\"><button type=\"submit\" class=\"{}\">{}</button></div></form>",
            field(
                "beside-query-name",
                v.tr(Msg::Name),
                None,
                h(format!(
                    "<input id=\"beside-query-name\" name=\"name\" type=\"text\" class=\"{INPUT}\" value=\"synthetic-vitals\">"
                )),
            ),
            Variant::Primary.class(),
            label(Some(Glyph::SAVE), v.tr(Msg::Save))
        )),
    });
    card(
        v.tr(Msg::ActionGroups),
        definitions(vec![
            (v.tr(Msg::APrimaryAlone), action_group(&v.locale, alone)),
            (v.tr(Msg::APrimaryPopover), action_group(&v.locale, beside)),
            (v.tr(Msg::ASplitPrimary), action_group(&v.locale, split)),
            (v.tr(Msg::MoreAlone), action_group(&v.locale, more)),
        ]),
    )
}

/// A row menu per file, and the select buttons: the language, which posts,
/// and the version, whose choices are links.
fn menus(v: &Visit) -> Html {
    let rows = FILES
        .into_iter()
        .enumerate()
        .map(|(at, row)| {
            let label = v.tr_with(Msg::ActionsFor, &[("name", row.into())]);
            split(
                h(format!("<code>{row}</code>")),
                row_menu(&v.locale, &file_menu(v, &format!("row-menu-{at}"), label)),
            )
        })
        .collect();
    let here = page_path(v, "overlays");
    let version = listbox(
        &v.locale,
        &Listbox {
            id: String::from("version-list"),
            label: v.tr(Msg::Version),
            value: v
                .tr_with(
                    Msg::VersionLatestOf,
                    &[("version", "1.1.0".into()), ("count", 2.into())],
                )
                .into(),
            choices: vec![
                Choice {
                    label: String::from("1.1.0").into(),
                    note: Some(v.tr(Msg::Latest).into()),
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
        },
    );
    let language = language_select(
        &v.locale,
        "language-list",
        &v.preferences.language,
        &crate::locale::choices(),
        &ferrohealth_ui::i18n::Locale::english(),
        &Post {
            action: String::from(crate::app::PREFERENCE_ROUTE),
            back: here.clone(),
        },
    );
    card(
        v.tr(Msg::MenusAndSelects),
        stack(Html::join([
            divided_list(Some(v.tr(Msg::Files).into()), rows),
            definitions(vec![
                (v.tr(Msg::Language), language),
                (v.tr(Msg::Version), version),
            ]),
        ])),
    )
}

/// The modal dialogs: a row's Delete leads to the page with the row's
/// confirmation drawn open, naming the row, and Rename opens a wide form in
/// place.
fn modals(v: &Visit) -> Html {
    let here = page_path(v, "overlays");
    let rows = FILES
        .into_iter()
        .map(|row| {
            split(
                h(format!("<code>{row}</code>")),
                h(format!(
                    "<a href=\"{}?delete={row}\" class=\"{}\" data-delete=\"{row}\" data-fui=\"keep\">{}{}</a>",
                    t(&here),
                    Variant::Danger.small(),
                    icon(Glyph::TRASH, "14"),
                    t(v.tr(Msg::Delete))
                )),
            )
        })
        .collect();
    let pending = v.param("delete").filter(|row| FILES.contains(row));
    let named = pending.map_or_else(|| v.tr(Msg::TheFile), str::to_owned);
    let delete = modal_dialog(
        &v.locale,
        Modal {
            id: "delete-row".into(),
            title: v
                .tr_with(Msg::DeleteRow, &[("row", Arg::from(named.clone()))])
                .into(),
            danger: true,
            width: Width::Narrow,
            body: h(format!("<p>{}</p>", t(v.tr(Msg::FileGoesGoodRunningSet)))),
            actions: h(format!(
                "<a href=\"{}?toast=row-deleted&amp;row={}\" id=\"confirm-delete-row\" class=\"{}\"{} data-fui=\"keep\">{}{}</a>",
                t(&here),
                t(&named),
                Variant::Destructive.class(),
                pending
                    .map(|row| raises(&Flash::new(
                        Tone::Ok,
                        v.tr_with(Msg::RowDeleted, &[("row", Arg::from(row))])
                    )))
                    .unwrap_or_default(),
                icon(Glyph::TRASH, "16"),
                t(v.tr(Msg::Delete))
            )),
            open: pending.is_some(),
            cancel: Some(format!("{here}#modals")),
        },
    );
    let rename = modal_dialog(
        &v.locale,
        Modal {
            id: "rename-file".into(),
            title: v.tr(Msg::RenameFile).into(),
            danger: false,
            width: Width::Wide,
            body: field(
                "rename-name",
                v.tr(Msg::NewName),
                Some(v.tr(Msg::PathBelowFhirconnectOmocl)),
                h(format!(
                    "<input id=\"rename-name\" type=\"text\" class=\"{INPUT}\" value=\"synthetic-007.model.yml\">"
                )),
            ),
            actions: h(format!(
                "<button type=\"button\" class=\"{}\" command=\"close\" commandfor=\"rename-file\">{}{}</button>",
                Variant::Primary.class(),
                icon(Glyph::SAVE, "16"),
                t(v.tr(Msg::Rename))
            )),
            open: false,
            cancel: None,
        },
    );
    card(
        v.tr(Msg::ModalDialogs),
        h(format!(
            "<div id=\"modals\">{}<button type=\"button\" class=\"{}\" id=\"open-rename\" command=\"show-modal\" commandfor=\"rename-file\" aria-haspopup=\"dialog\">{}</button>{delete}{rename}</div>",
            divided_list(Some(v.tr(Msg::Files).into()), rows),
            Variant::Secondary.class(),
            label(Some(Glyph::PENCIL), v.tr(Msg::RenameTheFile))
        )),
    )
}

/// The session's access, which the side panel holds and its own page
/// shows.
fn access_body(v: &Visit) -> Html {
    stack(Html::join([
        definitions(vec![
            (v.tr(Msg::Signed), t(v.tr(Msg::SyntheticOperator))),
            (v.tr(Msg::Method), t(v.tr(Msg::OpenidConnect))),
            (
                v.tr(Msg::Policy),
                t(v.tr(Msg::OperatorScopeDecidesWhatConsole)),
            ),
        ]),
        h(format!("<h3>{}</h3>", t(v.tr(Msg::Scopes)))),
        cluster(Html::join([
            chip("ferrobridge.operator"),
            chip("openid"),
            chip("profile"),
        ])),
    ]))
}

/// The side panel a button opens, holding the session's access; with no
/// script its opener leads to the access page.
fn side_panel_card(v: &Visit) -> Html {
    let opener = Opener {
        label: v.tr(Msg::ViewAccess),
        glyph: Some(Glyph::SHIELD_CHECK),
        variant: Variant::Secondary,
        href: page_path(v, "overlays/access"),
    };
    card(
        v.tr(Msg::SidePanel),
        h(format!(
            "<p class=\"fui-muted\">{}</p>{}",
            t(v.tr(Msg::ContentNoNavigationFromEnd)),
            side_panel(SidePanel {
                id: String::from("access"),
                title: v.tr(Msg::Access),
                opener,
                body: access_body(v),
                words: SidePanelWords::of(&v.locale),
            })
        )),
    )
}

/// The toast the address's `toast` names, as the action that led here
/// answered it.
fn flash(v: &Visit) -> Option<Flash> {
    told(v, v.param("toast")?)
}

/// The toast the key `key` names.
fn told(v: &Visit, key: &str) -> Option<Flash> {
    let tables = super::overview::brand_page(v, "tables");
    let here = page_path(v, "overlays");
    Some(match key {
        "ok" => Flash::new(Tone::Ok, v.tr(Msg::SavedSetReloaded))
            .line(v.tr(Msg::ThreeFilesCompiledFacadeServes))
            .action(ToastLink {
                label: v.tr(Msg::OpenTheTables),
                href: tables,
            }),
        "info" => Flash::new(Tone::Info, v.tr(Msg::SetReloadsBackground)),
        "warn" => Flash::new(Tone::Warn, v.tr(Msg::SetCompiled2Warnings))
            .line(v.tr(Msg::TwoMappingsNameMissingPath)),
        "bad" => Flash::new(Tone::Bad, v.tr(Msg::SaveWasRefused412))
            .line(v.tr(Msg::SomeoneSavedTheFileFirst))
            .action(ToastLink {
                label: v.tr(Msg::TryAgain),
                href: format!("{here}?toast=ok"),
            }),
        "reloaded" => Flash::new(Tone::Ok, v.tr(Msg::SetReloaded)),
        "deleted" => Flash::new(Tone::Ok, v.tr(Msg::SyntheticFileWasDeleted)),
        "row-deleted" => {
            let row = v.param("row").filter(|row| FILES.contains(row))?;
            Flash::new(
                Tone::Ok,
                v.tr_with(Msg::RowDeleted, &[("row", Arg::from(row))]),
            )
        }
        _ => return None,
    })
}

/// A link per tone that leads to the page drawn again with a toast in it:
/// a title, a line under it where there is more to say, and an action, the
/// success toast's leading to the tables and the refusal's trying again.
fn toasts(v: &Visit) -> Html {
    let here = page_path(v, "overlays");
    let buttons = [
        (Tone::Ok, Msg::Success),
        (Tone::Info, Msg::Information),
        (Tone::Warn, Msg::Warning),
        (Tone::Bad, Msg::Refusal),
    ]
    .into_iter()
    .map(|(tone, words)| {
        let raised = told(v, tone.name())
            .map(|flash| raises(&flash))
            .unwrap_or_default();
        h(format!(
            "<a href=\"{}?toast={}\" class=\"{}\" data-toast=\"{}\"{raised} data-fui=\"keep\">{}</a>",
            t(&here),
            tone.name(),
            Variant::Secondary.class(),
            tone.name(),
            label(Some(Glyph::BELL), v.tr(words))
        ))
    })
    .collect();
    card(v.tr(Msg::Toasts), cluster(buttons))
}

/// The popover form: Save the query, a small form that opens under its
/// trigger and posts with no script.
fn popover(v: &Visit) -> Html {
    let body = h(format!(
        "<form method=\"get\" action=\"#popover-form\" class=\"fui-form\">{}{}<div class=\"fui-dialog-actions\"><button type=\"submit\" class=\"{}\">{}</button></div></form>",
        field(
            "query-name",
            v.tr(Msg::Name),
            None,
            h(format!(
                "<input id=\"query-name\" name=\"name\" type=\"text\" class=\"{INPUT}\" value=\"synthetic-vitals\">"
            )),
        ),
        field(
            "query-version",
            v.tr(Msg::Version),
            None,
            h(format!(
                "<input id=\"query-version\" name=\"version\" type=\"text\" class=\"{INPUT}\" value=\"1.0.0\">"
            )),
        ),
        Variant::Primary.class(),
        label(Some(Glyph::SAVE), v.tr(Msg::Save))
    ));
    card(
        v.tr(Msg::PopoverForm),
        h(format!(
            "<p class=\"fui-muted\" id=\"popover-form\">{}</p>{}",
            t(v.tr(Msg::PopoverFormOpensUnderTrigger)),
            cluster(popover_form(PopoverForm {
                id: "save-query".into(),
                variant: Variant::Secondary,
                glyph: Glyph::SAVE,
                label: v.tr(Msg::SaveTheQuery).into(),
                title: v.tr(Msg::SaveTheQueryToTheServer).into(),
                body,
            }))
        )),
    )
}

/// The page's shortcuts, as the legend lists them: the ones the gallery
/// binds, and a group of one.
fn shortcut_groups(v: &Visit) -> Vec<ShortcutGroup> {
    let shortcut = |keys: &str, does: Msg| Shortcut {
        keys: String::from(keys),
        does: v.tr(does),
    };
    vec![
        ShortcutGroup {
            title: v.tr(Msg::ShortcutsEverywhere),
            shortcuts: vec![
                shortcut("/", Msg::ShortcutFind),
                shortcut("?", Msg::ShortcutShow),
                shortcut("Esc", Msg::ShortcutClose),
            ],
        },
        ShortcutGroup {
            title: v.tr(Msg::ShortcutsSearch),
            shortcuts: vec![shortcut("Enter", Msg::ShortcutOpenFirst)],
        },
    ]
}

/// The keyboard shortcut legend, opened on `?` and by its button, which the
/// `shortcuts` module shows as it runs, from 40 rem up, and the hint
/// a listing shows at its foot.
fn shortcuts(v: &Visit) -> Html {
    let hint = [
        Shortcut {
            keys: String::from("/"),
            does: v.tr(Msg::ShortcutFind),
        },
        Shortcut {
            keys: String::from("?"),
            does: v.tr(Msg::ShortcutShow),
        },
    ];
    card(
        v.tr(Msg::KeyboardShortcuts),
        h(format!(
            "<p class=\"fui-muted\">{}</p><button type=\"button\" id=\"open-shortcuts\" class=\"fui-button fui-button-secondary fui-wide-only\" aria-keyshortcuts=\"?\" {OPENS_LEGEND} hidden>{}{}</button>{}{}",
            t(v.tr(Msg::ShortcutsCardIntro)),
            t(v.tr(Msg::KeyboardShortcuts)),
            kbd("?"),
            shortcut_legend(&v.locale, shortcut_groups(v)),
            shortcut_hint(&v.locale, &hint)
        )),
    )
}

/// The access page: the side panel's content as a page of its own, where
/// the panel's opener leads with no script.
#[must_use]
pub fn access(v: &Visit) -> Page {
    let header = page_header(&v.locale, v.tr(Msg::Access), None, ActionGroup::none());
    Page::shell(
        v.tr(Msg::Access),
        Html::join([header, panel(access_body(v))]),
    )
}

/// The overlays page.
#[must_use]
pub fn page(v: &Visit) -> Page {
    let header = page_header(
        &v.locale,
        v.tr(Msg::OverlaysFeedback),
        Some(v.tr(Msg::DialogsModalsSidePanelToasts)),
        overlays_actions(v),
    );
    let body = stack(Html::join([
        grid(Html::join([confirmations(v), side_panel_card(v)])),
        grid(Html::join([action_groups(v), menus(v)])),
        modals(v),
        grid(Html::join([toasts(v), popover(v)])),
        shortcuts(v),
    ]));
    Page::shell(v.tr(Msg::OverlaysFeedback), Html::join([header, body]))
        .with_flash(flash(v).into_iter().collect())
}
