# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Shared markup for the FerroPIX console mockups: the FerroUI shell and the
component patterns, written as the library's Rust views emit them."""
import html as _html

# Lucide outlines, copied from ferrohealth-ui/src/icon.rs (ISC licence).
GLYPHS = {
    "activity": '<path d="M22 12h-2.48a2 2 0 0 0-1.93 1.46l-2.35 8.36a.25.25 0 0 1-.48 0L9.24 2.18a.25.25 0 0 0-.48 0l-2.35 8.36A2 2 0 0 1 4.49 12H2" />',
    "arrow-down": '<path d="M12 5v14" /><path d="m19 12-7 7-7-7" />',
    "arrow-up-down": '<path d="m21 16-4 4-4-4" /><path d="M17 20V4" /><path d="m3 8 4-4 4 4" /><path d="M7 4v16" />',
    "bell": '<path d="M10.268 21a2 2 0 0 0 3.464 0" /><path d="M3.262 15.326A1 1 0 0 0 4 17h16a1 1 0 0 0 .74-1.673C19.41 13.956 18 12.499 18 8A6 6 0 0 0 6 8c0 4.499-1.411 5.956-2.738 7.326" />',
    "check": '<path d="M20 6 9 17l-5-5" />',
    "chevron-down": '<path d="m6 9 6 6 6-6" />',
    "chevron-left": '<path d="m15 18-6-6 6-6" />',
    "chevron-up": '<path d="m18 15-6-6-6 6" />',
    "chevron-right": '<path d="m9 18 6-6-6-6" />',
    "chevrons-left": '<path d="m11 17-5-5 5-5" /><path d="m18 17-5-5 5-5" />',
    "chevrons-right": '<path d="m6 17 5-5-5-5" /><path d="m13 17 5-5-5-5" />',
    "circle-alert": '<circle cx="12" cy="12" r="10" /><line x1="12" x2="12" y1="8" y2="12" /><line x1="12" x2="12.01" y1="16" y2="16" />',
    "circle-check": '<circle cx="12" cy="12" r="10" /><path d="m16 9-5.5 5.5L8 12" />',
    "circle-user": '<circle cx="12" cy="12" r="10" /><circle cx="12" cy="10" r="3" /><path d="M7 20.662V19a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v1.662" />',
    "circle-x": '<circle cx="12" cy="12" r="10" /><path d="m15 9-6 6" /><path d="m9 9 6 6" />',
    "clock": '<circle cx="12" cy="12" r="10" /><path d="M12 6v6l4 2" />',
    "copy": '<rect width="14" height="14" x="8" y="8" rx="2" ry="2" /><path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />',
    "download": '<path d="M12 15V3" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><path d="m7 10 5 5 5-5" />',
    "eye": '<path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" /><circle cx="12" cy="12" r="3" />',
    "file-clock": '<path d="M16 22h2a2 2 0 0 0 2-2V8a2.4 2.4 0 0 0-.706-1.706l-3.588-3.588A2.4 2.4 0 0 0 14 2H6a2 2 0 0 0-2 2v2.85" /><path d="M14 2v5a1 1 0 0 0 1 1h5" /><path d="M8 14v2.2l1.6 1" /><circle cx="8" cy="16" r="6" />',
    "funnel": '<path d="M10 20a1 1 0 0 0 .553.895l2 1A1 1 0 0 0 14 21v-7a2 2 0 0 1 .517-1.341L21.74 4.67A1 1 0 0 0 21 3H3a1 1 0 0 0-.742 1.67l7.225 7.989A2 2 0 0 1 10 14z" />',
    "inbox": '<polyline points="22 12 16 12 14 15 10 15 8 12 2 12" /><path d="M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z" />',
    "info": '<circle cx="12" cy="12" r="10" /><path d="M12 16v-4" /><path d="M12 8h.01" />',
    "layers": '<path d="M12.83 2.18a2 2 0 0 0-1.66 0L2.6 6.08a1 1 0 0 0 0 1.83l8.58 3.91a2 2 0 0 0 1.66 0l8.58-3.9a1 1 0 0 0 0-1.83z" /><path d="M2 12a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 12" /><path d="M2 17a1 1 0 0 0 .58.91l8.6 3.91a2 2 0 0 0 1.65 0l8.58-3.9A1 1 0 0 0 22 17" />',
    "layout-dashboard": '<rect width="7" height="9" x="3" y="3" rx="1" /><rect width="7" height="5" x="14" y="3" rx="1" /><rect width="7" height="9" x="14" y="12" rx="1" /><rect width="7" height="5" x="3" y="16" rx="1" />',
    "link": '<path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71" /><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71" />',
    "list-checks": '<path d="M13 5h8" /><path d="M13 12h8" /><path d="M13 19h8" /><path d="m3 17 2 2 4-4" /><path d="m3 7 2 2 4-4" />',
    "log-in": '<path d="m10 17 5-5-5-5" /><path d="M15 12H3" /><path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4" />',
    "log-out": '<path d="m16 17 5-5-5-5" /><path d="M21 12H9" /><path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4" />',
    "menu": '<path d="M4 5h16" /><path d="M4 12h16" /><path d="M4 19h16" />',
    "moon": '<path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401" />',
    "network": '<rect x="16" y="16" width="6" height="6" rx="1" /><rect x="2" y="16" width="6" height="6" rx="1" /><rect x="9" y="2" width="6" height="6" rx="1" /><path d="M5 16v-3a1 1 0 0 1 1-1h12a1 1 0 0 1 1 1v3" /><path d="M12 12V8" />',
    "panel-right": '<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M15 3v18" />',
    "pencil": '<path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z" /><path d="m15 5 4 4" />',
    "plus": '<path d="M5 12h14" /><path d="M12 5v14" />',
    "radio-tower": '<path d="M4.9 16.1C1 12.2 1 5.8 4.9 1.9" /><path d="M7.8 4.7a6.14 6.14 0 0 0-.8 7.5" /><circle cx="12" cy="9" r="2" /><path d="M16.2 4.8c2 2 2.26 5.11.8 7.47" /><path d="M19.1 1.9a9.96 9.96 0 0 1 0 14.1" /><path d="M9.5 18h5" /><path d="m8 22 4-11 4 11" />',
    "refresh-cw": '<path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" /><path d="M21 3v5h-5" /><path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" /><path d="M8 16H3v5" />',
    "rotate-ccw": '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" />',
    "search": '<path d="m21 21-4.34-4.34" /><circle cx="11" cy="11" r="8" />',
    "search-x": '<path d="m13.5 8.5-5 5" /><path d="m8.5 8.5 5 5" /><circle cx="11" cy="11" r="8" /><path d="m21 21-4.3-4.3" />',
    "server": '<rect width="20" height="8" x="2" y="2" rx="2" ry="2" /><rect width="20" height="8" x="2" y="14" rx="2" ry="2" /><line x1="6" x2="6.01" y1="6" y2="6" /><line x1="6" x2="6.01" y1="18" y2="18" />',
    "server-crash": '<path d="M6 10H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v4a2 2 0 0 1-2 2h-2" /><path d="M6 14H4a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-4a2 2 0 0 0-2-2h-2" /><path d="M6 6h.01" /><path d="M6 18h.01" /><path d="m13 6-4 6h6l-4 6" />',
    "settings": '<path d="M9.671 4.136a2.34 2.34 0 0 1 4.659 0 2.34 2.34 0 0 0 3.319 1.915 2.34 2.34 0 0 1 2.33 4.033 2.34 2.34 0 0 0 0 3.831 2.34 2.34 0 0 1-2.33 4.033 2.34 2.34 0 0 0-3.319 1.915 2.34 2.34 0 0 1-4.659 0 2.34 2.34 0 0 0-3.32-1.915 2.34 2.34 0 0 1-2.33-4.033 2.34 2.34 0 0 0 0-3.831A2.34 2.34 0 0 1 6.35 6.051a2.34 2.34 0 0 0 3.319-1.915" /><circle cx="12" cy="12" r="3" />',
    "shield-check": '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" /><path d="m9 12 2 2 4-4" />',
    "sliders-horizontal": '<path d="M10 5H3" /><path d="M12 19H3" /><path d="M14 3v4" /><path d="M16 17v4" /><path d="M21 12h-9" /><path d="M21 19h-5" /><path d="M21 5h-7" /><path d="M8 10v4" /><path d="M8 12H3" />',
    "sun": '<circle cx="12" cy="12" r="4" /><path d="M12 2v2" /><path d="M12 20v2" /><path d="m4.93 4.93 1.41 1.41" /><path d="m17.66 17.66 1.41 1.41" /><path d="M2 12h2" /><path d="M20 12h2" /><path d="m6.34 17.66-1.41 1.41" /><path d="m19.07 4.93-1.41 1.41" />',
    "tags": '<path d="M13.172 2a2 2 0 0 1 1.414.586l6.71 6.71a2.4 2.4 0 0 1 0 3.408l-4.592 4.592a2.4 2.4 0 0 1-3.408 0l-6.71-6.71A2 2 0 0 1 6 9.172V3a1 1 0 0 1 1-1z" /><path d="M2 7v6.172a2 2 0 0 0 .586 1.414l6.71 6.71a2.4 2.4 0 0 0 3.191.193" /><circle cx="10.5" cy="6.5" r=".5" fill="currentColor" />',
    "triangle-alert": '<path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3" /><path d="M12 9v4" /><path d="M12 17h.01" />',
    "users": '<path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><path d="M16 3.128a4 4 0 0 1 0 7.744" /><path d="M22 21v-2a4 4 0 0 0-3-3.87" /><circle cx="9" cy="7" r="4" />',
    "x": '<path d="M18 6 6 18" /><path d="m6 6 12 12" />',
    # Not in FerroUI yet; the Lucide outlines the marks and the merge use.
    "equal-not": '<line x1="5" x2="19" y1="9" y2="9" /><line x1="5" x2="19" y1="15" y2="15" /><line x1="19" x2="5" y1="5" y2="19" />',
    "minus": '<path d="M5 12h14" />',
    "merge": '<path d="m8 6 4-4 4 4" /><path d="M12 2v10.3a4 4 0 0 1-1.172 2.872L4 22" /><path d="m20 22-5-5" />',
    "git-branch": '<line x1="6" x2="6" y1="3" y2="15" /><circle cx="18" cy="6" r="3" /><circle cx="6" cy="18" r="3" /><path d="M18 9a9 9 0 0 1-9 9" />',
    "keyboard": '<path d="M10 8h.01" /><path d="M12 12h.01" /><path d="M14 8h.01" /><path d="M16 12h.01" /><path d="M18 8h.01" /><path d="M6 8h.01" /><path d="M7 16h10" /><path d="M8 12h.01" /><rect width="20" height="16" x="2" y="4" rx="2" />',
    "fingerprint": '<path d="M12 10a2 2 0 0 0-2 2c0 1.02-.1 2.51-.26 4" /><path d="M14 13.12c0 2.38 0 6.38-1 8.88" /><path d="M17.29 21.02c.12-.6.43-2.3.5-3.02" /><path d="M2 12a10 10 0 0 1 18-6" /><path d="M2 16h.01" /><path d="M21.8 16c.2-2 .131-5.354 0-6" /><path d="M5 19.5C5.5 18 6 15 6 12a6 6 0 0 1 .34-2" /><path d="M8.65 22c.21-.66.45-1.32.57-2" /><path d="M9 6.8a6 6 0 0 1 9 5.2v2" />',
    "user-round-search": '<circle cx="10" cy="8" r="5" /><path d="M2 21a8 8 0 0 1 10.434-7.62" /><circle cx="18" cy="18" r="3" /><path d="m22 22-1.9-1.9" />',
    "globe": '<circle cx="12" cy="12" r="10" /><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" /><path d="M2 12h20" />',
    "undo": '<path d="M3 7v6h6" /><path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />',
    "history": '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" /><path d="M3 3v5h5" /><path d="M12 7v5l4 2" />',
    "square-check": '<rect width="18" height="18" x="3" y="3" rx="2" /><path d="m9 12 2 2 4-4" />',
    # Round 1: the arrows, the row menu, the eye toggle, the rail and the
    # status pages (Lucide outlines, ISC licence).
    "arrow-left": '<path d="m12 19-7-7 7-7" /><path d="M19 12H5" />',
    "arrow-right": '<path d="M5 12h14" /><path d="m12 5 7 7-7 7" />',
    "arrow-up": '<path d="m5 12 7-7 7 7" /><path d="M12 19V5" />',
    "ellipsis": '<circle cx="12" cy="12" r="1" /><circle cx="19" cy="12" r="1" /><circle cx="5" cy="12" r="1" />',
    "eye-off": '<path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" /><path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" /><path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" /><path d="m2 2 20 20" />',
    "lock": '<rect width="18" height="11" x="3" y="11" rx="2" ry="2" /><path d="M7 11V7a5 5 0 0 1 10 0v4" />',
    "calendar": '<path d="M8 2v4" /><path d="M16 2v4" /><rect width="18" height="18" x="3" y="4" rx="2" /><path d="M3 10h18" />',
    "wifi-off": '<path d="M12 20h.01" /><path d="M8.5 16.429a5 5 0 0 1 7 0" /><path d="M5 12.859a10 10 0 0 1 5.17-2.69" /><path d="M19 12.859a10 10 0 0 0-2.007-1.523" /><path d="M2 8.82a15 15 0 0 1 4.177-2.643" /><path d="M22 8.82a15 15 0 0 0-11.288-3.764" /><path d="m2 2 20 20" />',
    "ban": '<circle cx="12" cy="12" r="10" /><path d="m4.9 4.9 14.2 14.2" />',
    "compass": '<path d="m16.24 7.76-1.804 5.411a2 2 0 0 1-1.265 1.265L7.76 16.24l1.804-5.411a2 2 0 0 1 1.265-1.265z" /><circle cx="12" cy="12" r="10" />',
    "panel-left-close": '<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /><path d="m16 15-3-3 3-3" />',
    "panel-left-open": '<rect width="18" height="18" x="3" y="3" rx="2" /><path d="M9 3v18" /><path d="m14 9 3 3-3 3" />',
    "archive": '<rect width="20" height="5" x="2" y="3" rx="1" /><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8" /><path d="M10 12h4" />',
    "monitor": '<rect width="20" height="14" x="2" y="3" rx="2" /><line x1="8" x2="16" y1="21" y2="21" /><line x1="12" x2="12" y1="17" y2="21" />',
    "git-compare": '<circle cx="18" cy="18" r="3" /><circle cx="6" cy="6" r="3" /><path d="M13 6h3a2 2 0 0 1 2 2v7" /><path d="M11 18H8a2 2 0 0 1-2-2V9" />',
    "upload": '<path d="M12 3v12" /><path d="m17 8-5-5-5 5" /><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />',
    "file-text": '<path d="M15 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V7Z" /><path d="M14 2v4a2 2 0 0 0 2 2h4" /><path d="M10 9H8" /><path d="M16 13H8" /><path d="M16 17H8" />',
}


def e(text):
    return _html.escape(str(text), quote=True)



# Glyphs the shared library's components use (from FerroFED).
GLYPHS.update({'route': '<circle cx="6" cy="19" r="3" /><path d="M9 19h8.5a3.5 3.5 0 0 0 0-7h-11a3.5 3.5 0 0 1 0-7H15" /><circle cx="18" cy="5" r="3" />', 'database': '<ellipse cx="12" cy="5" rx="9" ry="3" /><path d="M3 5V19A9 3 0 0 0 21 19V5" /><path d="M3 12A9 3 0 0 0 21 12" />', 'square-terminal': '<path d="m7 11 2-2-2-2" /><path d="M11 13h4" /><rect width="18" height="18" x="3" y="3" rx="2" ry="2" />', 'play': '<polygon points="6 3 20 12 6 21 6 3" />', 'key-round': '<path d="M2.586 17.414A2 2 0 0 0 2 18.828V21a1 1 0 0 0 1 1h3a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h1a1 1 0 0 0 1-1v-1a1 1 0 0 1 1-1h.172a2 2 0 0 0 1.414-.586l.814-.814a6.5 6.5 0 1 0-4-4z" /><circle cx="16.5" cy="7.5" r=".5" fill="currentColor" />', 'building-2': '<path d="M6 22V4a2 2 0 0 1 2-2h8a2 2 0 0 1 2 2v18Z" /><path d="M6 12H4a2 2 0 0 0-2 2v6a2 2 0 0 0 2 2h2" /><path d="M18 9h2a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2h-2" /><path d="M10 6h4" /><path d="M10 10h4" /><path d="M10 14h4" /><path d="M10 18h4" />', 'timer': '<line x1="10" x2="14" y1="2" y2="2" /><line x1="12" x2="15" y1="14" y2="11" /><circle cx="12" cy="14" r="8" />', 'user-check': '<path d="m16 11 2 2 4-4" /><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2" /><circle cx="9" cy="7" r="4" />', 'heart-pulse': '<path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z" /><path d="M3.22 12H9.5l.5-1 2 4.5 2-7 1.5 3.5h5.27" />', 'shield-alert': '<path d="M20 13c0 5-3.5 7.5-7.66 8.95a1 1 0 0 1-.67-.01C7.5 20.5 4 18 4 13V6a1 1 0 0 1 1-1c2 0 4.5-1.2 6.24-2.72a1.17 1.17 0 0 1 1.52 0C14.51 3.81 17 5 19 5a1 1 0 0 1 1 1z" /><path d="M12 8v4" /><path d="M12 16h.01" />', 'save': '<path d="M15.2 3a2 2 0 0 1 1.4.6l3.8 3.8a2 2 0 0 1 .6 1.4V19a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2z" /><path d="M17 21v-7a1 1 0 0 0-1-1H8a1 1 0 0 0-1 1v7" /><path d="M7 3v4a1 1 0 0 0 1 1h7" />', 'trash-2': '<path d="M3 6h18" /><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" /><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" /><line x1="10" x2="10" y1="11" y2="17" /><line x1="14" x2="14" y1="11" y2="17" />', 'braces': '<path d="M8 3H7a2 2 0 0 0-2 2v5a2 2 0 0 1-2 2 2 2 0 0 1 2 2v5c0 1.1.9 2 2 2h1" /><path d="M16 21h1a2 2 0 0 0 2-2v-5c0-1.1.9-2 2-2a2 2 0 0 1-2-2V5a2 2 0 0 0-2-2h-1" />', 'plug': '<path d="M12 22v-5" /><path d="M9 8V2" /><path d="M15 8V2" /><path d="M18 8v5a4 4 0 0 1-4 4h-4a4 4 0 0 1-4-4V8Z" />', 'send': '<path d="M14.536 21.686a.5.5 0 0 0 .937-.024l6.5-19a.496.496 0 0 0-.635-.635l-19 6.5a.5.5 0 0 0-.024.937l7.93 3.18a2 2 0 0 1 1.112 1.11z" /><path d="m21.854 2.147-10.94 10.939" />', 'split': '<path d="M16 3h5v5" /><path d="M8 3H3v5" /><path d="M12 22v-8.3a4 4 0 0 0-1.172-2.872L3 3" /><path d="m15 9 6-6" />', 'eraser': '<path d="m7 21-4.3-4.3c-1-1-1-2.5 0-3.4l9.6-9.6c1-1 2.5-1 3.4 0l5.6 5.6c1 1 1 2.5 0 3.4L13 21" /><path d="M22 21H7" /><path d="m5 11 9 9" />', 'badge-check': '<path d="M3.85 8.62a4 4 0 0 1 4.78-4.77 4 4 0 0 1 6.74 0 4 4 0 0 1 4.78 4.78 4 4 0 0 1 0 6.74 4 4 0 0 1-4.77 4.78 4 4 0 0 1-6.75 0 4 4 0 0 1-4.78-4.77 4 4 0 0 1 0-6.76Z" /><path d="m9 12 2 2 4-4" />', 'hourglass': '<path d="M5 22h14" /><path d="M5 2h14" /><path d="M17 22v-4.172a2 2 0 0 0-.586-1.414L12 12l-4.414 4.414A2 2 0 0 0 7 17.828V22" /><path d="M7 2v4.172a2 2 0 0 0 .586 1.414L12 12l4.414-4.414A2 2 0 0 0 17 6.172V2" />', 'hand': '<path d="M18 11V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2" /><path d="M14 10V4a2 2 0 0 0-2-2a2 2 0 0 0-2 2v2" /><path d="M10 10.5V6a2 2 0 0 0-2-2a2 2 0 0 0-2 2v8" /><path d="M18 8a2 2 0 1 1 4 0v6a8 8 0 0 1-8 8h-2c-2.8 0-4.5-.86-5.99-2.34l-3.6-3.6a2 2 0 0 1 2.83-2.82L7 15" />', 'scroll-text': '<path d="M15 12h-5" /><path d="M15 8h-5" /><path d="M19 17V5a2 2 0 0 0-2-2H4" /><path d="M8 21h12a2 2 0 0 0 2-2v-1a1 1 0 0 0-1-1H11a1 1 0 0 0-1 1v1a2 2 0 1 1-4 0V5a2 2 0 1 0-4 0v2a1 1 0 0 0 1 1h3" />', 'list-filter': '<path d="M2 5h20" /><path d="M6 12h12" /><path d="M9 19h6" />'})

def icon(name, size="16", cls=""):
    classes = "fui-icon" + (f" {cls}" if cls else "")
    return (
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 24 24" '
        f'fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" '
        f'aria-hidden="true" focusable="false" class="{classes}">{GLYPHS[name]}</svg>'
    )


# ---- status marks ----

def cap_first(word):
    """State words start with a capital (family rule 12)."""
    w = str(word)
    return w[:1].upper() + w[1:]


def pill(tone, word):
    return f'<span class="fui-pill fui-tone-{tone}">{e(cap_first(word))}</span>'


def badge(tone, word):
    return f'<span class="fui-badge fui-tone-{tone}">{e(cap_first(word))}</span>'


def status(tone, glyph, word, detail=None):
    d = f'<span class="fui-status-detail">{e(detail)}</span>' if detail else ""
    return f'<span class="fui-status fui-tone-{tone}">{icon(glyph, "14")}{e(cap_first(word))}{d}</span>'


def chip(text, title=None):
    """An id in the code face (a person id, a request id)."""
    t = f' title="{e(title)}"' if title else ""
    return f'<span class="fui-chip"{t}>{e(text)}</span>'


def tag(text):
    """A name in the UI face: a domain, a channel (review item 10)."""
    return f'<span class="fui-tag">{e(text)}</span>'


KEY_WORDS = {"left": "←", "right": "→", "up": "↑", "down": "↓"}


def key_cap(k):
    """A key as printed on the keyboard: letters as capitals."""
    shown = KEY_WORDS.get(k, k)
    if len(shown) == 1 and shown.isalpha():
        shown = shown.upper()
    return shown


def kbd(keys):
    return f'<kbd class="fui-kbd">{e(key_cap(keys))}</kbd>'


# ---- time: one rule on every screen (review item 15) ----
# Today is relative ("today 09:42"); any other day is absolute
# ("2026-10-20 09:41"); the full timestamp is the title and the datetime.
TODAY = "2026-10-21"


def zone(date):
    """CEST from the last Sunday of March to the last Sunday of October (2026: 29 March to 25 October)."""
    md = date[5:10]
    return "CEST" if "03-29" <= md < "10-25" else "CET"


def when(iso, seconds=False):
    """iso: 'YYYY-MM-DDTHH:MM[:SS]' or 'YYYY-MM-DD'."""
    date, _, clock = iso.partition("T")
    clock = clock.rstrip("Z")
    if clock and len(clock) == 5:
        clock += ":00"
    full = f"{date} {clock} {zone(date)}" if clock else date
    if not clock:
        shown = date
    else:
        hm = clock if seconds else clock[:5]
        shown = f"today {hm}" if date == TODAY else f"{date} {hm}"
    dt = iso if iso.endswith("Z") or not clock else iso
    return f'<time class="fui-time" datetime="{e(dt)}" title="{e(full)}">{e(shown)}</time>'


def time_(iso, shown=None):
    return when(iso)


# ---- buttons ----

def button(variant, label, glyph=None, small=False, type_="button", attrs="", keys=None, cls=""):
    """`keys`: the shortcut; drawn as a quiet cap after the label at wide
    widths (the one responsive rule hides it on a phone) and announced
    through aria-keyshortcuts."""
    c = f"fui-button fui-button-{variant}" + (" fui-button-small" if small else "") + (f" {cls}" if cls else "")
    g = icon(glyph, "16") if glyph else ""
    k = ""
    if keys:
        k = kbd(keys)
        attrs += f' aria-keyshortcuts="{e(aria_key(keys))}"'
    return f'<button type="{type_}" class="{c}" {attrs}>{g}{label}{k}</button>'


def aria_key(keys):
    m = {"left": "ArrowLeft", "right": "ArrowRight", "up": "ArrowUp", "down": "ArrowDown", "Esc": "Escape"}
    return m.get(keys, keys.upper() if len(keys) == 1 and keys.isalpha() else keys)


def busy_button(variant, label, small=False):
    c = f"fui-button fui-button-{variant}" + (" fui-button-small" if small else "")
    return f'<button type="button" class="{c}" aria-busy="true" disabled><span class="fui-spinner-ring" aria-hidden="true"></span>{label}</button>'


def link_button(variant, label, href="#", glyph=None, small=False, attrs="", keys=None, glyph_end=None):
    cls = f"fui-button fui-button-{variant}" + (" fui-button-small" if small else "")
    g = icon(glyph, "16") if glyph else ""
    ge = icon(glyph_end, "16") if glyph_end else ""
    k = ""
    if keys:
        k = kbd(keys)
        attrs += f' aria-keyshortcuts="{e(aria_key(keys))}"'
    return f'<a href="{href}" class="{cls}" {attrs}>{g}{label}{ge}{k}</a>'


def icon_button(glyph, label, attrs="", cls=""):
    c = "fui-icon-button" + (f" {cls}" if cls else "")
    return f'<button type="button" class="{c}" aria-label="{e(label)}" title="{e(label)}" {attrs}>{icon(glyph, "16")}</button>'


def copy_button(label="Copy", what="the person id"):
    return (
        '<span class="fui-copy">'
        + button("secondary", label, "copy", small=True, attrs=f'aria-label="Copy {e(what)}"')
        + "</span>"
    )


# ---- forms ----

def field(id_, label, control, help_=None, error=None, cls=""):
    h = f'<p id="{id_}-help" class="fui-help">{help_}</p>' if help_ else ""
    er = f'<p id="{id_}-error" class="fui-help fui-error">{icon("circle-alert", "14")} {error}</p>' if error else ""
    c = "fui-field" + (f" {cls}" if cls else "")
    return f'<div class="{c}"><label for="{id_}" class="fui-label">{label}</label>{control}{h}{er}</div>'


def input_(id_, value="", placeholder="", type_="text", extra="", cls="fui-input"):
    ph = f' placeholder="{e(placeholder)}"' if placeholder else ""
    return (
        f'<input id="{id_}" name="{id_}" type="{type_}" class="{cls}" value="{e(value)}"'
        f'{ph} autocomplete="off" {extra} />'
    )


def select(id_, options, chosen=None, extra="", cls="fui-select"):
    opts = "".join(
        f'<option value="{e(v)}"{" selected" if v == chosen else ""}>{e(w)}</option>' for v, w in options
    )
    return f'<select id="{id_}" name="{id_}" class="{cls}" {extra}>{opts}</select>'


def textarea(id_, value="", placeholder="", rows=2, extra="", cls="fui-textarea fui-textarea-prose"):
    ph = f' placeholder="{e(placeholder)}"' if placeholder else ""
    return (
        f'<textarea id="{id_}" name="{id_}" class="{cls}" rows="{rows}"'
        f'{ph} {extra}>{e(value)}</textarea>'
    )


def choice_group(name, legend, options, checked):
    segs = "".join(
        f'<label for="{name}-{v}" class="fui-segment"><input type="radio" id="{name}-{v}" name="{name}" '
        f'value="{v}" class="fui-sr-only"{" checked" if v == checked else ""} />{e(w)}</label>'
        for v, w in options
    )
    return (
        f'<fieldset class="fui-fieldset fui-choice"><legend class="fui-label">{legend}</legend>'
        f'<div class="fui-segmented">{segs}</div></fieldset>'
    )


def segmented_form(label, choices, current):
    btns = "".join(
        f'<button type="submit" name="segment" value="{v}" class="fui-segment" aria-pressed="{"true" if v == current else "false"}">{w}</button>'
        for v, w in choices
    )
    return f'<form method="post" action="#"><div role="group" aria-label="{label}" class="fui-segmented">{btns}</div></form>'


def check(id_, label, checked=False, help_=None, extra=""):
    h = f'<span class="fui-help">{help_}</span>' if help_ else ""
    return (
        f'<div class="fui-check"><input type="checkbox" id="{id_}" name="{id_}"{" checked" if checked else ""} {extra} />'
        f'<label for="{id_}" class="fui-check-text"><span class="fui-label">{label}</span>{h}</label></div>'
    )


# ---- surfaces ----

def card(title, body, wide=False, extra_cls="", actions=""):
    cls = "fui-card" + (" fui-card-wide" if wide else "") + (f" {extra_cls}" if extra_cls else "")
    if title and actions:
        h = f'<div class="fui-panel-head mk-card-head"><h2 class="fui-card-title"><span>{title}</span></h2><div class="fui-actions">{actions}</div></div>'
    else:
        h = f'<h2 class="fui-card-title"><span>{title}</span></h2>' if title else ""
    return f'<section class="{cls}">{h}{body}</section>'


def titled_panel(title, actions, body):
    return (
        f'<section class="fui-card fui-panel"><div class="fui-panel-head"><h2 class="fui-card-title"><span>{title}</span></h2>'
        f'<div class="fui-actions">{actions}</div></div>{body}</section>'
    )


def stat_card(glyph, value, label, mark="", href=None):
    inner = (
        f'<span class="fui-stat-glyph">{icon(glyph, "20")}</span><div class="fui-stat-text">'
        f'<div class="fui-stat-value">{value}</div><div class="fui-stat-label"><span title="{e(label)}">{e(label)}</span>{mark}</div></div>'
    )
    if href:
        return f'<a href="{href}" class="fui-card fui-stat">{inner}</a>'
    return f'<div class="fui-card fui-stat">{inner}</div>'


def definitions(rows):
    return '<dl class="fui-facts">' + "".join(f"<dt>{k}</dt><dd>{v}</dd>" for k, v in rows) + "</dl>"


def facts_row(rows):
    return '<dl class="fui-facts-row">' + "".join(f"<div><dt>{k}</dt><dd>{v}</dd></div>" for k, v in rows) + "</dl>"


def disclosure(summary, body, open_=False):
    return f'<details class="fui-disclosure"{" open" if open_ else ""}><summary>{summary}</summary><div>{body}</div></details>'


def well(body):
    return f'<div class="fui-well">{body}</div>'


def page_heading(title, actions="", subtitle=None, crumbs=(), mono=False, badge=None):
    trail = ""
    if crumbs:
        links = "".join(
            f'<li><a href="{href}"><span class="fui-crumb-back" aria-hidden="true">{icon("arrow-left", "14")}</span>{e(label)}</a><span class="fui-crumb-sep" aria-hidden="true">/</span></li>'
            for label, href in crumbs
        )
        trail = f'<nav class="fui-crumbs" aria-label="Breadcrumb"><ol>{links}<li aria-current="page">{e(title)}</li></ol></nav>'
    h1 = f'<h1{" class=\"fui-page-title-mono\"" if mono else ""}>{title}</h1>'
    sub = f'<p class="fui-page-subtitle">{subtitle}</p>' if subtitle else ""
    if badge:
        h1 = f'<div class="fui-page-title-line">{h1}{badge}</div>'
    return f'<header class="fui-page-header">{trail}<div class="fui-page-heading">{h1}<div class="fui-actions">{actions}</div></div>{sub}</header>'


def tabs(label, items, current, focus=None, hover=None, scrolled=False):
    """Binder tabs (review item 5). items: (key, words, href, count|None)."""
    links = ""
    for item in items:
        key, words, href = item[:3]
        count = item[3] if len(item) > 3 else None
        cur = ' aria-current="page"' if key == current else ""
        cls = "fui-tab" + (" mk-focus" if key == focus else "") + (" mk-hover" if key == hover else "")
        c = f'<span class="fui-tab-count" aria-label="{count} item{"" if count == 1 else "s"}">{count}</span>' if count is not None else ""
        links += f'<a href="{href}" class="{cls}"{cur} data-tab="{key}">{words}{c}</a>'
    sc = " mk-scrolled" if scrolled else ""
    return f'<nav class="fui-tabs{sc}" aria-label="{label}">{links}</nav>'


def tabbed(tabs_html, panel_html):
    return f'<div class="fui-tabbed">{tabs_html}<div class="fui-tab-panel">{panel_html}</div></div>'


def empty_state(glyph, title, body, action="", filtered=False):
    cls = "fui-empty" + (" fui-empty-filtered" if filtered else "")
    a = f'<div class="fui-actions">{action}</div>' if action else ""
    return (
        f'<div class="{cls}"><span class="fui-empty-glyph">{icon(glyph, "24")}</span><h2>{title}</h2>'
        f'<p class="fui-empty-body">{body}</p>{a}</div>'
    )


def empty_quiet(glyph, title, line):
    return (
        f'<div class="fui-empty fui-empty-quiet"><span class="fui-empty-glyph">{icon(glyph, "20")}</span><h2>{title}</h2>'
        f'<p class="fui-empty-body">{line}</p></div>'
    )


def notice(tone, glyph, text, action=""):
    return f'<div role="status" class="fui-notice fui-tone-{tone}">{icon(glyph, "16")}<div>{text}{action}</div></div>'


def refusal(title, request=None, issues=(), evidence=None):
    # A request path may break after each slash, never inside a segment.
    req = f'<p class="fui-alert-request"><code>{e(request).replace("/", "/<wbr>")}</code></p>' if request else ""
    items = "".join(
        f'<li><span class="fui-badge fui-tone-{tone}">{cap_first(sev)}</span><span class="fui-issue-head"><code>{code}</code>:</span>'
        f'<span class="fui-issue-text">{text}</span></li>'
        for tone, sev, code, text in issues
    )
    il = f'<ul class="fui-issues">{items}</ul>' if items else ""
    ev = (
        f'<div class="fui-code-panel fui-alert-evidence" role="region" aria-label="What the server sent" tabindex="0"><pre>{e(evidence)}</pre></div>'
        if evidence
        else ""
    )
    return f'<div role="alert" class="fui-alert"><p class="fui-alert-title">{icon("circle-alert", "16")} {title}</p>{req}{il}{ev}</div>'


def toast(tone, glyph, title, line="", action=""):
    l = f"<span>{line}</span>" if line else ""
    return (
        f'<div class="fui-toast fui-tone-{tone}" role="{"alert" if tone == "bad" else "status"}">{icon(glyph, "16")}'
        f'<div class="fui-toast-text"><strong>{title}</strong>{l}</div>{action}'
        f'{icon_button("x", "Dismiss")}</div>'
    )


def toasts(*items):
    return f'<div class="fui-toasts" aria-live="polite">{"".join(items)}</div>'


def menu(items, anchored=True, up=False, start=False, label=None, id_=None, sheet=None, cancel=True):
    """The anchored menu (F13): items are menu_item() strings and MENU_SEP.
    `sheet`: below 40 rem it opens as a bottom sheet with Cancel; a labelled
    menu does so unless told otherwise."""
    sheet = bool(label) if sheet is None else sheet
    cls = "fui-menu fui-menu-compact" + (" fui-menu-anchored" if anchored else "") + (" fui-menu-up" if up else "") + (" fui-menu-start" if start else "") + (" fui-menu-sheet" if sheet else "")
    i = f' id="{id_}"' if id_ else ""
    lab = f' aria-label="{e(label)}"' if label else ""
    c = f'<button type="button" class="fui-menu-cancel">Cancel</button>' if sheet and cancel else ""
    head = f'<p class="fui-menu-sheet-title">{e(label)}</p>' if sheet and label else ""
    return f'<div{i} role="menu" class="{cls}"{lab}>{head}<div class="fui-menu-items">{"".join(items)}</div>{c}</div>'


def menu_item(label, glyph=None, keys=None, danger=False, href=None, focus=False, current=False, hover=False, desc=None, disabled=None):
    g = icon(glyph, "16") if glyph else ""
    k = kbd(keys) if keys else ""
    cls = "fui-menu-item" + (" fui-menu-danger" if danger else "") + (" mk-focus" if focus else "") + (" mk-hover" if hover else "")
    cur = ' aria-current="true"' if current else ""
    dis = f' aria-disabled="true" tabindex="-1" title="{e(disabled)}"' if disabled else ' tabindex="-1"'
    d = f'<span class="fui-menu-item-desc">{e(desc)}</span>' if desc else ""
    text = f'<span class="fui-menu-item-text"><span class="fui-menu-item-label">{label}</span>{d}</span>'
    if href and not disabled:
        return f'<a role="menuitem" href="{href}" class="{cls}"{cur}{dis}>{g}{text}{k}</a>'
    return f'<button type="button" role="menuitem" class="{cls}"{cur}{dis}>{g}{text}{k}</button>'


MENU_SEP = '<div class="fui-menu-sep" role="separator"></div>'


# ---- tables ----

def th(label, key=None, sort=None, num=False, title=None, cls=""):
    c = " ".join(x for x in ["fui-num" if num else "", cls] if x)
    cls_attr = f' class="{c}"' if c else ""
    t = f' title="{e(title)}"' if title else ""
    if key is None:
        return f"<th scope=\"col\"{cls_attr}{t}>{label}</th>"
    aria = sort or "none"
    glyph = {"ascending": "arrow-up", "descending": "arrow-down", "none": "arrow-up-down"}[aria]
    order = "desc" if aria == "ascending" else "asc"
    return (f'<th scope="col"{cls_attr}{t} aria-sort="{aria}"><form method="get" class="fui-sort-form">'
            f'<input type="hidden" name="order" value="{order}" />'
            f'<button type="submit" class="fui-sort" name="sort" value="{e(key)}">{label}{icon(glyph, "12")}</button></form></th>')


def td(label, body, cls="", role=None):
    """A body cell; its content is one node, so a stacked card's label and
    value grid keeps the value together. `role` places it in the
    work-list's phone card."""
    c = f' class="{cls}"' if cls else ""
    r = f' data-role="{role}"' if role else ""
    if "fui-table-actions" not in cls and "fui-row-menu" not in cls and "fui-row-check" not in cls and not body.startswith("<time"):
        body = f"<span>{body}</span>"
    return f'<td data-label="{e(label)}"{c}{r}>{body}</td>'


def table(caption, head, rows, stack=True, scroll_label=None):
    cls = "fui-table-scroll fui-table-stack" if stack else "fui-table-scroll"
    body = "".join(rows)
    return (
        f'<div class="{cls}" role="region" aria-label="{e(scroll_label or caption)}" tabindex="0">'
        f'<table class="fui-table"><caption>{e(caption)}</caption><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div>'
    )


def pager(label, total, size, page=1, sizes=(10, 25, 50)):
    """The pager says one thing in its numbers and is absent when every row
    fits on one page (review item 12)."""
    pages = max(1, -(-total // size))
    if pages == 1:
        return ""
    first = (page - 1) * size + 1
    last = min(total, page * size)
    prev, next_ = page > 1, page < pages

    def step(key, glyph_, words, avail, first_=True):
        dis = "" if avail else " disabled"
        g = icon(glyph_, "14")
        inner = f'{g}<span class="fui-step-words">{words}</span>' if first_ else f'<span class="fui-step-words">{words}</span>{g}'
        unavailable = "" if avail else '<span class="fui-sr-only"> (unavailable)</span>'
        return f'<button type="button" class="fui-button fui-button-secondary fui-button-small" data-page="{key}"{dis}>{inner}{unavailable}</button>'

    steps = (
        step("first", "chevrons-left", "First", prev)
        + step("previous", "chevron-left", "Previous", prev)
        + f'<span class="fui-pager-position">Page {page} of {pages}</span>'
        + step("next", "chevron-right", "Next", next_, first_=False)
        + step("last", "chevrons-right", "Last", next_, first_=False)
    )
    sz = "".join(
        f'<button type="button" class="fui-pager-size" aria-pressed="{"true" if s == size else "false"}">{s}</button>'
        for s in sizes
    )
    return (
        f'<nav class="fui-pager" aria-label="{label}"><p role="status">{first}–{last} of {total}</p>'
        f'<div class="fui-pager-steps">{steps}</div>'
        f'<div class="fui-pager-sizes"><span class="fui-muted">Rows per page</span>{sz}</div></nav>'
    )


# ---- the gaps ----

def countdown(left, total=30):
    share = left / total
    r = 5.5
    circ = 2 * 3.14159 * r
    return (
        f'<span class="fui-identifier-countdown" role="status" aria-live="polite">'
        f'<svg viewBox="0 0 14 14" aria-hidden="true"><circle class="fui-identifier-countdown-track" cx="7" cy="7" r="{r}"/>'
        f'<circle class="fui-identifier-countdown-left" cx="7" cy="7" r="{r}" stroke-dasharray="{circ:.2f}" stroke-dashoffset="{circ * (1 - share):.2f}"/></svg>'
        f'{left} s left</span>'
    )


def identifier(domain, tail, revealed=None, left=None, total=30, static=False, reveal_id=None, dots=6,
               show_domain=True, state=None, copied=False):
    """A masked identifier: the domain tag (left out where the row already
    names the domain, review item 11), the mask ending in `tail`, and the
    reveal control. `revealed` draws the value with its countdown, Hide
    and Copy. `state`: 'busy' (the round trip) or 'expired'."""
    dom = tag(domain) if show_domain else ""
    name = f'<span class="fui-sr-only">{e(domain)} </span>' if not show_domain else ""
    if revealed is not None:
        done = f'<span class="fui-copy-done" role="status">{icon("check", "14")}Copied</span>' if copied else ""
        return (
            f'<span class="fui-identifier"><span class="fui-sr-only">{e(domain)} revealed, </span>{dom}'
            f'<span class="fui-identifier-shown"><span class="fui-identifier-value">{e(revealed)}</span>{countdown(left, total)}</span>'
            f'<span class="fui-identifier-tools">'
            + button("secondary", "Hide", "eye-off", small=True, attrs='aria-pressed="true"', cls="fui-identifier-reveal")
            + button("secondary", "Copy", "copy", small=True)
            + done
            + "</span></span>"
        )
    mask = (
        f'<span class="fui-identifier-mask" aria-label="masked, ends in {e(tail)}">'
        f'<span class="fui-identifier-dots" aria-hidden="true">{"•" * dots}</span>{e(tail)}</span>'
    )
    if static:
        return f'<span class="fui-identifier fui-identifier-static">{name}{dom}{mask}</span>'
    if state == "busy":
        reveal = busy_button("secondary", "Revealing", small=True)
    else:
        attrs = 'aria-pressed="false"'
        if reveal_id:
            attrs += f' aria-controls="{reveal_id}"'
        reveal = button("secondary", "Reveal", "eye", small=True, attrs=attrs, cls="fui-identifier-reveal")
    exp = '<span class="fui-identifier-expired" role="status">Hidden again after 30 s</span>' if state == "expired" else ""
    return f'<span class="fui-identifier">{name}{dom}{mask}{reveal}{exp}</span>'


MARKS = {
    "agree": ("check", "agrees", False),
    "differ": ("equal-not", "differs", True),
    "absent": ("minus", "absent", True),
}


def mark(kind):
    if kind is None:
        return ""
    glyph, word, visible = MARKS[kind]
    w = f"<span>{word}</span>" if visible else f'<span class="fui-sr-only">{word}</span>'
    return f'<span class="fui-compare-mark fui-compare-{kind}" title="{word}">{icon(glyph, "12")}{w}</span>'


def compare(caption, fields, leading, others, verdict, differ_first=True, note=None, shown=0):
    """fields: [(key, label)]; leading/others: {key, title, sub, values: {field: html|None}};
    verdict(field, record_index) -> agree|differ|absent|None."""
    order = list(fields)
    if differ_first:
        def rank(f):
            vs = [verdict(f[0], i) for i in range(len(others))]
            return 0 if "differ" in vs else (1 if "absent" in vs else (2 if any(v for v in vs) else 3))

        order.sort(key=rank)

    def head(rec, lead):
        sub = f'<span class="fui-compare-sub">{rec["sub"]}</span>' if rec.get("sub") else ""
        lead_badge = badge("accent", "leading") if lead else ""
        cls = ' class="fui-compare-leading"' if lead else ""
        return f'<th scope="col"{cls}><span class="fui-compare-title">{e(rec["title"])}{lead_badge}</span>{sub}</th>'

    def cell(rec, f, i, lead):
        v = rec["values"].get(f)
        shown_ = f'<span class="fui-compare-value">{v}</span>' if v is not None else '<span class="fui-compare-value fui-compare-none" title="absent">–</span>'
        m = "" if lead else mark(verdict(f, i))
        if v is None and m:
            shown_ = ""
        cls = ' class="fui-compare-leading"' if lead else ""
        return f'<td{cls} data-label="{e(rec["title"])}"><span class="fui-compare-cell">{shown_}{m}</span></td>'

    rows = ""
    for key, label in order:
        vs = [verdict(key, i) for i in range(len(others))]
        rcls = ' class="fui-compare-row-differ"' if "differ" in vs else ""
        rows += f'<tr{rcls}><th scope="row">{e(label)}</th>{cell(leading, key, -1, True)}'
        rows += "".join(cell(r, key, i, False) for i, r in enumerate(others)) + "</tr>"
    heads = head(leading, True) + "".join(head(r, False) for r in others)
    wide = (
        f'<div class="fui-table-scroll fui-compare-wide" role="region" aria-label="{e(caption)}" tabindex="0">'
        f'<table class="fui-compare"><caption>{e(caption)}</caption><thead><tr><th scope="col">Field</th>{heads}</tr></thead>'
        f"<tbody>{rows}</tbody></table></div>"
    )
    # The stacked form: a choice of which record to read, every record
    # offered; the other records are marked against the leading one.
    recs = [leading] + list(others)
    segs = "".join(
        f'<button type="button" class="fui-segment" aria-pressed="{"true" if i == shown + 1 else "false"}">{e(r.get("short", r["title"]))}</button>'
        for i, r in enumerate(recs)
    )
    shown_rec = others[shown]
    srows = ""
    for key, label in order:
        v = shown_rec["values"].get(key)
        lv = leading["values"].get(key)
        vd = verdict(key, shown)
        sv = v if v is not None else ""
        lead_line = f'<span class="fui-small fui-muted">{e(leading.get("letter", leading["title"]))}: {lv if lv is not None else "–"}</span>' if vd == "differ" else ""
        rcls = ' class="fui-compare-row-differ"' if vd == "differ" else ""
        srows += f'<div{rcls}><dt>{e(label)}</dt><dd><span class="fui-compare-cell"><span class="fui-compare-value">{sv}</span>{mark(vd)}</span>{lead_line}</dd></div>'
    stacked = (
        f'<div class="fui-compare-stacked"><div class="fui-compare-switch"><span class="fui-label" id="read-label">Record to read</span>'
        f'<div role="group" class="fui-segmented" aria-labelledby="read-label">{segs}</div>'
        f'<span class="fui-small fui-muted">Marked against {e(leading["title"])}, the leading record.</span></div>'
        f'<dl class="fui-compare-stacked-rows">{srows}</dl></div>'
    )
    n = f'<p class="fui-small">{note}</p>' if note else ""
    return n + wide + stacked


def score_bar_row(label, reading, weight, scale=6.5):
    """One contribution: the bar from the zero line at 40% of the width."""
    zero = 40.0
    span = min(abs(weight) / scale, 1.0) * (60.0 if weight >= 0 else 40.0)
    if weight > 0:
        rect = f'<rect class="fui-score-bar-add" x="{zero}" y="1" width="{span:.1f}" height="12" rx="2"/>'
        cls, text = "fui-score-add", f"+{weight:.1f}"
    elif weight < 0:
        rect = f'<rect class="fui-score-bar-remove" x="{zero - span:.1f}" y="1" width="{span:.1f}" height="12" rx="2"/>'
        cls, text = "fui-score-remove", f"−{abs(weight):.1f}"
    else:
        rect, cls, text = "", "fui-score-zero", "0"
    svg = (
        f'<svg class="fui-score-bar" viewBox="0 0 100 14" preserveAspectRatio="none" aria-hidden="true">'
        f'<rect class="fui-score-bar-ground" x="0" y="1" width="100" height="12" rx="2"/>{rect}'
        f'<line class="fui-score-bar-zero" x1="{zero}" x2="{zero}" y1="0" y2="14"/></svg>'
    )
    return (
        f'<div class="fui-score-row"><span class="fui-score-label">{e(label)}<span class="fui-score-reading">{e(reading)}</span></span>'
        f"{svg}<span class=\"fui-score-weight {cls}\">{text}</span></div>"
    )


THRESHOLDS = [("possible", 0.60), ("probable", 0.80), ("certain", 0.95)]


def score_scale(score, label, thresholds=THRESHOLDS, lo=0.5):
    """The scale from `lo` to 1 (no candidate scores under possible_at, so
    the track starts below it), each threshold labelled once at its mark
    on one row, and the score as the point, unlabelled: the total above
    it says the number once (review item 19)."""
    def x(v):
        return (v - lo) / (1 - lo) * 100

    bands = [("none", lo)] + list(thresholds) + [("end", 1.0)]
    rects = ""
    for (name, start), (_, end) in zip(bands, bands[1:]):
        rects += f'<rect class="fui-score-scale-band-{name}" x="{x(start):.1f}%" y="8" width="{x(end) - x(start):.1f}%" height="10"/>'
    marks = ""
    for name, at in thresholds:
        anchor = "end" if at > 0.93 else "middle"
        xx = f"{x(at):.1f}%" if anchor == "middle" else "100%"
        marks += (
            f'<line class="fui-score-mark" x1="{x(at):.1f}%" x2="{x(at):.1f}%" y1="4" y2="24"/>'
            f'<text class="fui-score-mark-label" x="{xx}" y="38" text-anchor="{anchor}">{name} {at:.2f}</text>'
        )
    tick = f'<text class="fui-score-scale-tick" x="0" y="38" text-anchor="start">{lo:.2f}</text>'
    point = f'<circle class="fui-score-point" cx="{x(score):.1f}%" cy="13" r="6"><title>Score {score:.2f}</title></circle>' if score is not None else ""
    return (
        f'<svg class="fui-score-scale" width="100%" height="44" role="img" aria-label="{e(label)}">'
        f'<rect class="fui-score-scale-band-none" x="0" y="8" width="100%" height="10" rx="3"/>{rects}{marks}{tick}{point}</svg>'
    )


def score_cell(score, grade_tone):
    pct = int(round(score * 100))
    return (
        f'<span class="fui-score-cell fui-tone-{grade_tone}"><svg class="fui-proportion" viewBox="0 0 100 6" preserveAspectRatio="none" '
        f'focusable="false" aria-hidden="true"><rect class="fui-proportion-ground" width="100" height="6" rx="3"></rect>'
        f'<rect class="fui-proportion-fill" width="{pct}" height="6" rx="3"></rect></svg><span class="fui-num">{score:.2f}</span></span>'
    )


def timeline(label, events):
    """events: dicts with at, kind, tone, actor, title, reason, body(html)."""
    items = ""
    for ev in events:
        actor = f'<span class="fui-timeline-meta">{e(ev["actor"])}</span>' if ev.get("actor") else ""
        body = ""
        if ev.get("reason") or ev.get("body"):
            reason = f'<p class="fui-timeline-reason">{e(ev["reason"])}</p>' if ev.get("reason") else ""
            body = f'<div class="fui-timeline-body">{reason}{ev.get("body", "")}</div>'
        items += (
            f'<li class="fui-timeline-event fui-tone-{ev["tone"]}"><span class="fui-timeline-dot" aria-hidden="true"></span>'
            f'<div><div class="fui-timeline-head">{badge(ev["tone"], ev["kind"])}<span class="fui-timeline-title">{ev["title"]}</span>'
            f'<span class="fui-timeline-meta">{when(ev["at"])}</span>{actor}</div>{body}</div></li>'
        )
    return f'<ol class="fui-timeline" aria-label="{e(label)}">{items}</ol>'


def shortcut_legend(groups, id_="shortcuts"):
    gs = ""
    for title, items in groups:
        rows = ""
        for keys, does in items:
            caps = '<span class="fui-muted">or</span>'.join(
                "".join(kbd(k) for k in alt.split(" ")) for alt in keys.split(" | ")
            )
            rows += f"<dt>{caps}</dt><dd>{e(does)}</dd>"
        gs += f'<section class="fui-shortcuts-group"><h3>{e(title)}</h3><dl class="fui-shortcuts-list">{rows}</dl></section>'
    return (
        f'<div id="{id_}" role="dialog" aria-labelledby="{id_}-title" class="fui-dialog fui-dialog-wide fui-modal mk-open" open>'
        f'<h2 id="{id_}-title" class="fui-dialog-title">{icon("keyboard", "18")} Keyboard shortcuts</h2>'
        f'<p class="fui-muted">A shortcut acts only when no field has the focus. Every shortcut has a control on the screen that does the same.</p>'
        f'<div class="fui-modal-body"><div class="fui-shortcuts">{gs}</div></div>'
        f'<div class="fui-dialog-actions">{button("secondary", "Close", "x", keys="Esc")}</div></div>'
    )


def shortcut_hint(items):
    """The one-line foot of a listing; " | " separates alternative keys."""
    def caps(keys):
        return '<span class="fui-muted">or</span>'.join(
            "".join(kbd(k) for k in alt.split(" ")) for alt in keys.split(" | "))
    return '<p class="fui-shortcut-hint" aria-hidden="true">' + "".join(
        f"<span>{caps(keys)} {e(does)}</span>" for keys, does in items
    ) + "</p>"


# ---- the docked decision bar (FerroUI #145) ----

def dock(field_html, actions_html, message=""):
    m = message
    return (
        f'<div class="fui-dock" role="region" aria-label="Decision">{m}'
        f'<div class="fui-dock-row">{field_html}<div class="fui-dock-actions">{actions_html}</div></div></div>'
    )


def dock_field(id_, label, value="", placeholder="", keys=None, error=None, help_=None, disabled=False):
    k = kbd(keys) if keys else ""
    inv = ' aria-invalid="true" aria-describedby="' + id_ + '-error"' if error else ""
    dis = " disabled" if disabled else ""
    ks = f' aria-keyshortcuts="{aria_key(keys)}"' if keys else ""
    er = f'<p id="{id_}-error" class="fui-error">{icon("circle-alert", "14")}{error}</p>' if error else ""
    h = f'<p class="fui-help">{help_}</p>' if help_ and not error else ""
    return (
        f'<div class="fui-dock-field"><label for="{id_}" class="fui-label">{label}{k}</label>'
        f'{input_(id_, value, placeholder, extra=inv + dis + ks)}{er}{h}</div>'
    )


def stepper(position, prev_href="#", next_href="#", prev_ok=True, next_ok=True):
    """Previous and next through a list, in the page header (review item 2)."""
    def step(label, glyph, href, ok, keys, end):
        g = icon(glyph, "16")
        inner = f'<span class="fui-step-words">{label}</span>{g}' if end else f'{g}<span class="fui-step-words">{label}</span>'
        dis = "" if ok else ' aria-disabled="true"'
        if not ok or href == "#":
            return f'<span class="fui-button fui-button-secondary fui-button-small" aria-disabled="true" aria-label="{label} candidate">{inner}</span>'
        return f'<a href="{href}" class="fui-button fui-button-secondary fui-button-small" aria-keyshortcuts="{aria_key(keys)}" aria-label="{label} candidate" title="{label} candidate ({KEY_WORDS[keys]})"{dis}>{inner}</a>'
    return (
        f'<nav class="fui-stepper" aria-label="Candidates">{step("Previous", "arrow-left", prev_href, prev_ok, "left", False)}'
        f'<span class="fui-stepper-position" role="status">{position}</span>{step("Next", "arrow-right", next_href, next_ok, "right", True)}</nav>'
    )


# ---- the shell ----

NAV = [
    ("work", "Work", [
        ("dashboard", "Dashboard", "layout-dashboard", "dashboard.html", "queue-count-none"),
        ("queue", "Review queue", "list-checks", "queue.html", "queue-count"),
    ]),
    ("patients", "Patients", [
        ("search", "Search", "user-round-search", "search.html", None),
    ]),
    ("registry", "Registry", [
        ("sources", "Sources", "radio-tower", "sources.html", None),
        ("domains", "Identifier domains", "tags", "domains.html", None),
        ("pseudonym-domains", "Pseudonymisation domains", "shield-check", "pseudonym-domains.html", None),
    ]),
    ("trail", "Trail", [
        ("audit", "Audit trail", "file-clock", "audit.html", None),
    ]),
    ("console", "Console", [
        ("configuration", "Configuration", "file-text", "configuration.html", None),
        ("about", "About", "info", "about.html", None),
    ]),
]


def nav_count(queue):
    """The queue's depth: a number, nothing when the queue is empty, a
    muted dash when it is not known (review item 9)."""
    if queue is None:
        return '<span class="fui-nav-count fui-nav-count-unknown" title="Not known: the server did not answer">–</span>'
    if queue == 0:
        return ""
    return f'<span class="fui-nav-count" aria-label="{queue} waiting">{queue}</span>'


def sidebar(current, queue=38, rail=False, tooltip=None, drawer=False):
    groups = ""
    for gid, title, entries in NAV:
        items = ""
        for key, label, glyph, href, count in entries:
            cur = ' aria-current="page"' if key == current else ""
            c = nav_count(queue) if count == "queue-count" else ""
            tip = f'<span class="fui-tooltip" role="tooltip">{e(label)}</span>' if rail and tooltip == key else ""
            t = f' title="{e(label)}"' if rail else ""
            items += f'<li><a href="{href}" class="fui-nav-entry"{cur}{t}>{icon(glyph, "16")}<span class="fui-nav-word">{e(label)}</span>{c}{tip}</a></li>'
        groups += f'<p id="nav-{gid}" class="fui-nav-group">{title}</p><ul class="fui-nav-list" aria-labelledby="nav-{gid}">{items}</ul>'
    collapse = (
        f'<div class="fui-sidebar-foot fui-wide-only">{button("ghost", "<span class=\"fui-nav-word\">Expand</span>" if rail else "Collapse", "panel-left-open" if rail else "panel-left-close", small=True, attrs="aria-expanded=\"" + ("false" if rail else "true") + "\" aria-controls=\"sidebar\"")}</div>'
    )
    cls = "fui-sidebar" + (" fui-sidebar-rail" if rail else "") + (" mk-drawer-open" if drawer else "")
    return (
        f'<nav id="sidebar" popover="auto" aria-label="Console" class="{cls}">'
        '<button type="button" class="fui-button fui-button-ghost fui-drawer-only" popovertarget="sidebar" popovertargetaction="hide">'
        + icon("x", "16")
        + "Close</button>"
        + groups
        + collapse
        + "</nav>"
    )


def brand_mark():
    return (
        '<span class="fui-brand-mark" aria-hidden="true">'
        '<img class="fui-brand-light" src="ferropix-icon.svg" alt="" width="24" height="24" />'
        '<img class="fui-brand-dark" src="ferropix-icon-dark.svg" alt="" width="24" height="24" /></span>'
    )


def lockup(link=True):
    inner = f'{brand_mark()}<span class="fui-lockup-name">FerroPIX <span class="fui-lockup-accent">Console</span></span>'
    if not link:
        return f'<span class="fui-lockup">{inner}</span>'
    return f'<a href="dashboard.html" class="fui-lockup">{inner}</a>'


def search_control(value="", state=None, word="Find", placeholder="Identifier, name or birth date", id_="find", results=None):
    """The topbar search as FerroUI #144 draws it: one joined control with
    the leading icon, the field, the quiet `/` hint and the submit on the
    field's end edge. `state` is a mockup-only mimic: focus, hover, busy,
    disabled. `results` is an open answer list under the control."""
    cls = "fui-search" + (f" mk-{state}" if state in ("focus", "hover") else "")
    if state == "focus-filled":
        cls += " mk-focus"
    busy = ' aria-busy="true"' if state == "busy" else ""
    disabled = ' disabled title="Search waits while the index is rebuilding"' if state == "disabled" else ""
    submit_off = " disabled" if state in ("disabled", "busy") else ""
    hint = "" if state == "disabled" else '<kbd class="fui-kbd fui-search-hint" aria-hidden="true">/</kbd>'
    reason = '<span class="fui-sr-only" id="find-why">Search waits while the index is rebuilding.</span>' if state == "disabled" else ""
    glyph = (
        '<span class="fui-spinner-ring" aria-hidden="true"></span>' if state == "busy" else icon("search", "16")
    )
    expanded = ' aria-expanded="true" aria-controls="find-results"' if results else ""
    res = results or ""
    return (
        f'<form role="search" method="post" action="#" class="{cls}"{busy}>'
        f'<label for="{id_}" class="fui-sr-only">Find a patient</label>'
        f'<div class="fui-search-field"><span class="fui-search-glyph">{glyph}</span>'
        f'<input id="{id_}" name="q" type="search" class="fui-input" placeholder="{e(placeholder)}" value="{e(value)}" '
        f'autocomplete="off" spellcheck="false" aria-keyshortcuts="/"{expanded}{disabled} />'
        f'{hint}</div>'
        f'<button type="submit" class="fui-search-submit" aria-label="{e(word)}"{submit_off}>{icon("search", "16")}<span class="fui-search-word">{e(word)}</span></button>'
        f"{reason}{res}</form>"
    )


def operator_menu(open_=False, focus=None, theme="light"):
    items = [
        '<div class="fui-menu-header"><strong>Demo Steward</strong><span>Steward and administrator · FerroHEALTH ID</span></div>',
        MENU_SEP,
        menu_item("Your preferences", "settings", href="preferences.html"),
        '<div class="fui-menu-label" id="theme-label">Theme</div>',
        '<div role="group" aria-labelledby="theme-label" class="fui-segmented">'
        + "".join(
            f'<button type="button" role="menuitemradio" class="fui-segment" aria-checked="{"true" if v == theme else "false"}">{icon(g, "14")}{w}</button>'
            for v, w, g in (("light", "Light", "sun"), ("dark", "Dark", "moon"), ("system", "System", "monitor"))
        )
        + "</div>",
        menu_item("Keyboard shortcuts", "keyboard", keys="?", href="shortcuts.html"),
        MENU_SEP,
        menu_item("Sign out", "log-out", href="sign-in-signed-out.html", focus=(focus == "signout")),
    ]
    return menu(items, label="Demo Steward", sheet=True) if open_ else ""


def status_pill(tone, word, detail):
    """The top bar's status pill (F12): its shape and its words, which stay its
    accessible name below 64 rem; the tooltip holds the words and the detail."""
    tip = f"{word} · {detail}" if detail else word
    return f'<span class="fui-pill fui-tone-{tone} fui-topbar-status" title="{e(tip)}"><span class="fui-roomy-text">{e(word)}</span></span>'


def topbar(status_tone="ok", status_word="Server up", search_value="", search_state=None, results=None,
           menu_open=False, menu_theme="light"):
    search = search_control(search_value, search_state, results=results)
    operator = (
        '<span class="fui-anchor">'
        f'<button type="button" class="fui-button fui-button-ghost fui-operator" aria-haspopup="menu" aria-expanded="{"true" if menu_open else "false"}" aria-controls="operator-menu">'
        f'{icon("circle-user", "16")}<span class="fui-operator-name">Demo Steward</span>{icon("chevron-down", "14")}</button>'
        + operator_menu(menu_open, theme=menu_theme)
        + "</span>"
    )
    # The theme lives in the user menu and Your preferences; the top bar
    # carries no toggle (owner, family-wide).
    end = f'{status_pill(status_tone, status_word, "")}{operator}'
    drawer = f'<button type="button" class="fui-icon-button fui-drawer-only" popovertarget="sidebar" aria-controls="sidebar" aria-label="Menu">{icon("menu", "18")}</button>'
    return (
        f'<header class="fui-topbar"><div class="fui-topbar-row">{drawer}{lockup()}<div class="fui-topbar-middle">{search}</div>'
        f'<div class="fui-topbar-end">{end}</div></div></header>'
    )


def footer():
    return '<footer class="fui-footer"><p>FerroPIX Console v0.1.0 · Cadasto B.V. · <a href="about.html">About</a></p></footer>'


def head(title, density="comfortable"):
    return (
        f'<!doctype html><html lang="en" data-density="{density}"><head><meta charset="utf-8" />'
        '<meta name="viewport" content="width=device-width, initial-scale=1" />'
        f"<title>{e(title)} · FerroPIX Console</title>"
        '<link rel="icon" href="favicon.svg" type="image/svg+xml" />'
        '<link rel="stylesheet" href="ferrohealth-ui.css" /><link rel="stylesheet" href="ferropix.css" /><link rel="stylesheet" href="mockup.css" />'
        "</head>"
    )


def shell(title, current, content, density="comfortable", status=("ok", "Server up"), overlay=None, search_value="",
          queue=38, dock_html="", toast_html="", rail=False, rail_tip=None, drawer=False, search_state=None,
          results=None, menu_open=False, menu_theme="light", script=""):
    """A signed-in page; `overlay` is an open dialog drawn over the page,
    `dock_html` the docked decision bar at the foot of the main area."""
    ov = f'<div class="mk-scrim" aria-hidden="true"></div>{overlay}' if overlay else ""
    cls = "fui-page" + (" mk-page-overlaid" if overlay or drawer else "")
    dr = '<div class="mk-scrim mk-scrim-drawer" aria-hidden="true"></div>' if drawer else ""
    sc = f"<script>{script}</script>" if script else ""
    return (
        head(title, density)
        + f'<body><div class="{cls}"><a href="#main" class="fui-skip">Skip to the page</a>'
        + topbar(status[0], status[1], search_value, search_state, results, menu_open, menu_theme)
        + f'<div class="fui-body">{sidebar(current, queue, rail, rail_tip, drawer)}{dr}<main id="main" class="fui-main"><div class="fui-column">{content}</div>{dock_html}</main></div>'
        + footer()
        + toast_html
        + ov
        + f"</div>{sc}</body></html>"
    )


def centred(title, content):
    return (
        head(title)
        + f'<body><div class="fui-page"><main id="main" class="fui-centred">{content}</main>{footer()}</div></body></html>'
    )


def action_group(primary, items, open_=False, label="More actions", split_items=None, split_open=False, hover=None):
    """F14 action_group: at most one primary action plus one More menu button;
    destructive items only in the menu, last, after a separator. With
    `split_items`, the primary is a split button whose menu holds variants
    of the same action. Below 40 rem More opens as the bottom sheet."""
    if split_items is not None:
        sm = menu(split_items, label=f"{label}: variants", sheet=True) if split_open else ""
        primary = (f'<span class="fui-split-button">{primary}<span class="fui-anchor"><button type="button" class="fui-button fui-button-secondary fui-split-toggle" aria-haspopup="menu" '
                   f'aria-expanded="{"true" if split_open else "false"}" aria-label="{e(label)}: variants">{icon("chevron-down", "16")}</button>{sm}</span></span>')
    m = menu(items, label=label, sheet=True) if open_ else ""
    more = (f'<span class="fui-anchor"><button type="button" class="fui-button fui-button-secondary fui-more" aria-haspopup="menu" aria-expanded="{"true" if open_ else "false"}" aria-label="{e(label)}">'
            f'More{icon("chevron-down", "16")}</button>{m}</span>') if items else ""
    return f'<div class="fui-action-group">{primary}{more}</div>'


# ---- T3. The settings and reference page: title, binder tabs, one tab panel ----

def settings_tabs(label, items, current, panel_html, drafts=(), many=False, focus=None):
    """items: (key, words, href). Each tab is its own route; the tab panel shows
    only that section. drafts: tab keys that hold a pending draft. many: more
    tabs than a phone row shows, so at 390 px a select replaces the row."""
    links = ""
    for key, words, href in items:
        cur = ' aria-current="page"' if key == current else ""
        badge = '<span class="fui-tab-draft" title="Holds changes in a draft">draft</span>' if key in drafts else ""
        cls = "fui-tab" + (" mk-focus" if key == focus else "")
        links += f'<a href="{href}" class="{cls}"{cur} data-tab="{key}">{e(words)}{badge}</a>'
    nav = f'<nav class="fui-tabs" aria-label="{e(label)}" data-keys="arrows">{links}</nav>'
    sel = ""
    if many:
        opts = "".join(f'<option value="{href}"{" selected" if key == current else ""}>{e(words)}{" (draft)" if key in drafts else ""}</option>' for key, words, href in items)
        sel = f'<div class="fui-tabs-select"><label class="fui-sr-only" for="tabs-sel">{e(label)}</label><select id="tabs-sel" class="fui-select">{opts}</select></div>'
    cls = "fui-tabbed fui-settings-tabbed" + (" fui-tabbed-many" if many else "")
    return f'<div class="{cls}">{sel}{nav}<div class="fui-tab-panel">{panel_html}</div></div>'


def settings_panel(rows_html, intro="", actions="", top=""):
    """One tab's section: an optional one-line intro with the tab's actions,
    then the setting rows across the panel's width."""
    head = ""
    if intro or actions:
        head = f'<div class="fui-settings-panel-head"><p class="fui-settings-panel-intro">{intro}</p><div class="fui-actions">{actions}</div></div>'
    return f'<div class="fui-settings-panel">{head}{top}<div class="fui-setting-rows">{rows_html}</div></div>'


DOT_WORD = {"ok": "ok", "warn": "attention", "bad": "failing", "neutral": "not known", "muted": "inactive"}


def health_dot(tone, label=None):
    word = DOT_WORD.get(tone, tone)
    return f'<span class="fui-dot fui-tone-{tone if tone != "muted" else "neutral"}{" is-muted" if tone == "muted" else ""}" role="img" aria-label="{e(label or word)}" title="{e(label or word)}"></span>'
