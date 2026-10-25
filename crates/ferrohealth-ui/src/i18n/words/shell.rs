// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the shell: the chrome, the user menu, the
//! sidebar, the search box and the breadcrumb.

use crate::i18n::Text;

texts! {
    /// The accessible name of the breadcrumb trail.
    Breadcrumb = "breadcrumb",
    /// A button that closes the drawer or a side panel.
    Close = "close",
    /// The search box's button.
    Find = "find",
    /// The footer's link to the About page.
    FooterAbout = "footer-about",
    /// The user menu's link to the keyboard shortcuts.
    KeyboardShortcuts = "keyboard-shortcuts",
    /// The word between the alternative keys of one shortcut.
    KeysOr = "keys-or",
    /// A definition whose value is absent.
    NoneValue = "none-value",
    /// The name of the button that opens the navigation drawer.
    OpenMenu = "open-menu",
    /// The search panel's option that submits to the find page.
    SearchAll = "search-all",
    /// The accessible name of the list of hits under the search box.
    SearchAnswer = "search-answer",
    /// What a screen reader hears once the search panel's hits arrive.
    SearchCounted = "search-counted",
    /// The search panel's row when the console answered with an error.
    SearchFailed = "search-failed",
    /// The search panel's and the find page's words when nothing matches.
    SearchNothing = "search-nothing",
    /// The search panel's row while the console is asked.
    SearchSearching = "search-searching",
    /// The search panel's row when the console could not be reached.
    SearchUnreachable = "search-unreachable",
    /// The search panel's row when the console's answer could not be read.
    SearchUnreadable = "search-unreadable",
    /// The sentence under the legend's heading.
    ShortcutsIntro = "shortcuts-intro",
    /// The sidebar's button that folds it to the rail of glyphs.
    SidebarCollapse = "sidebar-collapse",
    /// The rail's button that unfolds it to the full sidebar.
    SidebarExpand = "sidebar-expand",
    /// The user menu's sign-out item.
    SignOut = "sign-out",
    /// The link that skips the chrome to the page.
    SkipToPage = "skip-to-page",
    /// The user menu's link to the preferences page.
    YourPreferences = "your-preferences",
}
