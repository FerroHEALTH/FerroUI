// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the operation parts: the load report, the
//! transform pair, the OMOP row preview and the HL7 v2 viewer.

use crate::i18n::Text;

texts! {
    /// A value whose concept did not resolve, beside its warn mark.
    RowsConceptZero = "rows-concept-zero",
    /// A table's row count in the row preview, by `$count`.
    RowsCount = "rows-count",
    /// A value that is an empty text, set apart from NULL.
    RowsEmpty = "rows-empty",
    /// A column with no value, as the database writes it.
    RowsNull = "rows-null",
}
