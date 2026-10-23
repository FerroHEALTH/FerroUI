// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the tables, the pagers and the change table.

use crate::i18n::Text;

texts! {
    /// The hidden word before an added value.
    ChangeAdded = "change-added",
    /// The change table's value after.
    ChangeAfter = "change-after",
    /// The change table's value before.
    ChangeBefore = "change-before",
    /// A change table with no change.
    ChangeNone = "change-none",
    /// A side of a change that holds no value.
    ChangeNotSet = "change-not-set",
    /// The hidden word before a removed value.
    ChangeRemoved = "change-removed",
    /// The change table's setting column.
    ChangeSetting = "change-setting",
    /// The button that applies a filter menu's checked choices.
    FilterMenuShow = "filter-menu-show",
    /// A filter menu's button while it keeps `$shown` of its `$total`
    /// choices, after its `$label`.
    FilterMenuSome = "filter-menu-some",
    /// The range of a listing with no total: `$nouns` opens it.
    OpenRangeOf = "open-range-of",
    /// The range of a listing with no total whose page `$page` came back
    /// empty past the first: it is past the last `$noun`.
    OpenRangePast = "open-range-past",
    /// The pager's position: `$page` of `$pages`.
    PageOf = "page-of",
    /// The note when the server counted `$stated` rows and the listing holds `$held`.
    PartialNote = "partial-note",
    /// The range of a page: `$first` to `$last` of `$total` rows.
    Range = "range",
    /// The range of an empty listing.
    RangeEmpty = "range-empty",
    /// The range naming the rows: `$noun` is the word for `$total` rows.
    RangeOf = "range-of",
    /// The range followed by its partial note.
    RangeWithNote = "range-with-note",
    /// The library's word for `$count` rows.
    Rows = "rows",
    /// The library's word for rows, opening a sentence.
    RowsOpening = "rows-opening",
    /// The pager's size choices.
    RowsPerPage = "rows-per-page",
    /// The pager's step to the first page.
    StepFirst = "step-first",
    /// The pager's step to the last page.
    StepLast = "step-last",
    /// The pager's step to the page after.
    StepNext = "step-next",
    /// The pager's step to the page before.
    StepPrevious = "step-previous",
    /// The hidden words of a step that cannot be taken.
    StepUnavailable = "step-unavailable",
}
