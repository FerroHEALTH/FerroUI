<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Moving a console to FerroUI 0.6

FerroUI 0.6.0 renders every component on the server from Askama templates
and adds behaviour with small plain ES modules. The library no longer
depends on Leptos, and a console built on 0.5 changes its whole UI layer:
its pages become axum handlers that return HTML, and its WebAssembly half
goes away. This page says what replaces each 0.5 part, with the code before
and after, and the order the consoles move in. The decision and its reasons
are on [FerroUI #510](https://github.com/FerroHEALTH/FerroUI/issues/510).

No specification governs the rendering mode: our own design (the owner's
rulings of 2026-10-29 on #510).

## What a console gains

- A navigation loads one complete document, drawn by your server with the
  state it read. Nothing waits for WebAssembly before a click works, and no
  panel goes blank while a router asks the server again.
- An in-page change (a tab, a tree level, a menu, a dialog) is a module
  toggling what your server drew, so the expansion, the scroll and the
  selection stay where they were.
- One build: `cargo build`. No `cargo-leptos`, no `wasm32` target, no
  `--split`, no site bundle and no bundle budget of your own. The modules
  are vendored in the crate and served by it.
- A strict policy with no nonce: `script-src 'self'; style-src 'self'`.

## The order the consoles move in

Each console moves in its own repository, under its own migration issue,
pinned to `=0.6.0`:

1. **FerroEHR**, the viewer first: it has the most trees, tabs and
   identifiers, the parts the hydrated mode lost state in.
2. **FerroBRIDGE**: the editor, the answer-mode check and the load report.
3. **FerroFED**: the secret fields, the repeaters and the commit
   countdown.
4. **FerroTERM**: the concept trees, the version select and the shortcut
   legend.
5. **FerroPIX**: the worklist, the side panel, the session watch and the
   masked identifiers.

A console moves whole: a page drawn by Leptos and a page drawn by a handler
cannot share one shell.

## Step by step

### 1. The dependencies

Before:

```toml
[dependencies]
ferrohealth-ui = "=0.5.8"
leptos = { version = "0.8", features = ["nightly"] }
leptos_router = "0.8"
leptos_axum = { version = "0.8", optional = true }
wasm-bindgen = { version = "0.2", optional = true }
console_error_panic_hook = { version = "0.1", optional = true }

[features]
hydrate = ["leptos/hydrate", "dep:wasm-bindgen", "dep:console_error_panic_hook"]
ssr = ["leptos/ssr", "dep:leptos_axum"]

[package.metadata.leptos]
# site-root, bin-features, lib-features, ...
```

After:

```toml
[dependencies]
ferrohealth-ui = "=0.6.0"
askama = { version = "0.16", default-features = false, features = ["derive", "std"] }
axum = "0.8"
```

Delete the `hydrate` and `ssr` features, the `[package.metadata.leptos]`
table, the `cdylib` crate type, the browser entry (`lib.rs`'s
`#[wasm_bindgen] pub fn hydrate`), the `wasm32` lint lane and the site
bundle's budget check. `ferrohealth-ui` keeps its default `axum` feature,
which serves the library's assets.

### 2. The assets and the document

Mount the library's asset router once, and draw every page through
`view::shell::document`, which writes `<html lang>`, the preference
attributes, the stylesheet, the brand's theme and the one loader script.

Before:

```rust
// The shell component, drawn on the server and hydrated in the browser.
view! {
    <!DOCTYPE html>
    <html lang=locale().tag()>
        <head>
            <Stylesheet href="/ferrohealth-ui.css"/>
            <Stylesheet href="/theme.css"/>
            <HydrationScripts options/>
        </head>
        <body><App/></body>
    </html>
}
```

After:

```rust
use ferrohealth_ui::html::Html;
use ferrohealth_ui::serve::{Assets, PREFIX, axum_router};
use ferrohealth_ui::style::Brand;
use ferrohealth_ui::view::shell::{Document, document};

let app = axum::Router::new()
    .route("/templates", axum::routing::get(templates))
    .nest(PREFIX, axum_router::router());

fn page(locale: &Locale, preferences: Preferences, title: String, body: Html) -> Html {
    document(Document {
        locale: locale.clone(),
        preferences,
        title,
        assets: Assets::of(Brand::Bridge),
        nonce: None,
        head: Html::raw(String::from(r#"<link rel="icon" href="/favicon.svg">"#)),
        body,
    })
}
```

The assets live under `/ferrohealth-ui/<version>/` with a one-year
immutable cache, so a new pin is a new path. A console with no axum serves
the same bytes from `style::STYLESHEET`, `style::Brand::theme` and
`script::MODULES` at the paths `serve::Assets` names.

Set the policy to `default-src 'self'; script-src 'self'; style-src
'self'`. The nonce is gone unless your own policy keeps one, in which case
pass it as `Document::nonce` and the loader's tag carries it.

### 3. Pages: from route groups to handlers

Before, a page was a component inside a `LazyRoute`, its data read through
a server function inside a `Transition` or a `Suspense`:

```rust
#[lazy_route]
impl LazyRoute for TemplatesRoute {
    fn data() -> Self { Self }
    fn view(this: Self) -> AnyView {
        let rows = Resource::new(|| (), |()| list_templates());
        view! {
            <Transition fallback=|| ()>
                {move || rows.get().map(|rows| templates_table(rows))}
            </Transition>
        }
        .into_any()
    }
}

#[server]
async fn list_templates() -> Result<Vec<Template>, ServerFnError> { /* ... */ }
```

After, a page is a handler that reads its data and returns the document:

```rust
async fn templates(State(cdr): State<Cdr>, request: Parts) -> Response {
    let visit = Visit::of(&request); // your own: locale, preferences, path
    let rows = match cdr.templates().await {
        Ok(rows) => templates_table(&visit.locale, rows),
        Err(error) => refusal(&visit.locale, error.into()),
    };
    let body = shell(&visit.locale, frame(&visit, rows));
    axum::response::Html(String::from(page(&visit.locale, visit.preferences, title, body)))
        .into_response()
}
```

What replaces each part:

| 0.5 | 0.6 |
|---|---|
| `leptos_router` routes and `LazyRoute` groups | axum routes, one handler per page |
| `widget::prefetch::routes` | nothing: a page is one document, and the browser's own cache keeps the assets |
| a server function read through `Resource` | the handler reads the data before it draws |
| `Transition` and `Suspense` with a fallback | nothing: the document arrives whole, so there is no fallback state |
| `SharedValue` carrying server state to the browser | the HTML: a module reads what the server drew, in `data-fui-*` attributes |
| `provide_locale` and `locale()` in both halves | one `&Locale` per request, passed to every component |
| `*Words::of_page()` props | the component takes `&Locale` and resolves its own words |
| `widget::nav::follow_path` and the shell drawn once | the shell drawn by every handler, the page marked from the request path |
| `island::PageAnnouncer` | nothing: a document load moves the focus and announces the title as the browser does |
| `island::FormNavigator`, `widget::submit::{NAVIGATES, RELOADS, drawn, posts, follow}` | nothing: a form posts or gets as the browser does |
| `widget::hydration::{hydrate, explain, hydrated, mark}` and `data-ferroui-hydrated` | nothing: no hydration |

### 4. Actions: from server functions to posts

Before, a mutation was a server function behind `<ActionForm>`, with a
return field for the post without WebAssembly:

```rust
let remove = ServerAction::<RemoveMember>::new();
view! {
    <ActionForm action=remove>
        {widget::action::return_field()}
        <input type="hidden" name="member" value=member/>
        <button type="submit">{words.remove}</button>
    </ActionForm>
}

#[server]
async fn remove_member(member: String, ferroui_return: String) -> Result<(), ServerFnError> {
    store().remove(&member).await?;
    leptos_axum::redirect(&widget::action::plain_return(&ferroui_return));
    Ok(())
}
```

After, it is a form that posts to a route of yours, and the route redirects
back with an outcome the next page draws as a flash toast
(post, redirect, get):

```rust
// The page: a plain form.
let form = Html::raw(format!(
    r#"<form method="post" action="/members/{id}/remove">{fields}<button type="submit" class="fui-button fui-button-danger">{remove}</button></form>"#,
    id = Html::text(&member), fields = reason_field, remove = Html::text(&words.remove),
));

// The route.
async fn remove_member(Path(member): Path<String>, Form(form): Form<Removal>) -> Response {
    let outcome = match store().remove(&member, &form.reason).await {
        Ok(()) => "removed",
        Err(_) => "refused",
    };
    Redirect::to(&format!("/members?outcome={outcome}")).into_response()
}

// The listing reads the outcome and raises its toast as the page arrives.
let flash = match query.outcome.as_deref() {
    Some("removed") => vec![Flash::new(Tone::Ok, tr(locale, Msg::MemberRemoved))],
    Some("refused") => vec![Flash::new(Tone::Bad, tr(locale, Msg::RemovalRefused))],
    _ => Vec::new(),
};
let toasts = toast_region(locale, flash);
```

Keep the outcome out of the address where it names a patient: carry it in a
short-lived cookie of your own and clear it as the page reads it. The
library does not choose where the flash travels.

| 0.5 | 0.6 |
|---|---|
| `<ActionForm>` and `ServerAction` | `<form method="post">` to a route of yours |
| `widget::action::{return_field, RETURN, plain_return, with_error}` | the route redirects where it means to; the `no-referrer` workaround is gone |
| `widget::toast::raise` from a component | a `Flash` for `view::toast::toast_region`, or a link marked `view::toast::RAISES` that the `toast` module raises in place |
| `view::dialog::danger_action_dialog(confirmation, fields, form, done, confirm)` | `view::dialog::danger_form_dialog(locale, confirmation, ConfirmForm { action, fields }, confirm)`: the dialog's form posts to `action` |

### 5. A form whose answer is the point

Before, the answer mode drew the page the form was on with the server
function's answer, through `widget::action::draw_answers` around the router,
`answer_field`, `answered` and `secret_mark`.

After, the route the form posts to draws the page itself, with the answer
and the posted fields, and answers `200`. That is all the answer mode was:

```rust
async fn check_page(visit: Visit) -> Html { draw_check(&visit, None) }

async fn check_post(visit: Visit, Form(posted): Form<Draft>) -> Html {
    let answer = checker().check(&posted.draft).await;
    draw_check(&visit, Some((&posted, answer)))
}

fn draw_check(visit: &Visit, done: Option<(&Draft, Answer)>) -> Html {
    let draft = done.map_or("", |(posted, _)| posted.draft.as_str());
    // The textarea carries the posted text, escaped by the template;
    // a secret field is drawn empty, never with what was posted.
    ...
}
```

`draw_answers`, `answer_field`, `answered`, `Posted`, `ANSWER`, `SECRET`
and `secret_mark` have no successor: a handler holds the posted form
directly, and a secret field is one it does not write back.

### 6. Components: from `island` and `widget` to `view`

Every component is now a function under `view`, taking `&Locale` and
returning `Html`. Where it has behaviour, it marks its root with
`data-fui="<job>"` and the loader imports `scripts/<job>.js`. The 0.5
components the consoles draw, by their successor:

| 0.5 | 0.6 |
|---|---|
| `island::ToastRegion`, `widget::toast::RegionWords` | `view::toast::toast_region(locale, flash)` |
| `island::NavDrawer`, `widget::nav` | `view::nav::sidebar(locale, Sidebar { .. })`, inside `view::shell::shell` |
| `island::UserMenuButton`, `widget::menu` | `view::menu::user_menu(locale, UserMenu { .. })` |
| `island::MenuButton`, `widget::action_menu` | `view::action_menu::{menu_button, row_menu, more_menu, labelled_menu, variants_menu}` |
| `island::KeyboardTabs`, `island::TabKeys`, `island::TabSelect`, `widget::tabs` | `view::tabs::{keyboard_tabs, tab_panels, binder}` |
| `island::TreeView`, `widget::tree` | `view::tree::tree_view`; the server draws the open levels it read, and the `tree` module keeps the folds for the session |
| `island::Search`, `widget::search` | `view::search::search_box(locale, SearchBox { .. })` |
| `island::ShortcutLegend`, `widget::shortcuts::{LegendWords, typing, phone_layout}` | `view::shortcuts::shortcut_legend`; the key handling is the `shortcuts` module's |
| `island::DatePicker`, `island::DateRangePicker`, `widget::date` | `view::date::{date_field, date_range}` |
| `island::Combobox`, `widget::combobox` | `view::combobox::{combobox, keyed_combobox, searched_combobox}` |
| `island::PickerField`, `widget::picker` | `view::picker::picker_field` |
| `island::Repeater`, `widget::repeat::fresh_key` | `view::repeat::{repeat, fresh_key}` |
| `island::CopyButton`, `island::KeyHint`, `widget::copy` | `view::copy::{copy_button, copy_from, key_hint}` |
| `island::HoldToShow`, `widget::masked::HoldWords` | `view::masked::hold_to_show` |
| `widget::identifier` (and `KEEP`) | `view::identifier::{identifier, identifier_list, identifier_static}` |
| `island::CommitCountdown`, `widget::countdown` | `view::countdown::countdown` |
| `island::SidePanel`, `widget::side_panel` | `view::side_panel::side_panel` |
| `island::WorkList`, `widget::worklist` | `view::worklist::worklist` |
| `island::TopologyKeys`, `widget::topology` | `view::topology::topology` |
| `island::DocumentViews`, `widget::document` | `view::document::document_pane` |
| `island::ChartLegend`, `widget::chart` | `view::chart::{legend, line_chart, activity_chart, proportion}` |
| `island::PreferenceForm`, `widget::preference` | `view::preferences::{preferences_page, preference_row, select_preference, switch_preference}` |
| `widget::editor` (`code_editor_by_edit`, `EditorWords`, `LineMark`, `caret_at`) | `view::editor::{code_editor, editor_panel, editor_split, editor_diagnostics, caret_at}`; a check as you type is a form your route answers |
| `widget::upload::UploadWords` | `view::upload::upload_field` |
| `widget::segmented` | `view::segmented::{segmented_links, segmented_form, choice_group}` |
| `widget::version` | `view::version::version_select` |
| `widget::filter` | `view::filter::table_filter` |
| `widget::pick_or_type` | `view::pick_or_type::pick_or_type` |
| `widget::focus`, `widget::announce` | nothing: the browser moves the focus on a document load |

A component in a console's own Askama template is written with
`{{ part }}`: `Html` implements `HtmlSafe`, so Askama writes it unescaped,
and every text inside it was escaped when the library drew it.

```html
{# templates/members.html in your console #}
<section class="fui-card">
  {{ header }}
  {{ table }}
</section>
```

### 7. The session watch

Before:

```rust
view! { <SessionWatch watch=widget::session::Watch::new(status, sign_in, sign_out, left)/> }
// and on the signed-out page:
view! { <SessionEnded/> }
// and where your own code meets a 401:
widget::session::gone();
```

After:

```rust
let watch = view::session::Watch::new(locale, "/session", "/sign-in", "/sign-out", left);
let watch = view::session::session_watch(&watch);   // in the shell of a signed-in page
let ended = view::session::session_ended();         // on the signed-out page
```

`Left`, `RETURN`, `WARNING_ID` and `sign_in_address` keep their names under
`view::session`. `gone()` has no successor: a `401` on a page your handler
draws is a redirect to sign-in, and the search box leaves on its own `401`
(`SearchBox::signed_out`). The session route's wire is unchanged
(`docs/consoles.md` § The session's end).

### 8. Tests

- A test of a component renders it to a string and asserts on the markup:
  `component(&Locale::english(), ..).as_str()`. There is no `Owner` and no
  `view!`.
- A journey that waited for `data-ferroui-hydrated` waits for the loader's
  mark, `html[data-fui-script]` (`script::MARK`), or for nothing when it
  only reads the page. The loader writes that mark before any module has
  arrived, so a journey that drives a component whose module changes
  nothing it can see first waits for the component's `data-fui-ready`
  (`script::READY`), which the loader writes once every module the
  component asked for has enhanced it.
- The no-WebAssembly journey becomes a no-script journey: block
  `/ferrohealth-ui/<version>/scripts/` and hold every page to its content,
  links and forms, as the gallery's `tests/it/no_script.rs` does.
- A journey of in-page state checks that a tab switch loads no document
  and a tree's folds survive a reload, as the gallery's
  `tests/it/navigation.rs` does.

## What 0.6 no longer does

- **No navigation in the page.** A link loads a document. A page that held
  state in memory across navigations (a draft in a signal) keeps it on the
  server or in the form it posts.
- **No live re-check while typing that needs the server, until 0.6.3.**
  The editor highlights as you type in the browser; a check that runs on
  the server runs when the form posts. From 0.6.3 a form marked
  `view::answer::Answer` posts as you type and on submit and takes its
  answer in place (`docs/consoles.md` § Answered in place).
- **No toast from component code at run time.** A toast comes from the
  server as a `Flash`, or from a link or a button marked
  `view::toast::RAISES`.
- **No prefetch of route code.** There is no route code to fetch.

## Before you open the pull request

1. Pin `ferrohealth-ui = "=0.6.0"` and remove every Leptos crate,
   `cargo-leptos` and the `wasm32` target from your build and your CI.
2. Draw every page through `view::shell::document`, and mount
   `serve::axum_router::router()` at `serve::PREFIX`.
3. Turn every server function into a handler or a post route, and every
   `ActionForm` into a `<form>`.
4. Set `script-src 'self'; style-src 'self'`.
5. Run your journeys with the scripts refused and with them loaded.
6. Refresh your screenshots and look at them.
