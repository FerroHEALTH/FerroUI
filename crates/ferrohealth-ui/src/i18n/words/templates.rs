// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the page templates: the list, detail, settings
//! and About pages.

use crate::i18n::Text;

texts! {
    /// The About page's row of the console's own version.
    AboutConsole = "about-console",
    /// The About page's row of the documentation.
    AboutDocumentation = "about-documentation",
    /// The About page's row of the licence.
    AboutLicence = "about-licence",
    /// The About page's row of the maker.
    AboutMadeBy = "about-made-by",
    /// The About page's card of the maker.
    AboutManufacturer = "about-manufacturer",
    /// The About page's row of where to ask for help.
    AboutSupport = "about-support",
    /// The About page's card of versions.
    AboutVersions = "about-versions",
    /// The file-mode line's link to how configuration works.
    ConfigHow = "config-how",
    /// The file-mode line's words on when the file is read.
    ConfigReadAtStart = "config-read-at-start",
    /// The file-mode line's words before the file.
    ConfigSetIn = "config-set-in",
    /// A heading's count of the items shown of a larger set, after its
    /// middle dot.
    CountShownOf = "count-shown-of",
    /// The button that downloads a document.
    Download = "download",
    /// The link that removes every filter.
    FilterClear = "filter-clear",
    /// The accessible name of the link that removes one filter.
    FilterRemove = "filter-remove",
    /// The words before the active filters of a list.
    FilterShowing = "filter-showing",
    /// The words before the active filters, with the rows they keep.
    FilterShowingOf = "filter-showing-of",
    /// The accessible name of a setting's key hint.
    KeyCopy = "key-copy",
    /// The tooltip of a setting's key hint.
    KeyCopyTitle = "key-copy-title",
    /// A setting that takes effect at the next start.
    NextStart = "next-start",
    /// A setting's value nothing sets: the product's default.
    SettingSourceDefault = "setting-source-default",
    /// A setting's value from an environment variable, which wins over the
    /// file and a revision.
    SettingSourceEnvironment = "setting-source-environment",
    /// A managed setting's value from the configuration file.
    SettingSourceFile = "setting-source-file",
    /// The step to the next item of a queue, named with the item.
    StepperNext = "stepper-next",
    /// The step to the previous item of a queue, named with the item.
    StepperPrevious = "stepper-previous",
}
