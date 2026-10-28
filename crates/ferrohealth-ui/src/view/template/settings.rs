// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! T3, the settings and reference page: the title, binder tabs (one
//! section each, each a route) and one full-width panel of setting rows.
//!
//! A console's configuration comes from one of two sources (family rule
//! 13). A product with no configuration store reads it from its file
//! alone ([`Source::File`]): every tab is read-only, the panel says where
//! the values are set in one quiet line, and there is no mode badge, no
//! edit and no per-row Next start marker. A product with its own store
//! manages it ([`Source::Managed`]): the mode badge sits beside the title,
//! the revision line under the header, and each tab's panel opens with its
//! intro and its own Edit. A draft is never a strip: the tab carries the
//! draft badge and the panel's intro says it. The product names its own
//! configuration key; the library names none.

use std::borrow::Cow;

use leptos::prelude::*;

use crate::i18n::{Text, Word, tr};
use crate::icon::{Glyph, icon};
use crate::view::action::ActionGroup;
use crate::view::replica::{Replica, replica_line};
use crate::view::surface::{Header, page_heading};
use crate::view::tabs::{TabRow, joined};
use crate::widget::copy::KeyHintWords;

/// One setting: its label, what it governs, its value and its keys.
#[derive(Debug)]
pub struct Setting {
    /// The setting's name, at most 80 characters.
    pub label: String,
    /// One line on what the setting governs.
    pub description: String,
    /// The value as it runs, or in edit the control that changes it.
    pub value: AnyView,
    /// The configuration keys that set it, each a key hint; a setting that
    /// shows two or more values names each key.
    pub keys: Vec<String>,
    /// Whether a change takes effect at the next start, in a managed
    /// configuration.
    pub next_start: bool,
    /// Where the value came from when it is not where the page says the
    /// settings come from, drawn beside the value ([`ValueSource`]).
    pub source: Option<ValueSource>,
}

/// Where a setting's value came from, when it is not where the page says
/// the settings come from.
///
/// The unmarked case (`Setting::source` is `None`) is the page's own
/// source: the file of a file-only configuration ([`Source::File`]), the
/// running revision of a managed one. A marker tells an operator that
/// changing that source leaves the value alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ValueSource {
    /// An environment variable, which wins over the file and a revision:
    /// "From the environment".
    Environment,
    /// Nothing sets it: the product's own default, "Default".
    Default,
    /// The configuration file, in a managed configuration whose section
    /// the file sets: "From the file".
    File,
    /// A `--set key=value` argument on the command line, which wins over
    /// the environment, the file and a revision: "From the command line".
    CommandLine,
}

impl ValueSource {
    /// The glyph drawn beside the source's words.
    #[must_use]
    pub const fn glyph(self) -> Glyph {
        match self {
            Self::Environment => Glyph::SQUARE_TERMINAL,
            Self::Default => Glyph::ROTATE_CCW,
            Self::File => Glyph::FILE_TEXT,
            Self::CommandLine => Glyph::KEYBOARD,
        }
    }

    /// The library's words for the source, in the page's language.
    #[must_use]
    pub fn words(self) -> Word {
        tr(match self {
            Self::Environment => Text::SettingSourceEnvironment,
            Self::Default => Text::SettingSourceDefault,
            Self::File => Text::SettingSourceFile,
            Self::CommandLine => Text::SettingSourceCommandLine,
        })
    }
}

/// Where a page's settings come from (family rule 13).
#[derive(Debug)]
pub enum Source {
    /// The product's file alone, read at start.
    File {
        /// The file, such as `ferrobridge.toml`.
        file: String,
        /// Where "How configuration works" leads.
        how: How,
    },
    /// The product's own configuration store.
    Managed(Box<Managed>),
}

/// Where the words "How configuration works" of a file line lead.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum How {
    /// A link to the page that says it, such as the book's.
    Page(String),
    /// A button that opens the informational dialog with this id
    /// ([`crate::view::dialog::info_dialog`]) by the `show-modal` command,
    /// drawn as the link would be, so it opens with no script.
    Dialog(Cow<'static, str>),
}

/// What a managed configuration's page adds.
#[derive(Debug)]
pub struct Managed {
    /// The mode badge beside the title.
    pub badge: AnyView,
    /// The revision line, or the strip of an event until it is resolved,
    /// where the settings are revisioned.
    pub revision: Option<AnyView>,
    /// The tab's one-sentence intro on what it governs and how a change
    /// takes effect, or in edit what the draft holds.
    pub intro: String,
    /// The tab's actions: Edit, or in edit what the draft can do.
    pub actions: Option<AnyView>,
}

/// What a settings page shows.
#[derive(Debug)]
pub struct SettingsPage {
    /// The title and its subtitle; its badge is the source's to set.
    pub header: Header,
    /// The title row's action group (F14), such as How configuration works
    /// and More.
    pub actions: ActionGroup,
    /// Where the settings come from.
    pub source: Source,
    /// The binder tabs, when the page has three or more sections.
    pub tabs: Option<TabRow>,
    /// The tab's settings.
    pub settings: Vec<Setting>,
    /// The replica that answered, for a replicated product's reference
    /// page (F6), drawn as one quiet line under the header.
    pub replica: Option<Replica>,
}

/// Returns a settings page (T3): its header with the mode badge of a
/// managed configuration, the revision line, and one panel of settings
/// under binder tabs where it has them.
#[must_use]
pub fn settings_page(page: SettingsPage) -> AnyView {
    let SettingsPage {
        header,
        actions,
        source,
        tabs,
        settings,
        replica,
    } = page;
    let managed = matches!(source, Source::Managed(_));
    let (badge, revision, intro, panel_actions) = match source {
        Source::File { file, how } => (None, None, file_line(file, how), None),
        Source::Managed(managed) => {
            let Managed {
                badge,
                revision,
                intro,
                actions,
            } = *managed;
            (Some(badge), revision, intro.into_any(), actions)
        }
    };
    let heading = page_heading(Header { badge, ..header }, actions).into_any();
    let replica = replica.map(replica_line);
    let rows = settings
        .into_iter()
        .map(|setting| setting_row(setting, managed))
        .collect::<Vec<_>>();
    let panel = settings_panel(intro, panel_actions, rows);
    let section = match tabs {
        Some(tabs) => joined(tabs, panel, Some("fui-settings-tabbed")),
        None => view! { <div class="fui-card fui-panel">{panel}</div> }.into_any(),
    };
    view! {
        {heading}
        {replica}
        <div class="fui-stack">{revision} {section}</div>
    }
    .into_any()
}

/// Returns the one quiet line of a file-only configuration: "Set in `file`
/// · read at start · How configuration works", the last leading where
/// `how` says: a link to a page, or a button that opens a dialog.
#[must_use]
pub fn file_line(file: String, how: How) -> AnyView {
    let set_in = tr(Text::ConfigSetIn).map(|words| format!("{words} "));
    let read = tr(Text::ConfigReadAtStart).map(|words| format!(" · {words} · "));
    let how = match how {
        How::Page(href) => view! { <a href=href>{tr(Text::ConfigHow)}</a> }.into_any(),
        How::Dialog(id) => view! {
            <button
                type="button"
                class="fui-link-button"
                command="show-modal"
                commandfor=id
                aria-haspopup="dialog"
            >
                {tr(Text::ConfigHow)}
            </button>
        }
        .into_any(),
    };
    view! {
        {set_in}
        <code>{file}</code>
        {read}
        {how}
    }
    .into_any()
}

/// Returns a tab's panel: its `intro` and `actions` in the panel's head,
/// then the setting `rows` across the panel's width.
#[must_use]
pub fn settings_panel(intro: AnyView, actions: Option<AnyView>, rows: Vec<AnyView>) -> AnyView {
    view! {
        <div class="fui-settings-panel">
            <div class="fui-settings-panel-head">
                <p class="fui-settings-panel-intro">{intro}</p>
                <div class="fui-actions">{actions}</div>
            </div>
            <div class="fui-setting-rows">{rows}</div>
        </div>
    }
    .into_any()
}

/// Returns one setting row (R27): the label column, then the value.
///
/// The label carries its key hints and the line under it; the value sits
/// from 40 % of the panel, with the Next start marker after it where a
/// managed setting takes effect at the next start, and the value's source
/// after that where it is not the page's own ([`ValueSource`]).
///
/// A file-only configuration (`managed` false) draws no Next start marker:
/// every value there is read at start.
#[must_use]
pub fn setting_row(setting: Setting, managed: bool) -> AnyView {
    let Setting {
        label,
        description,
        value,
        keys,
        next_start,
        source,
    } = setting;
    let hints = keys
        .into_iter()
        .map(|key| {
            let words = KeyHintWords::of_page(&key);
            view! { <crate::island::KeyHint key words /> }
        })
        .collect_view();
    let next = (managed && next_start).then(|| {
        view! {
            <span class="fui-effect fui-effect-restart">
                {icon(Glyph::REFRESH_CW, "12")} {tr(Text::NextStart)}
            </span>
        }
    });
    let source = source.map(|source| {
        view! {
            <span class="fui-setting-source">
                {icon(source.glyph(), "12")} {source.words()}
            </span>
        }
    });
    view! {
        <div class="fui-setting-row">
            <div class="fui-setting-text">
                <span class="fui-setting-label">{label} {hints}</span>
                <p class="fui-setting-desc">{description}</p>
            </div>
            <div class="fui-setting-control">
                <span class="fui-setting-value">{value}</span>
                {next}
                {source}
            </div>
        </div>
    }
    .into_any()
}

/// Returns the revision line of revisioned settings: one line, such as
/// "Revision 7 · applied 2026-10-14 by Demo Steward · in step", under the
/// settings page's header.
#[must_use]
pub fn revision_line(line: AnyView) -> AnyView {
    view! {
        <div class="fui-revision fui-revision-compact">
            {icon(Glyph::FILE_TEXT, "16")}
            <p>{line}</p>
        </div>
    }
    .into_any()
}
