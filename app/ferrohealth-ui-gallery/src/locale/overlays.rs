// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the overlays page.

use super::Msg;

messages! {
    /// `A primary alone`
    APrimaryAlone = "a-primary-alone",
    /// `A primary beside a popover form`
    APrimaryPopover = "a-primary-popover",
    /// `A split primary and its variants`
    ASplitPrimary = "a-split-primary",
    /// `Access`
    Access = "access",
    /// `Action groups`
    ActionGroups = "action-groups",
    /// `Confirmation dialogs`
    ConfirmationDialogs = "confirmation-dialogs",
    /// `Content that is no navigation, from the end of the page.`
    ContentNoNavigationFromEnd = "content-no-navigation-from-end",
    /// `Copy the set's name`
    CopyTheSetName = "copy-the-set-name",
    /// `Delete the file`
    DeleteFile = "delete-file",
    /// The question of the dialog that deletes the row `$row`.
    DeleteRow = "delete-row",
    /// `Delete the file?`
    DeleteTheFile = "delete-the-file",
    /// `Dialogs, modals, the side panel and toasts.`
    DialogsModalsSidePanelToasts = "dialogs-modals-side-panel-toasts",
    /// `Download the file`
    DownloadTheFile = "download-the-file",
    /// `The file goes for good, and the running set reloads without it.`
    FileGoesGoodRunningSet = "file-goes-good-running-set",
    /// `The files on disk compile and replace the running set when they compile.`
    FilesDiskCompileReplaceRunning = "files-disk-compile-replace-running",
    /// `How reloading works`
    HowReloadingWorks = "how-reloading-works",
    /// `Information`
    Information = "information",
    /// `Keyboard shortcuts`
    KeyboardShortcuts = "keyboard-shortcuts",
    /// `Menus and select buttons`
    MenusAndSelects = "menus-and-selects",
    /// `Method`
    Method = "method",
    /// `Modal dialogs`
    ModalDialogs = "modal-dialogs",
    /// `More actions for the file`
    MoreActionsFile = "more-actions-file",
    /// `More alone`
    MoreAlone = "more-alone",
    /// `New name`
    NewName = "new-name",
    /// `Open the tables`
    OpenTheTables = "open-the-tables",
    /// `Opened by their trigger with no script; Escape and Cancel close them and the focus...`
    OpenedTheirTriggerNoScript = "opened-their-trigger-no-script",
    /// `OpenID Connect`
    OpenidConnect = "openid-connect",
    /// `The operator scope decides what the console may do.`
    OperatorScopeDecidesWhatConsole = "operator-scope-decides-what-console",
    /// `Overlays and feedback`
    OverlaysFeedback = "overlays-feedback",
    /// `A path below fhirconnect/ or omocl/.`
    PathBelowFhirconnectOmocl = "path-below-fhirconnect-omocl",
    /// `Policy`
    Policy = "policy",
    /// `Popover form`
    PopoverForm = "popover-form",
    /// `It opens under its trigger and, on a phone, rises from the foot of the screen.`
    PopoverFormOpensUnderTrigger = "popover-form-opens-under-trigger",
    /// `The file is provisioned and read-only.`
    ProvisionedReadOnly = "provisioned-read-only",
    /// `Refusal`
    Refusal = "refusal",
    /// `Reload`
    Reload = "reload",
    /// `Reload the set`
    ReloadSet = "reload-set",
    /// `Reload the set?`
    ReloadTheSet = "reload-the-set",
    /// `The console compiles the files on disk and swaps the running set...`
    ReloadingCompiles = "reloading-compiles",
    /// `A file that does not compile leaves the running set as it was...`
    ReloadingKeeps = "reloading-keeps",
    /// The Remove of the member `$member`.
    RemoveMember = "remove-member",
    /// The sentence of the confirmation that removes the member `$member`.
    RemoveMemberBody = "remove-member-body",
    /// The question of the confirmation that removes the member `$member`.
    RemoveMemberTitle = "remove-member-title",
    /// `Members, each with its own confirmation`
    RemoveMembers = "remove-members",
    /// `Rename`
    Rename = "rename",
    /// `Rename the file`
    RenameFile = "rename-file",
    /// `Rename a file`
    RenameTheFile = "rename-the-file",
    /// The toast once the row `$row` is deleted.
    RowDeleted = "row-deleted",
    /// `Save and reload the set`
    SaveAndReload = "save-and-reload",
    /// `Save as a draft`
    SaveAsDraft = "save-as-draft",
    /// `Save the query`
    SaveTheQuery = "save-the-query",
    /// `Save the query to the server`
    SaveTheQueryToTheServer = "save-the-query-to-the-server",
    /// `The save was refused: 412.`
    SaveWasRefused412 = "save-was-refused-412",
    /// `Saved, and the set reloaded.`
    SavedSetReloaded = "saved-set-reloaded",
    /// `Scopes`
    Scopes = "scopes",
    /// `The set compiled with 2 warnings.`
    SetCompiled2Warnings = "set-compiled-2-warnings",
    /// `The set reloaded.`
    SetReloaded = "set-reloaded",
    /// `The set reloads in the background.`
    SetReloadsBackground = "set-reloads-background",
    /// `Close a dialog, a menu or a panel`
    ShortcutClose = "shortcut-close",
    /// `Find a mapping file`
    ShortcutFind = "shortcut-find",
    /// `Open the search's first answer`
    ShortcutOpenFirst = "shortcut-open-first",
    /// `Show the keyboard shortcuts`
    ShortcutShow = "shortcut-show",
    /// `Press ? outside a field, or the button, to open the legend over this page; the line under it is the hint a listing shows at its foot.`
    ShortcutsCardIntro = "shortcuts-card-intro",
    /// `Everywhere`
    ShortcutsEverywhere = "shortcuts-everywhere",
    /// `The search`
    ShortcutsSearch = "shortcuts-search",
    /// `Side panel`
    SidePanel = "side-panel",
    /// `Signed in as`
    Signed = "signed",
    /// `Someone saved the file first; reload it to see their change.`
    SomeoneSavedTheFileFirst = "someone-saved-the-file-first",
    /// `Success`
    Success = "success",
    /// `The synthetic file goes for good; the running set reloads without it.`
    SyntheticFileGoesGoodRunning = "synthetic-file-goes-good-running",
    /// `The synthetic file was deleted.`
    SyntheticFileWasDeleted = "synthetic-file-was-deleted",
    /// `Synthetic operator`
    SyntheticOperator = "synthetic-operator",
    /// `the file`
    TheFile = "the-file",
    /// `3 files compiled; the facade serves them now.`
    ThreeFilesCompiledFacadeServes = "three-files-compiled-facade-serves",
    /// `Toasts`
    Toasts = "toasts",
    /// `Two mappings name a path the template does not hold.`
    TwoMappingsNameMissingPath = "two-mappings-name-missing-path",
    /// `{ $version } (latest of { $count })`
    VersionLatestOf = "version-latest-of",
    /// `View access`
    ViewAccess = "view-access",
    /// `Warning`
    Warning = "warning",
    /// `Ways to save`
    WaysToSave = "ways-to-save",
}
