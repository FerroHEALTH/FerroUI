<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->
# FerroUI console components

The shared design library for the FerroHEALTH operator consoles. It came out of the FerroPIX review rounds, the FerroFED redesign (FerroHEALTH/FerroFED#1218), the FerroBRIDGE redesign, the FerroTERM redesign and the FerroEHR viewer redesign. FerroPIX, FerroFED, FerroBRIDGE, FerroTERM and FerroEHR draw every page from it, with no console-local component.

## What is in this folder

| Path | What it is |
|---|---|
| `css/ferrohealth-ui.css` | The FerroUI base stylesheet, unchanged. |
| `css/ferroui-console.css` | The console component layer: R1 to R29 from FerroPIX, F1 to F14 and the templates T1 to T3 from FerroFED, B1 to B10 and the planned and proposed mark from FerroBRIDGE, M1 to M6 and the editor and answer parts from FerroTERM, E1 to E8 from FerroEHR, and the family rules below as CSS. Each console copies it unchanged as its `mockup.css`. Rules prefixed `mk-` are capture scaffolding for the mockups (forced hover and focus states, open menus, frames on state sheets). They are not part of the component API. |
| `themes/ferrofed.css`, `themes/ferropix.css`, `themes/ferrobridge.css`, `themes/ferroterm.css`, `themes/ferroehr.css` | One brand file per console: the accent of each theme, plus `--preview-accent-light` and `--preview-accent-dark`, which the theme previews in Your preferences draw. |
| `templates/` | The reference pages: T1 list (three states), T2 detail, T3 settings (resting and in edit), and About. They use example content and are drawn in the FerroFED theme. |
| `renders/` | The templates at 1440 px light, 1440 px dark and 390 px light. |
| `tools/` | The helpers the templates use (`common.py`, `tpl.py`, `parts.py`, `pg_about.py`), plus the generator (`templates.py`) and the checks every console runs: `render.py`, `overflow.py`, `hover.py`, `contrast.py`, `selects.py`, `popups.py`, `forms.py`. `common.py`'s `refusal()` lets the request path break after each slash, never inside a segment; on a phone each issue's text (`fui-issue-text`) takes the whole row under its badge and head, so a whole id in it fits unbroken. |

To regenerate and check:

```sh
python3 tools/templates.py templates
python3 tools/render.py templates renders
python3 tools/overflow.py templates
python3 tools/hover.py templates
python3 tools/contrast.py templates
```

## Family rules

These hold on every page of every console. A page that breaks one is a defect.

1. **One home per thing.** Each object, setting and document has exactly one page where it is shown in full and changed. Other pages link to it, or show a read-only mirror marked as such with a link. Deployment-wide settings that have no object of their own live in Configuration.
2. **No second sidebar.** The main sidebar is the only vertical navigation. Navigation inside a page is binder tabs across the top. A page with one or two sections has no tabs and no sub-navigation.
3. **No hover layout shift.** Hovering or focusing a control changes paint only: tint, colour, outline, opacity. Nothing moves or resizes. Hints that appear on hover reserve no space, or use `visibility`. `tools/hover.py` checks every page at 1440 and 390 px.
4. **No link underlines in tables.** A name in a table is in the ink without an underline. The whole row is the target (R9 extension). A link in a list's group header (a file over its diagnostics in B3, a table's group row) is a name too: ink, no underline, and the pointer tints it in the accent. Underlines stay for links in running text.
5. **The theme is set only in the user menu** (and in Your preferences). The top bar carries no theme toggle.
6. **No key hints on phones.** Key caps and shortcut hints are hidden below 40 rem (R1). The shortcut legend keeps its keys at every width.
7. **Destructive actions come last in menus,** after a separator, in the danger tone, with an ellipsis when a confirmation follows. They never sit as a button beside the primary action (F13, F14).
8. **The simplest design that does the job.** Use a template part only where it earns its place:
   - a list of ten items or fewer has no summary strip and no filter menus;
   - a page with three facts has no tabs;
   - a small fact has no side panel or modal;
   - text does not repeat the obvious.
9. **Personal preferences are cookies. The deployment's settings are Configuration.**
   - Each preference is its own cookie: theme, density, language, first day of the week, reduce motion, shortcut hints, and in FerroPIX reveal duration and hide on leave.
   - Every preference cookie is `SameSite=Lax`, `Secure`, lives one year and holds no patient data.
   - The server reads the cookies when it renders, so the first paint is right and no theme flashes.
   - The consoles keep no user store. Preferences never go to `localStorage`.
   - A bounded preference stays bounded by the server: the server caps every reveal at its maximum, whatever the cookie says.
   - The page says "Only for you, in this browser". Reset to defaults clears the cookies.
10. **Comfortable is the default density.** Compact is a preference. It tightens rows and keeps the column header and card header padding.
11. **Times are relative in lists** ("13 s ago") with the full timestamp in the tooltip, and absolute in records and revisions.
12. **States use a shape, a colour and a word together, never colour alone** (F12); the word starts with a capital ("Reachable", "Degraded", "Excluded"):
    - reachable: a filled dot;
    - degraded: a diamond;
    - unreachable: a cross (✕), the same mark the topology draws at the end of an unreachable edge. Never a triangle: the triangle is the warning glyph of alerts;
    - not probed: a hollow ring;
    - excluded: a muted row.

13. **Console configuration mode exists only for a product with its own configuration store.** Such a product offers one mode per deployment under a key of its own choosing (FerroFED: `config.ui = "managed" | "off"`), the core always in the file, the mode badge and the "How configuration works" dialog; the library names no key. A product without a store (FerroBRIDGE, FerroTERM, FerroEHR, whose PostgreSQL holds clinical data and never configuration) runs in file mode only: Configuration and every Settings tab are read-only views of its TOML, with one quiet line, "Set in <file> · read at start · How configuration works", and no per-row Next start markers.

## Interaction

Every component follows one interaction contract, so the consoles behave alike and every page keeps working when its WebAssembly has not loaded (FerroUI#196):

1. **Native elements first:** `<dialog>`, the `popover` attribute, `<details>`, plain links, and GET and POST forms.
2. **Behaviour in Rust compiled to WebAssembly:** menus and sheets (F13), keyboard tabs (R3), the date picker (R25), toasts with actions (R17), the reveal countdown (R13), live polling, topology pan and zoom (F9), work-list focus (R9), and the YAML and AQL editor (B2).
3. **Every page works without the WebAssembly:** complete server-rendered HTML, and a link or a form behind every important action.
4. **No hand-written JavaScript and no npm:** only the `wasm-bindgen` glue and the island loader Leptos writes. The strict CSP stays.

The consoles render in **Leptos islands mode** (the Leptos book, "Guide: Islands"): the server draws every page whole, and the browser hydrates only the interactive parts, each a small island of its own. Navigation is plain links and forms, with no client router. An island's props, its words among them, are resolved on the server and are page content, so no island carries clinical data the page does not already show. Toasts come from one region island in the shell; with no WebAssembly the server draws the outcome of a form post in it.

Hydrating the whole page was measured against islands on the gallery and rejected: 685903 bytes of brotli-compressed WebAssembly against 427449, and on a slowed phone profile 315 ms against 277 ms to interactive on the forms page. The gallery's budget is 491520 bytes, since every island takes its words as props and the browser carries no catalog; each console keeps its own (FerroBRIDGE: 458752). The islands router is revisited when Leptos ships it stable.

## Page templates

Every page uses one of three templates, or the simple About page. Each template's parts are optional, and each part below says when it appears. The dashboard (an overview) and a query console (a workbench) are the only exceptions.

### T1 List page

`templates/list.html`, `list-many.html`, `list-tabs.html`

| Part | When |
|---|---|
| Title, intro, one primary action | Always; the action only when items can be created here. |
| Summary strip (`fui-summary-chips`) | Only past 10 items, or when the chips filter the list. Each chip is a filter with a count. |
| Toolbar (`fui-list-toolbar`): find field, filter menus | Only past 10 items, or when the list is unbounded (history, audit, routes). |
| Active filter chips with Clear filters | Only while a filter is set. |
| The card (`fui-list-card`) | Always. Without tabs it opens with a header row: title, count, actions. |
| Binder tabs over the card | Only when the list has two or more real views (Organisations and All nodes). The tabs carry the title and count, and the card is their panel. |
| Pager in the card foot | Only when there are more rows than one page holds. |
| Row menu (⋯) | Only when a row has actions beyond opening it. |

**Documented exception: a stated multi-key sort.** A worklist whose order is several keys at once (the FerroPIX review queue: grade, then score, then the longest wait) states that order as one line in the card's header row, beside the count. One sort arrow on one column cannot say it. The column that leads the order still carries the arrow.

Fixed vertical rhythm: 24 px under the title and intro (the page header's margin), then 16 px between the summary strip, toolbar, filter chips and card.

### T2 Detail page

`templates/detail.html`

| Part | When |
|---|---|
| Breadcrumbs | Always under a list. On a phone they become one back link (R6). |
| An editor's title | The trail ends with the resource it edits and the title says Edit, or "New <thing>" for a new object ("New EHR", "New value set": the family form, since "New" alone does not say what is made) ("Value sets / Synthetic respiratory findings", title Edit, `fui-crumbs is-parent`); on a phone that resource is the back link. The intro is one sentence ("Version 2, opened 09:38."); the save rules belong to the states that need them. |
| Title with its state badge and a one-line subtitle | Always. |
| Action group (F14) | One primary action plus More. Destructive items are last in More. Leave out More when nothing else is possible. |
| Binder tabs over one panel | When the object has more than one section (Overview, its children, Requests, Audit, Settings). Each tab is a route. |
| A tab this object cannot have | Left out, never disabled: a resource with no write path has no History tab and no Edit, and its first tab says why once ("Served from a built index, with no write path"). |
| Variable primary action | The primary is what this reader can do first (Edit where the resource is writable and the reader may write; otherwise Browse, Expand or Translate). More always holds the others. |

### T3 Settings and reference page

`templates/settings.html`, `settings-edit.html`

| Part | When |
|---|---|
| Title, mode badge, action group | Always; the badge only where a deployment has modes. |
| Revision line (`fui-revision-compact`) | Only where the settings are revisioned. One line, "Revision 7 · applied 2026-10-14 by Demo Steward · in step", with the reason in the pill's tooltip and in History. An event (applied, a restart waiting, a conflict, a replica refusing) replaces it with a full revision strip in its tone until it is resolved. A draft is never a strip: the tab's draft badge and the panel intro say it. |
| Binder tabs, one section each | When the page has three or more sections. Each tab is a route (`/configuration/consent`) that survives a reload. ← and → move along the tablist. |
| Tab select at 390 px (`fui-tabbed-many`) | When there are more tabs than a phone row shows (Configuration's seven). Up to five tabs keep the scrolling row with edge fades. |
| Panel head: one-line intro and that tab's actions | One sentence on what the tab governs, then how a change takes effect ("Changes apply at once unless a row says Next start."). Edit is per tab. In edit the intro becomes "A draft of this tab, based on revision 7: 2 changes." A review shows the diff inside the same tab, still marked draft. |
| Setting rows | Label column of at most 80 characters, then the value from 40 % of the panel. A "Next start" marker sits after the value, small. The TOML key is a copyable hint on the label line, shown on hover and focus only, never on phones. |
| Draft badge on a tab (`fui-tab-draft`) | While that tab holds a draft. |
| Docked bar (R4) | In edit: it spans the content area and holds the reason, Discard and Review changes. |

Pages: Configuration in both consoles, Self-description in FerroFED, and Your preferences in both.

### About (its own simple page)

`templates/about.html`

- Two cards of facts: Versions, and Manufacturer (maker, licence, support, documentation).
- Below them, optionally, one full-width card with the product's raw self-description:
  - FerroFED shows its OPTIONS document; FerroPIX shows its CapabilityStatement;
  - the JSON uses the editor's token colours;
  - Copy and Download sit in the card header;
  - the box is capped at 24 rem with its own scroll and scrolls sideways on a phone.
- No tabs, no sub-navigation, no side panel.

## The table

Measured on the FerroPIX identifier domains table at 1440 px. Every console's tables use these values.

| Part | Value |
|---|---|
| Card | The card edge is the only frame. A table inside a card or a tab panel has no box of its own: no border, radius or shadow. Its last row has no bottom line. |
| Card header row | 12 px 16 px padding, a hairline under it; the title at weight 600, the count muted after a middle dot. |
| Column header | 35 px high, cells 8 px 12 px, 13 px text at weight 600 in the muted ink on `--surface-sunken`. The sort arrow is inline after the label. A sort is a `<button>` in a `GET` form that carries the sort (`sort`, `order`) and the rest of the listing's query, so a page sorts with no script; never a link (owner, 2026-10-22). A listing that keeps its state out of the URL posts the same fields. |
| Body rows | 43 px high, cells 8 px 12 px, hairline separators only, no vertical borders. |
| Numbers | Right-aligned, tabular figures. |
| Row menu column | Narrow, right-aligned, the ⋯ icon button. |
| Excluded row | Muted (`is-excluded`). The State column says so; the name carries no second tag. |
| Phone (below 40 rem) | Rows stack into cards (`fui-table-stack`). A cell's quiet note takes its own line, and ids and changed values never break inside themselves. A title cell with a second line (`fui-row-sub`) wraps across the card, and an empty cell takes no line. Inside a card or a panel the stacked rows are hairline-separated lines that keep the panel's inset, never cards in a card; a group row is a full-width band; the row ⋯ sits on the row's first line; a row's checkbox sits on its first line, at its start; a count cell carries its noun (`data-unit`). A column marked `data-role="wide"` leaves the phone row, so it holds only what a phone reader finds again on the row's own page: never a selection, a state or the row's name. |
| Group row | `tr.fui-group-row` in the sunken band; its links follow rule 4. |
| Find field | The toolbar's find field is a form the server answers: a `GET` with the words in the query, or a `POST` when the words can be clinical. Enter submits it, and it filters the whole listing, never only the rows already in the page. |
| Pager | In the card foot. A listing that keeps its position out of the URL pages through a `POST` form: each step and each size is a submit button carrying `page` and `size`, so no position reaches a URL or the history. |
| Empty list | The title-row primary stays. The empty state offers the same action as a quiet text link ("Add the first organisation"), never a second primary button. |

## Components

### From FerroPIX (R1 to R29)

| | |
|---|---|
| R1 | Responsive key hints |
| R2 | Tag |
| R3 | Binder tabs joined to their panel A tab row that scrolls starts with its current tab in view (`scroll-initial-target`). A tab marked `data-tab="keyboard"` (the family hook for Keyboard in Your preferences) is hidden below 40 rem: phones have no keyboard items. Each tab is a link to its own route, so the row works with no WebAssembly; the phone's tab select takes the row's place only once it works. |
| R4 | Docked decision bar |
| R5 | Previous and next in the page header |
| R6 | Breadcrumb as a back link on a phone |
| R7 | Page actions folding into one menu. F14 replaces it. |
| R8 | Anchored menu and row menu. F13 replaces it. |
| R9 | The worklist. Its extension: names in the ink, no underline, the row is the target. |
| R10 | Field with an inner control |
| R11 | Form grid. Its extension: fields share their label line (`align-items: start`), so help or an error under one field never lifts its neighbours; a field holding a URL, a path or another long token takes the whole row (`fui-span-row`). |
| R12 | Honest navigation count. The number is hidden from assistive technology and the whole phrase ("2 errors in the running load") is read in its place; nothing shows at zero; an unknown count is a muted dash with the reason in its tooltip. In the rail the count rides on the glyph. |
| R13 | Masked identifier revealed |
| R14 | Filter chips On a phone Clear filters keeps one line beside the chips. With a count the row says "Showing 7 of 21". |
| R15 | Configuration editing A key hint (`fui-key-hint`) is a labelled copy button on the label's line, reachable by keyboard, shown on hover and focus, and always shown on a phone. |
| R16 | Score scale |
| R17 | Toasts and popover form A toast is a title, an optional line and an optional action; the action sits under its text. The bad tone is an alert; a toast with an action stays until dismissed. A toast never repeats a banner or a load report that says the same. On a phone the page keeps clear of the toast region while a toast shows. |
| R18 | Focus ring on every control |
| R19 | Sign-in. One card (`fui-sign-in`) and its caption in a centred column (`fui-sign-in-stack`): the lockup as plain text, an optional title (`fui-sign-in-title`), the notice of the state in its tone (`fui-sign-in-notice`, a status, or an alert for a refusal), the provider's action (busy while redirecting, disabled while unavailable), the basic form only where the console has one, muted lines (`fui-sign-in-line`) and the deployment's links. No placeholder repeats a label. |
| R20 | Error page (`fui-status-page`) The status in words before its number, as one quiet line ("Not allowed · 403"); the title names the error in plain words; then what it means for you (who you are and the scope or role for a 403, the address and what it may mean for a 404, who can fix it for a 500), a quiet hint for the raw scope or key, and the way on as links. The time and the request id sit in a closed disclosure. It renders in the shell, and in the centred layout when no one is signed in. |
| R21 | Settings rows hugging their options |
| R22 | History filter |
| R23 | Date field |
| R24 | Title line and mode badge |
| R25 | Date picker, popover and sheet |
| R26 | The settings page. T3 replaces it: no in-page sidebar. |
| R27 | Setting rows |
| R28 | Radio preview cards Saved has a fixed slot above the control and never moves it; under preview cards it takes its own line. The three cards share one row on a phone. |
| R29 | Select as a button with its listbox. The listbox opens as F13 does: a solid raised surface, a full 1px border in the strong hairline, the card radius and the overlay shadow, 4 px under its trigger and aligned to its start (`is-end` flips it to the end edge, `is-up` above, where there is no room). It is as wide as its widest option and at least as wide as its trigger, never clipped, capped at 20 rem; an option's note is a second muted line. The trigger and the options name a value the same way ("1.1.0", the trigger "1.1.0 (latest of 2)" with "Version" as its label). A select in the title area sits under the subtitle as `fui-title-select`. |

### From FerroFED (F1 to F14), for every console

| | Spec |
|---|---|
| F1 Revision lifecycle and pending card | Saved, Approval, Apply and Confirm, each step done, current, waiting, not needed or failed. Stacked on a phone. |
| F2 Commit-confirm countdown | A ring and the time left in words, with Confirm and Roll back now. |
| F3 Write-only secret field | Keep, Set a value, or Read from a file. The current state shows by reference only, never the value. |
| F4 Provenance tag (`fui-tag-locked`) | A lock glyph: the value is owned by a file, a directory or a reserved namespace. |
| F5 Fan-out outcome | The per-node state of one request. |
| F6 Replica line | "Answered by replica", only on reference pages. |
| F7 Repeatable rows | Add and remove inline. Each row's Remove is its last control. On a phone the fields stack and Remove stays on the row's first line, beside the first field (or the sub-card's heading), never on a line of its own. M4's filter rows and M6's target rows are F7 rows. |
| F8 Editor toolbar | The editor's own state, above it. |
| F9 Topology | Its List twin's rows are scoped to `fui-topology-list`, so the base tree and M1 keep their own rows. The canvas hint's spinner takes the hint's light text. Hub layout with Bézier edges and horizontal tangents. The line style plus one marker at the child end carries the state; labels appear only on problem edges. Pulse along the path. Toolbar with the Status menu as the legend, and a zoom cluster whose icons name themselves in tooltips above. A canvas with no zoom cluster (the B9 flow, a small map) has no bottom padding and no grab cursor. A card click opens its page; the "i" button or Space opens the quick look. |
| F10 Nested sidebar section | An organisation and its nodes, with health dots. |
| F12 State | `fui-state`, the dot shape, colour and word: a filled circle, a diamond, a cross, a hollow ring (family rule 12); the sparkline. The top bar's status pill takes the same shapes, and below 64 rem it keeps its shape with the words as its accessible name and tooltip: the words sit in `fui-roomy-text`, visually hidden below 64 rem as `fui-sr-only` hides text, and the tooltip holds the words and the detail. |
| F13 Menu | Anchored 4 px from its trigger, ink items with muted icons, 36 px rows (40 px on touch), destructive items last. Below 40 rem a labelled menu opens as a bottom sheet with Cancel. A bottom sheet carries its own scrim (a spread shadow under it), so it always stands above the dimmed page; the user menu is a sheet below 40 rem too, its theme choices menuitemradio. The user menu is a popover its button opens with no script; its theme choices are submit buttons of one form posting to the console's theme route, Sign out is a form post, and a control the top bar shows from 40 rem up (a version choice) moves into it below that (`fui-menu-version`). Separators use the strong hairline in dark. |
| F14 Action group | One primary action plus a More menu; a split variant for variants of one action. The one exception: the Query workbench's title row holds Run and Save to the CDR without More, since both act on the query being written; while it runs, Save folds into More. |

### Added in the shared library pass

| | Spec |
|---|---|
| Card header row (`fui-list-head`) | T1's title, count (`fui-list-count`, muted after a middle dot) and actions above a table. |
| Tab draft badge (`fui-tab-draft`) | Accent tint, micro text, on a tab that holds a draft. |
| Tab select (`fui-tabs-select`, `fui-tabbed-many`) | Replaces a long tab row on a phone. |
| Settings panel (`fui-settings-panel`, `-head`, `-intro`) | T3's panel. |
| Tooltip above (`fui-tooltip-above`) | For controls at the foot of a canvas. |
| Search result row | The dot or glyph, then the name over its line. Results are grouped by kind; a FerroFED creating_system_id resolves to its node's Routes tab. |
| Self-description card (`fui-self-description`) | About's code card, with its own scroll, the scroll cue and token colours. |
| Top-bar status pill | The product's state in two words, the same shape in every console: "Server up" (FerroPIX), "Gateway up" (FerroFED), "Bridge up" (FerroBRIDGE), then unreachable, unknown or refused 401. The version is in its tooltip, after the words, and on About, never in the pill (`fui-topbar-status`). |

### From FerroBRIDGE (B1 to B10), for every console

| | Spec |
|---|---|
| B1 Code viewer (`fui-codeview`, in `fui-codeview-card`) | A read-only file in a card: a head with the path as its title in the prose font at the title weight, never bold mono, breaking only after a slash; its language and owner tags, the line count and the short digest; then the numbered lines. Code never wraps inside a token: a long line scrolls sideways with the scroll cue. A linked line takes `is-current` (accent tint and a 3 px accent edge). A diagnostic sits under the line it names (`fui-code-diag`, bad or warn tone), takes no line number and stays in view while the line scrolls. In a B10 section it runs edge to edge and the section head is its one head; on a phone its meta goes under the title. A document a console reads in several formats (`fui-document`) keeps the formats as links over the viewer, and its highlighted, raw and rendered views (`fui-document-view`) as links to each view, all drawn by the server, so every view is reached with no WebAssembly; an island switches them in place. |
| B2 Code editor (`fui-mapping-editor`) | The library editor with F8's toolbar above, gutter marks for diagnostics, and a status line below: an unsaved dot (`fui-editor-dirty`), the cursor's line and column, the language. Beside its diagnostics at wide widths (`fui-editor-split`), stacked under 64 rem. Lines never wrap. In a B10 section it has no card frame and runs edge to edge; an empty editor is 6 rem tall; on a phone its toolbar keeps its tools on one line. |
| B3 Diagnostics list (`fui-diags`, `fui-diag`) | One row per finding: the severity in shape, colour and word, the code in mono, the line and column as a link to the line, the message, and the mapping it belongs to. Errors first. Grouped by file under `fui-diags-file`, whose name follows family rule 4. The row for the current line takes `is-current`, in step with B1 and B2. One column on a phone. |
| B4 Load report (`fui-load-report`) | A loaded set's outcome in one line: the tone's icon, a title, the counts (`fui-load-counts`) and at most one action at the end. It sits under the page title or at the top of a panel. Ok, warn and bad tones; the bad tone takes the bad ground. It replaces a toast for the same outcome. On a phone the glyph keeps its line beside the title. |
| B5 Transform pair (`fui-transform`) | An operation's options in one row (`fui-transform-options`) with Run at its end, then two flat panes split by a hairline: the input and the answer, each with a sticky head (title, format tag, its own actions) and the answer's state in the answer pane's head. Before the first run, one quiet line where the answer will be. A refusal sits in the answer pane. Stacked under 64 rem; Run spans the row on a phone. The options row aligns fields on their label line, so an error under one field moves no other field; Run and a check group sit at the row's foot. |
| B6 OMOP row preview (`fui-rowgraph`) | The rows one input becomes, table by table: the table name in mono with its row count, then field and value pairs. A concept that did not resolve shows as `0` with a warn mark (`fui-concept-zero`). |
| B7 ETL run report (`fui-funnel`, `fui-barlist`) | Where the inputs went: one stacked bar (written, skipped, refused) with a legend of big tabular figures, then the rows per table as a bar list with the numbers right-aligned. The bar list stacks its track under the name on a phone. |
| B8 HL7 v2 viewer (`fui-hl7-raw`, `fui-hl7-segs`) | The raw message in mono with the separators and segment ids tinted, never wrapped, scrolling sideways; and the parsed view, one block per segment with its fields by position, name and value. The field names drop on a phone. |
| B9 Flow (`fui-pipeline`) | F9 drawn as a pipeline with no hub: lanes from callers to upstreams, the same edge styles and end marks, labels only on problem edges, the pulse along a hovered path. No zoom cluster, so no bottom padding and no grab cursor. One quiet foot line. On a phone it opens as its list (`fui-pipeline-list`), one lane heading and one row per card. |
| B10 Panel section (`fui-panel-section`) | The parts of a tab panel: a heading row (title, count, actions) and a hairline between sections. Never a card inside a card. A table, a B3 list or a B1 viewer inside a section runs edge to edge to the panel's frame (with no max-width, so it reaches both edges). `fui-source-split` puts a file beside its diagnostics, stacked under 64 rem. |
| Planned and proposed mark (`fui-proposed`) | A dashed, muted line with a badge ("Planned #612" or "Proposed") and one sentence on what will be there. It appears only on screens in the P group, which no live page links to, and never beside invented figures. |
| Scroll cue (`fui-scroll-cue`) | Every pane that clips long code (B1, B2, B5, B8, and About's self-description card) takes the one class: a thin scrollbar and a soft shadow on the clipped edge, which goes once you reach the end. |

### From FerroTERM (M1 to M6), for every console

| | Spec |
|---|---|
| M1 Concept tree (`fui-concept-tree`, in `fui-browser`) | The base tree as a hierarchy read one level at a time. A name is a name in a list (rule 4): ink, no underline, the row is the target, and it never wraps; a deep level scrolls sideways. The selected concept takes the accent tint; the path from the root to it is in the heavier weight; an inactive concept carries M2's inactive mark. A level larger than a page ends in one quiet row (`fui-tree-more`, "40 children, of which the page shows 25 · Show the next 25"); a level being read shows one row with the spinner (`fui-tree-loading`). ← → open and close, ↑ ↓ move, Enter reads. The concept browser (`fui-browser`) puts the search (`fui-match-list`), the tree and the concept as three panes of one panel, split by hairlines, stacked under 64 rem. A tree wider than its panel scrolls under the scroll cue; the focus ring is on the row; a uid never breaks. Over a flat code system the browser takes two panes (`fui-browser.is-two-pane`, from 64 rem): the concept gets the room of two. |
| M2 Coding (`fui-coding`) | A display over its code in mono and, where it helps, its system; `fui-coding-inline` puts the code after the display on one line. The code never breaks inside itself. Inactive (`is-inactive`, `fui-inactive-mark`) is a hollow ring, the muted colour and the word Inactive. A canonical (`fui-canonical`) breaks only after a slash. A row's name over a quiet second line in its title cell is `fui-row-sub`. |
| M3 Version select (`fui-version-select`) | The FHIR version a console reads through, as one compact control in the top bar: the version alone ("R4 ▾"), its label in the tooltip and the accessible name, no prefix text. Its menu lists each root with its release in a second column, a check on the one in view and "default" on the default. Below 40 rem it leaves the top bar and sits in the user menu as a segmented control (`fui-menu-version`). |
| M4 Clause (`fui-clause`, in `fui-clauses`) | One compose clause of a value set: the Include or Exclude mark, its source as the title, a plain sentence of what it selects, then its filter rows (`fui-clause-filter`: property, operator, value, Remove) and its codes (`fui-clause-codes`, each an M2 coding). Include is a solid accent edge, Exclude a dashed warn edge, with the word beside, so the kind never rests on colour. An invariant the clause breaks is named by its own number inline (`fui-clause-issue`, "vsd-3 This clause lists codes and filters at once."). Read-only on the resource page, editable in the composer; clauses run edge to edge in their section. |
| M5 Root matrix (`fui-root-matrix`) | A table whose columns are the FHIR roots. Each column head carries the root, its release and whether it answered, as an F12 state. The first column stays while the rest scroll sideways. A cell a root does not declare says "not declared"; a root that did not answer shows "no answer" in every cell. It scrolls sideways under B1's scroll cue. |
| M6 Mapping row (`fui-relation`) | A concept map's mapping: the source coding, the relationship as a word with its arrow ("broader than the target"), the target coding, and the comment. "Maps to nothing" is said in words with its own glyph, never an empty target. In an editor each source code is one block (`fui-map-element`) with its targets under it. |
| Editor parts | `fui-section-intro` (one sentence under a section's heading), `fui-add-row` (the Add buttons under a list), R4 with no field (the dock's state line sits in the field's place beside Discard and Save), and B5's options row inside a section (no padding of its own, Run beside its fields). |
| Operation answers | `fui-request` (the request an answer came from, in mono, breaking after slashes and ampersands, with Copy; the raw answer in a disclosure under it as B1), a hairline between B5's options and the answer, `fui-field-checks` (check boxes stacked as one field), and B5 dense (`is-dense`: four or more fields share the row so Run stays on it). An id field in the options row carries `is-id`: it starts at 24 rem, wide enough for a whole uuid or versioned object uid, and takes the rest of the row; a field for another long token (a list of scopes) uses it too. Every verdict is B4. |

### From FerroEHR (E1 to E8), for every console

| | Spec |
|---|---|
| E1 Subject line (`fui-subject`) | Whose record this is, under the title: the label Subject, then the subject id masked as R13 with its namespace as the tag and the audited Reveal, or "No external subject" in words; then the record's capability states as F12 (Queryable, Modifiable; a cleared one is the hollow ring and "Not queryable"). Wraps between its parts, never inside the identifier. |
| E2 Version history (`fui-at-time`, `fui-version-cell`) | A versioned object's revision table in a B10 section: the version with its Current mark kept together (or Deleted as F12), the change type, who committed it, a relative time, the contribution as a link. Under it, At a time: one row of a date and time field and Open, with one quiet line on what it opens. An older version opened elsewhere says so in one info notice ("You are reading version 2 of 3 · Read the latest"); the title row then leaves out the version. The envelope of the selected version is a facts section. The version being read is marked in its table (`aria-current`). At a time sits under the table with a hairline and a gap. |
| E3 Clinical reading (`fui-reading`) | A canonical document read without its template: each RM section is a heading with its archetype node id as a quiet hint; each ELEMENT a label and value row (two columns, one on a phone); a quantity keeps its unit as a person reads it (°C, mm Hg) with the UCUM code as the tooltip; a coded value is M2 inline; a null flavour is the hollow ring and its words. A nested CLUSTER is indented with a hairline edge. Bookkeeping is folded away and says so once in a section foot. |
| E4 Folder tree (`fui-folder-tree`) | M1's tree for a directory: a folder carries the folder glyph and its item count; a reference carries the file glyph in the accent and is a name in a list (rule 4) over its short uid. In edit (`is-editing`), each row ends in its icon actions (add folder, add reference, rename, remove; a reference has only Remove) and a rename is an inline field that grows with the name (12 to 28 rem); on a phone it takes its own line under the row's glyph and actions. Names never shrink; a deep tree scrolls sideways. |
| E5 Path browser (`fui-browser.is-two`, `fui-path-tree`, `fui-rm-type`, `fui-path`, `fui-archetype`) | A template's paths: M1's tree beside the node inspector, two panes of one panel, stacked under 64 rem. The RM type is a quiet mono tag in the row. The inspector is facts: the path, the RM type, occurrences, units, the FLAT path, and a coded node's value set as M2 codings, with Build a query on this path in its head. An archetype id or HRID (`fui-archetype`) breaks only after a dot; a path (`fui-path`) only after a slash, a dot or an opening bracket, and scrolls sideways when one segment is wider than its box. |
| E6 Condition groups (`fui-cond-group`, `fui-cond`) | The query builder's criteria: a group's head reads as one phrase, "Match [All | Any] of these conditions", the choice a segmented control, then a check box "Match the opposite (NOT)"; each group has an accent edge (dashed when negated; a nested group's edge is the strong hairline). A condition is an F7 row: the path as a select with its archetype path as a quiet mono line under it, the operator, a typed value (a unit suffix, a coded select), Remove; on a phone Remove stays on the first line. Add a condition and Add a group close each group. |
| E7 Results pane (`fui-results-window`) | A query's answer as a B10 section: the row count in the heading, Table or Chart as a segmented control and Export CSV and Export JSON as its actions, the row window in one quiet line ("Rows 1 to 3 of 3, inside the query's own LIMIT 50"), then the table edge to edge or the base line chart with one series per numeric column over the time column. Shared by every screen that runs a query. |
| E8 Scope grant (`fui-scopes`, `fui-scope`) | One SMART scope parsed: the raw scope in mono, never broken; the compartment and the resource in words; the five operations as tags, a permitted one on the accent tint and an absent one struck through (`del`), so the grant never rests on colour; a broad grant carries the warn badge Broad access. A malformed scope is the bad tone with what the grammar expected. |
| R17 anchored | A popover form inside an anchor (Save to the CDR) is a native `popover`, placed against its trigger by CSS anchor positioning: it opens 4 px under its trigger, aligned to its end edge; on a phone it rises from the foot of the screen at the content width. On a phone it is a bottom sheet over a scrim. |
| One overlay surface | Every open popup (F13 menu, R29 listbox, R17 popover, R25 date picker, the search answer) has a full border in the strong hairline, and in dark a ground lifted above the raised cards, so it never reads as pasted over what it opens on. A toolbar's date range opens as R25's bottom sheet on a phone. |
| A row's actions on a phone | In a stacked table inside a B10 section, the row-menu cell sits on the card's first line at its end, as F7 keeps Remove on a repeatable row's first line. |

## Checks every console runs

| Check | Rule |
|---|---|
| Build | Every internal link resolves to a page. Every nine-digit run fails the BSN eleven-proof, so no page carries a plausible BSN. |
| `render.py` | Every page at 1440 px light, 1440 px dark and 390 px light, full page, at 2x. |
| `overflow.py` | Flags a box wider than its parent, anything past the viewport, overlapping text, text past an SVG card, a word broken inside itself, and a short label forced over two lines. Open menus and tooltips are intended and are named in each review. |
| `hover.py` | Hovers and focuses every control and fails when anything outside it moves. |
| `contrast.py` | Every visible text run on every page, light and dark, against WCAG 2.2 AA: 4.5:1 for body text, 3:1 for large text. Disabled controls and placeholders are exempt. |
| `selects.py` | Every closed select at 1440 and 390 px whose chosen text is wider than its box. |
| `popups.py` | Every open popup (listbox, menu, popover, date picker, search answer) at 1440 light, 1440 dark and 390 light: an option clipped or past the popup's edge, a popup past the screen, a popup that is not solid (transparent ground, no border, no shadow), and one that does not stand out from what it opens over (ground within 1.08:1 and border within 1.3:1). |
| `forms.py` | At 1440 and 390 px: fields side by side in one row whose labels (or, with a hidden label, controls) do not share a top line, a check group left out since its labels sit on the inputs' line; a text input whose placeholder, or whose value when the field is narrower than its row, is wider than the room inside it (a value longer than a field that already spans its row scrolls, as any text input does); an open phone drawer drawn as a native popover (not docked at the left and top edges, not full height, a margin, a border on every side); and two visible primary actions with the same label outside a dialog, menu, popover or specimen card. |
