// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! Every component rendered to HTML on the host, as a console's handler
//! draws it: what each must say, what each must never carry, and the
//! escaping of what it is given.

use ferrohealth_ui::i18n::Locale;

/// The locale every render test draws in.
pub(crate) fn en() -> Locale {
    Locale::english()
}
