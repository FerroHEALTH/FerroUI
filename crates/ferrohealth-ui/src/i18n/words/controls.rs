// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the buttons, dialogs, toasts, tabs, menus and
//! selects.

use crate::i18n::Text;

texts! {
    /// A dialog's button that leaves without acting.
    Cancel = "cancel",
    /// The line under a combobox list that draws fewer options than match.
    ComboboxMore = "combobox-more",
    /// The combobox list when the filter keeps no option.
    ComboboxNoMatch = "combobox-no-match",
    /// The help switch's label.
    HelpShow = "help-show",
    /// The name of a masked field's hold-to-show button.
    HoldShow = "hold-show",
    /// The name of a masked field's hold-to-show button until the page's
    /// script runs, which says the field stays masked.
    HoldShowNeedsScript = "hold-show-needs-script",
    /// The title row's button that opens the More menu.
    MenuMore = "menu-more",
    /// The name of a split button's toggle: the variants of `$action`.
    MenuVariants = "menu-variants",
    /// The name of a select button: its `$label` and the `$value` chosen.
    SelectName = "select-name",
    /// The badge of a tab whose section holds a draft.
    TabDraft = "tab-draft",
    /// A `$tab` holding a draft, as the phone's tab select names it.
    TabDraftChoice = "tab-draft-choice",
    /// The button of the phone's tab select that leads to the tab chosen.
    TabGo = "tab-go",
    /// A toast's dismiss button.
    ToastDismiss = "toast-dismiss",
    /// The accessible name of the toast region.
    ToastRegion = "toast-region",
    /// The danger button variant.
    VariantDanger = "variant-danger",
    /// The destructive button variant.
    VariantDestructive = "variant-destructive",
    /// The ghost button variant.
    VariantGhost = "variant-ghost",
    /// The primary button variant, as the gallery names it.
    VariantPrimary = "variant-primary",
    /// The secondary button variant.
    VariantSecondary = "variant-secondary",
}
