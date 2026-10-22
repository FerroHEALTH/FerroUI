// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the page templates.

use super::Msg;

messages! {
    /// `The console, the server it reads, and the manufacturer.`
    AboutIntro = "about-intro",
    /// `About`
    AboutTitle = "about-title",
    /// `Access log`
    AccessLog = "access-log",
    /// `Active`
    Active = "active",
    /// `Add the first location`
    AddFirstLocation = "add-first-location",
    /// `Add unit`
    AddUnit = "add-unit",
    /// `Added`
    Added = "added",
    /// `All units`
    AllUnits = "all-units",
    /// `Audit`
    Audit = "audit",
    /// `Best effort`
    BestEffort = "best-effort",
    /// `A caller may ask for a partial answer.`
    BestEffortDesc = "best-effort-desc",
    /// `Id`
    ColumnId = "column-id",
    /// `Location`
    ColumnLocation = "column-location",
    /// `State`
    ColumnState = "column-state",
    /// `Units`
    ColumnUnits = "column-units",
    /// `Copy the location id`
    CopyLocationId = "copy-location-id",
    /// `{ $count } degraded`
    DegradedCount = "degraded-count",
    /// `Delete the location`
    DeleteLocation = "delete-location",
    /// `The synthetic location and its units go for good.`
    DeleteLocationBody = "delete-location-body",
    /// `Delete the location?`
    DeleteTheLocation = "delete-the-location",
    /// `Edit`
    EditWord = "edit-word",
    /// `Export as JSON`
    ExportJson = "export-json",
    /// `It cannot tell whether you are signed in, so it shows nothing. Ask whoever runs the console to restart it; the server keeps answering its clients meanwhile.`
    FaultBody = "fault-body",
    /// `The console cannot read its sessions`
    FaultTitle = "fault-title",
    /// `State: reachable`
    FilterReachable = "filter-reachable",
    /// `Location name or id`
    FindLocation = "find-location",
    /// `You are signed in as Demo Operator, but your access token carries no operator scope, so the server refused it. Ask your identity provider's administrator for the scope, then sign in again.`
    ForbiddenBody = "forbidden-body",
    /// `This view needs the operator scope`
    ForbiddenTitle = "forbidden-title",
    /// `General`
    General = "general",
    /// `How settings work`
    HowSettingsWork = "how-settings-work",
    /// `Business Source License 1.1`
    LicenceName = "licence-name",
    /// `Limits`
    Limits = "limits",
    /// `Locatie Noord`
    LocatieNoord = "locatie-noord",
    /// `Location 1 of 5`
    LocationOf = "location-of",
    /// `Location loc-n · 3 units`
    LocationSubtitle = "location-subtitle",
    /// `location`
    LocationWord = "location-word",
    /// `Location`
    LocationWordTitle = "location-word-title",
    /// `Locations`
    Locations = "locations",
    /// `{ $count ->`
    LocationsCount = "locations-count",
    /// `Managed`
    ManagedBadge = "managed-badge",
    /// `Merge with another location`
    MergeLocation = "merge-location",
    /// `Only an administrator merges locations.`
    MergeReason = "merge-reason",
    /// `Which units a request asks.`
    ModeDesc = "mode-desc",
    /// `Mode`
    ModeSetting = "mode-setting",
    /// `More actions for the location`
    MoreActionsLocation = "more-actions-location",
    /// `More actions for the settings`
    MoreActionsSettings = "more-actions-settings",
    /// `nearest`
    Nearest = "nearest",
    /// `New location`
    NewLocation = "new-location",
    /// `No location takes part yet.`
    NoLocation = "no-location",
    /// `The gallery knows no location loc-x. It may be mistyped, or the location was removed; every location is listed on Locations.`
    NotFoundTemplateBody = "not-found-template-body",
    /// `No location at this address`
    NotFoundTitle = "not-found-title",
    /// `offered`
    Offered = "offered",
    /// `Previous and next`
    QueueSteps = "queue-steps",
    /// `Retries`
    Retries = "retries",
    /// `How often a failed call is tried again.`
    RetriesDesc = "retries-desc",
    /// `Revision 7 · applied 2026-10-14 by Demo Steward · in step`
    RevisionLine = "revision-line",
    /// `The scope is`
    ScopeIs = "scope-is",
    /// `The self-description`
    SelfDescription = "self-description",
    /// `What the server declares, as every client reads it.`
    SelfDescriptionLine = "self-description-line",
    /// `Server`
    Server = "server",
    /// `Example content: deployment-wide settings, one section per tab.`
    SettingsIntro = "settings-intro",
    /// `How calls behave. Changes apply at once unless a row says Next start.`
    SettingsPanelIntro = "settings-panel-intro",
    /// `Settings sections`
    SettingsSections = "settings-sections",
    /// `Settings`
    SettingsTab = "settings-tab",
    /// `Show`
    Show = "show",
    /// `Sign in again`
    SignInAgain = "sign-in-again",
    /// `Standing`
    Standing = "standing",
    /// `Degraded`
    StateDegraded = "state-degraded",
    /// `Excluded`
    StateExcluded = "state-excluded",
    /// `Reachable`
    StateReachable = "state-reachable",
    /// `Example content: the template's parts, not a product's.`
    TemplateExample = "template-example",
    /// `The FerroBRIDGE book`
    TheBook = "the-book",
    /// `How long one call may take, 200 to 10,000 ms.`
    TimeoutDesc = "timeout-desc",
    /// `2,500 ms`
    TimeoutValue = "timeout-value",
    /// `Units`
    Units = "units",
}
