// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The Lucide icons, drawn as inline SVG beside the word they illustrate.
//!
//! Each [`Glyph`] is the inner markup of one outline of the Lucide release
//! `vendor/lucide/` holds (ISC License), and the test below holds every
//! entry to its vendored file, so a glyph never drifts from its source. An
//! icon is hidden from assistive technology, which reads the word beside
//! it: an icon never carries meaning alone (WCAG 2.2 SC 1.1.1, SC 1.4.1).

use leptos::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The inner markup of one Lucide outline, drawn on a 24 by 24 grid with a
/// two-unit round stroke in the current colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph(&'static str);

impl Glyph {
    /// The outline's markup, as the vendored file holds it.
    #[must_use]
    pub const fn markup(self) -> &'static str {
        self.0
    }
    /// Lucide `activity`.
    pub const ACTIVITY: Self = Self(
        r#"<path d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2" />"#,
    );
    /// Lucide `arrow-down`.
    pub const ARROW_DOWN: Self = Self(r#"<path d="M12 5v14" /><path d="m19 12-7 7-7-7" />"#);
    /// Lucide `arrow-left`.
    pub const ARROW_LEFT: Self = Self(r#"<path d="m12 19-7-7 7-7" /><path d="M19 12H5" />"#);
    /// Lucide `arrow-right`.
    pub const ARROW_RIGHT: Self = Self(r#"<path d="M5 12h14" /><path d="m12 5 7 7-7 7" />"#);
    /// Lucide `arrow-up-down`.
    pub const ARROW_UP_DOWN: Self = Self(
        r#"<path d="m21 16-4 4-4-4" /><path d="M17 20V4" /><path d="m3 8 4-4 4 4" /><path d="M7 4v16" />"#,
    );
    /// Lucide `arrow-up`.
    pub const ARROW_UP: Self = Self(r#"<path d="m5 12 7-7 7 7" /><path d="M12 19V5" />"#);
    /// Lucide `ban`.
    pub const BAN: Self =
        Self(r#"<circle cx="12" cy="12" r="10" /><path d="M4.929 4.929 19.07 19.071" />"#);
    /// Lucide `bell`.
    pub const BELL: Self = Self(
        r#"<path d="M10.268 21a2 2 0 0 0 3.464 0" /><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" />"#,
    );
    /// Lucide `book-a`.
    pub const BOOK_A: Self = Self(
        r#"<path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20" /><path d="m8 13 4-7 4 7" /><path d="M9.1 11h5.7" />"#,
    );
    /// Lucide `book-x`.
    pub const BOOK_X: Self = Self(
        r#"<path d="m14.5 7.5-5 5" /><path d="M4 19.5v-15A2.5 2.5 0 0 1 6.5 2H19a1 1 0 0 1 1 1v18a1 1 0 0 1-1 1H6.5a1 1 0 0 1 0-5H20" /><path d="m9.5 7.5 5 5" />"#,
    );
    /// Lucide `calendar`.
    pub const CALENDAR: Self = Self(
        r#"<path d="M8 2v3" /><path d="M16 2v3" /><rect x="3" y="3" width="18" height="18" rx="2" /><path d="M3 9h18" />"#,
    );
    /// Lucide `chart-line`.
    pub const CHART_LINE: Self =
        Self(r#"<path d="M3 3v16a2 2 0 0 0 2 2h16" /><path d="m19 9-5 5-4-4-3 3" />"#);
    /// Lucide `check`.
    pub const CHECK: Self = Self(r#"<path d="M20 6 9 17l-5-5" />"#);
    /// Lucide `chevron-down`.
    pub const CHEVRON_DOWN: Self = Self(r#"<path d="m6 9 6 6 6-6" />"#);
    /// Lucide `chevron-left`.
    pub const CHEVRON_LEFT: Self = Self(r#"<path d="m15 18-6-6 6-6" />"#);
    /// Lucide `chevron-right`.
    pub const CHEVRON_RIGHT: Self = Self(r#"<path d="m9 18 6-6-6-6" />"#);
    /// Lucide `chevron-up`.
    pub const CHEVRON_UP: Self = Self(r#"<path d="m18 15-6-6-6 6" />"#);
    /// Lucide `chevrons-left`.
    pub const CHEVRONS_LEFT: Self =
        Self(r#"<path d="m11 17-5-5 5-5" /><path d="m18 17-5-5 5-5" />"#);
    /// Lucide `chevrons-right`.
    pub const CHEVRONS_RIGHT: Self =
        Self(r#"<path d="m6 17 5-5-5-5" /><path d="m13 17 5-5-5-5" />"#);
    /// Lucide `circle-alert`.
    pub const CIRCLE_ALERT: Self = Self(
        r#"<circle cx="12" cy="12" r="10" /><line x1="12" x2="12" y1="8" y2="12" /><line x1="12" x2="12.01" y1="16" y2="16" />"#,
    );
    /// Lucide `circle-check`.
    pub const CIRCLE_CHECK: Self =
        Self(r#"<circle cx="12" cy="12" r="10" /><path d="m16 9-5.5 5.5L8 12" />"#);
    /// Lucide `circle-user`.
    pub const CIRCLE_USER: Self = Self(
        r#"<circle cx="12" cy="12" r="10" /><circle cx="12" cy="10" r="3" /><path d="M7 20.662V19a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v1.662" />"#,
    );
    /// Lucide `circle-x`.
    pub const CIRCLE_X: Self =
        Self(r#"<circle cx="12" cy="12" r="10" /><path d="m15 9-6 6" /><path d="m9 9 6 6" />"#);
    /// Lucide `clipboard-copy`.
    pub const CLIPBOARD_COPY: Self = Self(
        r#"<rect width="8" height="4" x="8" y="2" rx="1" ry="1" /><path d="M8 4H6a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-2" /><path d="M16 4h2a2 2 0 0 1 2 2v4" /><path d="M21 14H11" /><path d="m15 10-4 4 4 4" />"#,
    );
    /// Lucide `clock`.
    pub const CLOCK: Self = Self(r#"<circle cx="12" cy="12" r="10" /><path d="M12 6v6l4 2" />"#);
    /// Lucide `code`.
    pub const CODE: Self = Self(r#"<path d="m16 18 6-6-6-6" /><path d="m8 6-6 6 6 6" />"#);
    /// Lucide `compass`.
    pub const COMPASS: Self = Self(
        r#"<circle cx="12" cy="12" r="10" /><path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z" />"#,
    );
    /// Lucide `copy`.
    pub const COPY: Self = Self(
        r#"<rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />"#,
    );
    /// Lucide `database`.
    pub const DATABASE: Self = Self(
        r#"<ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M3 5V19A9 3 0 0 0 21 19V5" /><path d="M3 12A9 3 0 0 0 21 12" />"#,
    );
    /// Lucide `download`.
    pub const DOWNLOAD: Self = Self(
        r#"<path d="M12 15V3" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path d="m7 10 5 5 5-5" />"#,
    );
    /// Lucide `ellipsis`.
    pub const ELLIPSIS: Self = Self(
        r#"<circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /><circle cx="5" cy="12" r="1" />"#,
    );
    /// Lucide `equal-not`.
    pub const EQUAL_NOT: Self = Self(
        r#"<line x1="5" x2="19" y1="9" y2="9" /><line x1="5" x2="19" y1="15" y2="15" /><line x1="19" x2="5" y1="5" y2="19" />"#,
    );
    /// Lucide `eye`.
    pub const EYE: Self = Self(
        r#"<path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" /><circle cx="12" cy="12" r="3" />"#,
    );
    /// Lucide `eye-off`.
    pub const EYE_OFF: Self = Self(
        r#"<path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" /><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" /><path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" /><path d="m2 2 20 20" />"#,
    );
    /// Lucide `file-clock`.
    pub const FILE_CLOCK: Self = Self(
        r#"<path d="M16 22h2a2 2 0 0 0 2-2V8a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v2.85" /><path d="M14 2v5a1 1 0 0 0 1 1h5" /><path d="M8 14v2.2l1.6 1" /><circle cx="8" cy="16" r="6" />"#,
    );
    /// Lucide `file-code`.
    pub const FILE_CODE: Self = Self(
        r#"<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" /><path d="M14 2v5a1 1 0 0 0 1 1h5" /><path d="M10 12.5 8 15l2 2.5" /><path d="m14 12.5 2 2.5-2 2.5" />"#,
    );
    /// Lucide `file-plus`.
    pub const FILE_PLUS: Self = Self(
        r#"<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" /><path d="M14 2v5a1 1 0 0 0 1 1h5" /><path d="M9 15h6" /><path d="M12 18v-6" />"#,
    );
    /// Lucide `file-text`.
    pub const FILE_TEXT: Self = Self(
        r#"<path d="M6 22a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h8a2.4 2.4 0 0 1 1.704.706l3.588 3.588A2.4 2.4 0 0 1 20 8v12a2 2 0 0 1-2 2z" /><path d="M14 2v5a1 1 0 0 0 1 1h5" /><path d="M10 9H8" /><path d="M16 13H8" /><path d="M16 17H8" />"#,
    );
    /// Lucide `files`.
    pub const FILES: Self = Self(
        r#"<path d="M15 2h-4a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V8" /><path d="M16.706 2.706A2.4 2.4 0 0 0 15 2v5a1 1 0 0 0 1 1h5a2.4 2.4 0 0 0-.706-1.706z" /><path d="M5 7a2 2 0 0 0-2 2v11a2 2 0 0 0 2 2h8a2 2 0 0 0 1.732-1" />"#,
    );
    /// Lucide `flask-conical`.
    pub const FLASK_CONICAL: Self = Self(
        r#"<path d="M14 2v6a2 2 0 0 0 .245.96l5.51 10.08A2 2 0 0 1 18 22H6a2 2 0 0 1-1.755-2.96l5.51-10.08A2 2 0 0 0 10 8V2" /><path d="M6.453 15h11.094" /><path d="M8.5 2h7" />"#,
    );
    /// Lucide `folder-open`.
    pub const FOLDER_OPEN: Self = Self(
        r#"<path d="m6 14 1.5-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.54 6a2 2 0 0 1-1.95 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2" />"#,
    );
    /// Lucide `folder-plus`.
    pub const FOLDER_PLUS: Self = Self(
        r#"<path d="M12 10v6" /><path d="M9 13h6" /><path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />"#,
    );
    /// Lucide `folder-search`.
    pub const FOLDER_SEARCH: Self = Self(
        r#"<path d="M10.7 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H20a2 2 0 0 1 2 2v4.1" /><path d="m21 21-1.9-1.9" /><circle cx="17" cy="17" r="3" />"#,
    );
    /// Lucide `folder-x`.
    pub const FOLDER_X: Self = Self(
        r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" /><path d="m9.5 10.5 5 5" /><path d="m14.5 10.5-5 5" />"#,
    );
    /// Lucide `folder`.
    pub const FOLDER: Self = Self(
        r#"<path d="M20 20a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.9a2 2 0 0 1-1.69-.9L9.6 3.9A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13a2 2 0 0 0 2 2Z" />"#,
    );
    /// Lucide `funnel`.
    pub const FUNNEL: Self = Self(
        r#"<path d="M10 20a1 1 0 0 0 .553.895l2 1A1 1 0 0 0 14 21v-7a2 2 0 0 1 .517-1.341L21.74 4.67A1 1 0 0 0 21 3H3a1 1 0 0 0-.742 1.67l7.225 7.989A2 2 0 0 1 10 14z" />"#,
    );
    /// Lucide `gauge`.
    pub const GAUGE: Self =
        Self(r#"<path d="m12 14 4-4" /><path d="M3.34 19a10 10 0 1 1 17.32 0" />"#);
    /// Lucide `heart-pulse`.
    pub const HEART_PULSE: Self = Self(
        r#"<path d="M2 9.5a5.5 5.5 0 0 1 9.591-3.676.56.56 0 0 0 .818 0A5.49 5.49 0 0 1 22 9.5c0 2.29-1.5 4-3 5.5l-5.492 5.313a2 2 0 0 1-3 .019L5 15c-1.5-1.5-3-3.2-3-5.5" /><path d="M3.22 13H9.5l.5-1 2 4.5 2-7 1.5 3.5h5.27" />"#,
    );
    /// Lucide `inbox`.
    pub const INBOX: Self = Self(
        r#"<polyline points="22 12 16 12 14 15 10 15 8 12 2 12" /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />"#,
    );
    /// Lucide `info`.
    pub const INFO: Self =
        Self(r#"<circle cx="12" cy="12" r="10" /><path d="M12 16v-4" /><path d="M12 8h.01" />"#);
    /// Lucide `keyboard`.
    pub const KEYBOARD: Self = Self(
        r#"<path d="M10 8h.01" /><path d="M12 12h.01" /><path d="M14 8h.01" /><path d="M16 12h.01" /><path d="M18 8h.01" /><path d="M6 8h.01" /><path d="M7 16h10" /><path d="M8 12h.01" /><rect width="20" height="16" x="2" y="4" rx="2" />"#,
    );
    /// Lucide `layers`.
    pub const LAYERS: Self = Self(
        r#"<path d="M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83z" /><path d="M2 12a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 12" /><path d="M2 17a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 17" />"#,
    );
    /// Lucide `layout-dashboard`.
    pub const LAYOUT_DASHBOARD: Self = Self(
        r#"<rect width="7" height="9" x="3" y="3" rx="1" /><rect width="7" height="5" x="14" y="3" rx="1" /><rect width="7" height="9" x="14" y="12" rx="1" /><rect width="7" height="5" x="3" y="16" rx="1" />"#,
    );
    /// Lucide `link`.
    pub const LINK: Self = Self(
        r#"<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />"#,
    );
    /// Lucide `list-checks`.
    pub const LIST_CHECKS: Self = Self(
        r#"<path d="M13 5h8" /><path d="M13 12h8" /><path d="M13 19h8" /><path d="m3 17 2 2 4-4" /><path d="m3 7 2 2 4-4" />"#,
    );
    /// Lucide `lock`.
    pub const LOCK: Self = Self(
        r#"<rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />"#,
    );
    /// Lucide `log-in`.
    pub const LOG_IN: Self = Self(
        r#"<path d="m10 17 5-5-5-5" /><path d="M15 12H3" /><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />"#,
    );
    /// Lucide `log-out`.
    pub const LOG_OUT: Self = Self(
        r#"<path d="m16 17 5-5-5-5" /><path d="M21 12H9" /><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />"#,
    );
    /// Lucide `menu`.
    pub const MENU: Self =
        Self(r#"<path d="M4 5h16" /><path d="M4 12h16" /><path d="M4 19h16" />"#);
    /// Lucide `minus`.
    pub const MINUS: Self = Self(r#"<path d="M5 12h14" />"#);
    /// Lucide `monitor`.
    pub const MONITOR: Self = Self(
        r#"<rect width="20" height="14" x="2" y="3" rx="2" /><line x1="8" x2="16" y1="21" y2="21" /><line x1="12" x2="12" y1="17" y2="21" />"#,
    );
    /// Lucide `moon`.
    pub const MOON: Self = Self(
        r#"<path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401" />"#,
    );
    /// Lucide `network`.
    pub const NETWORK: Self = Self(
        r#"<rect x="16" y="16" width="6" height="6" rx="1" /><rect x="2" y="16" width="6" height="6" rx="1" /><rect x="9" y="2" width="6" height="6" rx="1" /><path d="M5 16v-3a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3" /><path d="M12 12V8" />"#,
    );
    /// Lucide `panel-left-close`.
    pub const PANEL_LEFT_CLOSE: Self = Self(
        r#"<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /><path d="m16 15-3-3 3-3" />"#,
    );
    /// Lucide `panel-left-open`.
    pub const PANEL_LEFT_OPEN: Self = Self(
        r#"<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /><path d="m14 9 3 3-3 3" />"#,
    );
    /// Lucide `panel-right`.
    pub const PANEL_RIGHT: Self =
        Self(r#"<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M15 3v18" />"#);
    /// Lucide `pencil`.
    pub const PENCIL: Self = Self(
        r#"<path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" /><path d="m15 5 4 4" />"#,
    );
    /// Lucide `play`.
    pub const PLAY: Self = Self(
        r#"<path d="M5 5a2 2 0 0 1 3.008-1.728l11.997 6.998a2 2 0 0 1 .003 3.458l-12 7A2 2 0 0 1 5 19z" />"#,
    );
    /// Lucide `plus`.
    pub const PLUS: Self = Self(r#"<path d="M5 12h14" /><path d="M12 5v14" />"#);
    /// Lucide `radio-tower`.
    pub const RADIO_TOWER: Self = Self(
        r#"<path d="M4.9 16.1C1 12.2 1 5.8 4.9 1.9" /><path d="M7.8 4.7a6.14 6.14 0 0 0-.8 7.5" /><circle cx="12" cy="9" r="2" /><path d="M16.2 4.8c2 2 2.26 5.11.8 7.47" /><path d="M19.1 1.9a9.96 9.96 0 0 1 0 14.1" /><path d="M9.5 18h5" /><path d="m8 22 4-11 4 11" />"#,
    );
    /// Lucide `refresh-cw`.
    pub const REFRESH_CW: Self = Self(
        r#"<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" /><path d="M8 16H3v5" />"#,
    );
    /// Lucide `rotate-ccw-clock`.
    pub const ROTATE_CCW_CLOCK: Self = Self(
        r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" /><path d="M12 7v5l4 2" />"#,
    );
    /// Lucide `rotate-ccw`.
    pub const ROTATE_CCW: Self = Self(
        r#"<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" />"#,
    );
    /// Lucide `route`.
    pub const ROUTE: Self = Self(
        r#"<circle cx="6" cy="19" r="3" /><path d="M9 19h8.5a3.5 3.5 0 0 0 0-7h-11a3.5 3.5 0 0 1 0-7H15" /><circle cx="18" cy="5" r="3" />"#,
    );
    /// Lucide `save`.
    pub const SAVE: Self = Self(
        r#"<path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" /><path d="M7 3v4a1 1 0 0 0 1 1h7" />"#,
    );
    /// Lucide `search-code`.
    pub const SEARCH_CODE: Self = Self(
        r#"<path d="m13 13.5 2-2.5-2-2.5" /><path d="m21 21-4.3-4.3" /><path d="M9 8.5 7 11l2 2.5" /><circle cx="11" cy="11" r="8" />"#,
    );
    /// Lucide `search-x`.
    pub const SEARCH_X: Self = Self(
        r#"<path d="m13.5 8.5-5 5" /><path d="m8.5 8.5 5 5" /><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />"#,
    );
    /// Lucide `search`.
    pub const SEARCH: Self =
        Self(r#"<path d="m21 21-4.34-4.34" /><circle cx="11" cy="11" r="8" />"#);
    /// Lucide `server-crash`.
    pub const SERVER_CRASH: Self = Self(
        r#"<path d="M6 10H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-2" /><path d="M6 14H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2h-2" /><path d="M6 6h.01" /><path d="M6 18h.01" /><path d="m13 6-4 6h6l-4 6" />"#,
    );
    /// Lucide `server`.
    pub const SERVER: Self = Self(
        r#"<rect width="20" height="8" x="2" y="2" rx="2" ry="2" /><rect width="20" height="8" x="2" y="14" rx="2" ry="2" /><line x1="6" x2="6.01" y1="6" y2="6" /><line x1="6" x2="6.01" y1="18" y2="18" />"#,
    );
    /// Lucide `settings`.
    pub const SETTINGS: Self = Self(
        r#"<path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" /><circle cx="12" cy="12" r="3" />"#,
    );
    /// Lucide `shield-check`.
    pub const SHIELD_CHECK: Self = Self(
        r#"<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" /><path d="m9 12 2 2 4-4" />"#,
    );
    /// Lucide `shield-off`.
    pub const SHIELD_OFF: Self = Self(
        r#"<path d="m2 2 20 20" /><path d="M5 5a1 1 0 0 0-1 1v7c0 5 3.5 7.5 7.67 8.94a1 1 0 0 0 .67.01c2.35-.82 4.48-1.97 5.9-3.71" /><path d="M9.309 3.652A12.252 12.252 0 0 0 11.24 2.28a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1v7a9.784 9.784 0 0 1-.08 1.264" />"#,
    );
    /// Lucide `sliders-horizontal`.
    pub const SLIDERS_HORIZONTAL: Self = Self(
        r#"<path d="M10 5H3" /><path d="M12 19H3" /><path d="M14 3v4" /><path d="M16 17v4" /><path d="M21 12h-9" /><path d="M21 19h-5" /><path d="M21 5h-7" /><path d="M8 10v4" /><path d="M8 12H3" />"#,
    );
    /// Lucide `square-terminal`.
    pub const SQUARE_TERMINAL: Self = Self(
        r#"<path d="m7 11 2-2-2-2" /><path d="M11 13h4" /><rect width="18" height="18" x="3" y="3" rx="2" ry="2" />"#,
    );
    /// Lucide `sun`.
    pub const SUN: Self = Self(
        r#"<circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="m4.93 4.93 1.41 1.41" /><path d="m17.66 17.66 1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="m6.34 17.66-1.41 1.41" /><path d="m19.07 4.93-1.41 1.41" />"#,
    );
    /// Lucide `tags`.
    pub const TAGS: Self = Self(
        r#"<path d="M13.172 2a2 2 0 0 1 1.414.586l6.71 6.71a2.4 2.4 0 0 1 0 3.408l-4.592 4.592a2.4 2.4 0 0 1-3.408 0l-6.71-6.71A2 2 0 0 1 6 9.172V3a1 1 0 0 1 1-1z" /><path d="M2 7v6.172a2 2 0 0 0 .586 1.414l6.71 6.71a2.4 2.4 0 0 0 3.191.193" /><circle cx="10.5" cy="6.5" r=".5" fill="currentColor" />"#,
    );
    /// Lucide `trash`.
    pub const TRASH: Self = Self(
        r#"<path d="M10 11v6" /><path d="M14 11v6" /><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6" /><path d="M3 6h18" /><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />"#,
    );
    /// Lucide `triangle-alert`.
    pub const TRIANGLE_ALERT: Self = Self(
        r#"<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" /><path d="M12 9v4" /><path d="M12 17h.01" />"#,
    );
    /// Lucide `undo`.
    pub const UNDO: Self =
        Self(r#"<path d="M3 7v6h6" /><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />"#);
    /// Lucide `upload`.
    pub const UPLOAD: Self = Self(
        r#"<path d="M12 3v12" /><path d="m17 8-5-5-5 5" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />"#,
    );
    /// Lucide `user-plus`.
    pub const USER_PLUS: Self = Self(
        r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" /><line x1="19" x2="19" y1="8" y2="14" /><line x1="22" x2="16" y1="11" y2="11" />"#,
    );
    /// Lucide `users`.
    pub const USERS: Self = Self(
        r#"<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><path d="M16 3.128a4 4 0 0 1 0 7.744" /><path d="M22 21v-2a4 4 0 0 0-3-3.87" /><circle cx="9" cy="7" r="4" />"#,
    );
    /// Lucide `x`.
    pub const X: Self = Self(r#"<path d="M18 6 6 18" /><path d="m6 6 12 12" />"#);

    /// Every glyph, with the Lucide name of its vendored file.
    pub const ALL: [(&str, Self); 94] = [
        ("activity", Self::ACTIVITY),
        ("arrow-down", Self::ARROW_DOWN),
        ("arrow-left", Self::ARROW_LEFT),
        ("arrow-right", Self::ARROW_RIGHT),
        ("arrow-up-down", Self::ARROW_UP_DOWN),
        ("arrow-up", Self::ARROW_UP),
        ("ban", Self::BAN),
        ("bell", Self::BELL),
        ("book-a", Self::BOOK_A),
        ("book-x", Self::BOOK_X),
        ("calendar", Self::CALENDAR),
        ("chart-line", Self::CHART_LINE),
        ("check", Self::CHECK),
        ("chevron-down", Self::CHEVRON_DOWN),
        ("chevron-left", Self::CHEVRON_LEFT),
        ("chevron-right", Self::CHEVRON_RIGHT),
        ("chevron-up", Self::CHEVRON_UP),
        ("chevrons-left", Self::CHEVRONS_LEFT),
        ("chevrons-right", Self::CHEVRONS_RIGHT),
        ("circle-alert", Self::CIRCLE_ALERT),
        ("circle-check", Self::CIRCLE_CHECK),
        ("circle-user", Self::CIRCLE_USER),
        ("circle-x", Self::CIRCLE_X),
        ("clipboard-copy", Self::CLIPBOARD_COPY),
        ("clock", Self::CLOCK),
        ("code", Self::CODE),
        ("compass", Self::COMPASS),
        ("copy", Self::COPY),
        ("database", Self::DATABASE),
        ("download", Self::DOWNLOAD),
        ("ellipsis", Self::ELLIPSIS),
        ("equal-not", Self::EQUAL_NOT),
        ("eye", Self::EYE),
        ("eye-off", Self::EYE_OFF),
        ("file-clock", Self::FILE_CLOCK),
        ("file-code", Self::FILE_CODE),
        ("file-plus", Self::FILE_PLUS),
        ("file-text", Self::FILE_TEXT),
        ("files", Self::FILES),
        ("flask-conical", Self::FLASK_CONICAL),
        ("folder-open", Self::FOLDER_OPEN),
        ("folder-plus", Self::FOLDER_PLUS),
        ("folder-search", Self::FOLDER_SEARCH),
        ("folder-x", Self::FOLDER_X),
        ("folder", Self::FOLDER),
        ("funnel", Self::FUNNEL),
        ("gauge", Self::GAUGE),
        ("heart-pulse", Self::HEART_PULSE),
        ("inbox", Self::INBOX),
        ("info", Self::INFO),
        ("keyboard", Self::KEYBOARD),
        ("layers", Self::LAYERS),
        ("layout-dashboard", Self::LAYOUT_DASHBOARD),
        ("link", Self::LINK),
        ("list-checks", Self::LIST_CHECKS),
        ("lock", Self::LOCK),
        ("log-in", Self::LOG_IN),
        ("log-out", Self::LOG_OUT),
        ("menu", Self::MENU),
        ("minus", Self::MINUS),
        ("monitor", Self::MONITOR),
        ("moon", Self::MOON),
        ("network", Self::NETWORK),
        ("panel-left-close", Self::PANEL_LEFT_CLOSE),
        ("panel-left-open", Self::PANEL_LEFT_OPEN),
        ("panel-right", Self::PANEL_RIGHT),
        ("pencil", Self::PENCIL),
        ("play", Self::PLAY),
        ("plus", Self::PLUS),
        ("radio-tower", Self::RADIO_TOWER),
        ("refresh-cw", Self::REFRESH_CW),
        ("rotate-ccw-clock", Self::ROTATE_CCW_CLOCK),
        ("rotate-ccw", Self::ROTATE_CCW),
        ("route", Self::ROUTE),
        ("save", Self::SAVE),
        ("search-code", Self::SEARCH_CODE),
        ("search-x", Self::SEARCH_X),
        ("search", Self::SEARCH),
        ("server-crash", Self::SERVER_CRASH),
        ("server", Self::SERVER),
        ("settings", Self::SETTINGS),
        ("shield-check", Self::SHIELD_CHECK),
        ("shield-off", Self::SHIELD_OFF),
        ("sliders-horizontal", Self::SLIDERS_HORIZONTAL),
        ("square-terminal", Self::SQUARE_TERMINAL),
        ("sun", Self::SUN),
        ("tags", Self::TAGS),
        ("trash", Self::TRASH),
        ("triangle-alert", Self::TRIANGLE_ALERT),
        ("undo", Self::UNDO),
        ("upload", Self::UPLOAD),
        ("user-plus", Self::USER_PLUS),
        ("users", Self::USERS),
        ("x", Self::X),
    ];
}

impl Glyph {
    /// The Lucide name of the glyph's vendored file, as [`Glyph::ALL`]
    /// lists it.
    #[must_use]
    pub fn name(self) -> Option<&'static str> {
        Self::ALL
            .iter()
            .find(|(_, glyph)| *glyph == self)
            .map(|(name, _)| *name)
    }

    /// The glyph whose vendored file Lucide names `name`, if the crate
    /// holds it.
    #[must_use]
    pub fn named(name: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .find(|(held, _)| *held == name)
            .map(|(_, glyph)| *glyph)
    }
}

// NOTE: leptos-ui.md §15.2; an island's glyph prop crosses to the browser as its Lucide name.
impl Serialize for Glyph {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let name = self
            .name()
            .ok_or_else(|| serde::ser::Error::custom("a glyph that Glyph::ALL does not list"))?;
        serializer.serialize_str(name)
    }
}

impl<'de> Deserialize<'de> for Glyph {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let name = std::borrow::Cow::<'de, str>::deserialize(deserializer)?;
        Self::named(&name)
            .ok_or_else(|| serde::de::Error::custom(format!("no Lucide glyph named {name}")))
    }
}

/// Returns `glyph` as an inline SVG `size` pixels square, hidden from
/// assistive technology.
///
/// `size` is a CSS pixel count as text, such as `"16"`; the icon takes the
/// text colour of its parent.
#[must_use]
pub fn icon(glyph: Glyph, size: &'static str) -> impl IntoView + use<> {
    icon_classed(glyph, size, "")
}

/// Returns `glyph` as [`icon`] draws it, with the classes `class` beside the
/// library's own.
#[must_use]
pub fn icon_classed(
    glyph: Glyph,
    size: &'static str,
    class: &'static str,
) -> impl IntoView + use<> {
    let class = if class.is_empty() {
        String::from("fui-icon")
    } else {
        format!("fui-icon {class}")
    };
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width=size
            height=size
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
            focusable="false"
            class=class
            inner_html=glyph.markup()
        ></svg>
    }
}

#[cfg(test)]
mod tests {
    use leptos::prelude::RenderHtml as _;

    use super::Glyph;

    /// The inner markup of the vendored outline `name`, each line trimmed and
    /// the lines joined, as the glyph table writes it.
    fn vendored(name: &str) -> String {
        let path = format!(
            "{}/vendor/lucide/icons/{name}.svg",
            env!("CARGO_MANIFEST_DIR")
        );
        let text = std::fs::read_to_string(&path).expect("the outline is vendored");
        let (_, body) = text
            .split_once("\n>\n")
            .expect("the opening tag ends on its own line");
        let (body, _) = body.split_once("</svg>").expect("the outline closes");
        body.lines().map(str::trim).collect()
    }

    #[test]
    fn a_glyph_crosses_to_the_browser_as_its_lucide_name() {
        let json = serde_json::to_string(&Glyph::SHIELD_CHECK).expect("a listed glyph serializes");
        assert_eq!(r#""shield-check""#, json);
        let back: Glyph = serde_json::from_str(&json).expect("its name reads back");
        assert_eq!(Glyph::SHIELD_CHECK, back);
        assert!(serde_json::from_str::<Glyph>(r#""no-such-glyph""#).is_err());
    }

    #[test]
    fn every_glyph_is_its_vendored_outline() {
        for (name, glyph) in Glyph::ALL {
            assert_eq!(vendored(name), glyph.markup(), "{name}");
        }
    }

    #[test]
    fn every_vendored_outline_has_a_glyph() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/vendor/lucide/icons");
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .expect("the icons are vendored")
            .map(|entry| {
                entry
                    .expect("an entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .filter_map(|file| file.strip_suffix(".svg").map(str::to_owned))
            .collect();
        names.sort();
        let mut listed: Vec<&str> = Glyph::ALL.iter().map(|(name, _)| *name).collect();
        listed.sort_unstable();
        assert_eq!(
            names, listed,
            "the glyph table lists every vendored outline"
        );
    }

    #[test]
    fn an_icon_is_hidden_and_draws_its_outline() {
        let html = super::icon(Glyph::CHECK, "16").to_html();
        assert!(html.contains(r#"aria-hidden="true""#), "{html}");
        assert!(html.contains(r#"width="16""#), "{html}");
        assert!(html.contains(Glyph::CHECK.markup()), "{html}");
        assert!(!html.contains("style="), "{html}");
    }
}
