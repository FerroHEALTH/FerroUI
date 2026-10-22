// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the error page.

use crate::i18n::Text;

texts! {
    /// The summary of an error's technical detail.
    ErrorDetail = "error-detail",
    /// The words of a 500.
    ErrorFault = "error-fault",
    /// The words of a 403.
    ErrorForbidden = "error-forbidden",
    /// The words of a 404.
    ErrorNotFound = "error-not-found",
    /// The id of the request that failed.
    ErrorRequest = "error-request",
    /// An error's status: its words, then its number.
    ErrorStatus = "error-status",
    /// The time an error happened.
    ErrorTime = "error-time",
}
