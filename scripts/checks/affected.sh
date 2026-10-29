#!/usr/bin/env bash
# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
# Reads the paths a change touches and prints what they reach, as key=value
# lines, for the CI `detect` job and for scripts/fast.sh. One table decides
# both, so the local gate and CI agree on what a change reaches. No
# specification governs this: our own design (.claude/rules/ci-cd.md).
#
#   rust=true|false     the Rust tier's inputs changed: the crates, the
#                       gallery, the workspace manifests and configuration,
#                       the screenshots the gallery's tests list
#   packages=...        the workspace packages a Rust change reaches (the
#                       library reaches the gallery, which depends on it)
#   gallery=true|false  the gallery's build, its journeys and its capture
#                       are reached
#   capture=true|false  the change can move a pixel the screenshot capture
#                       draws: everything the gallery tier reaches but a
#                       journey module that is no capture harness
#   journeys=...        a nextest filter of the journeys the change reaches,
#                       `all`, or empty (scripts/fast.sh)
#   design=...          the design sets whose pages the change reaches
#
# A change to the CI workflow, its actions or this script reaches
# everything. Markdown reaches nothing: no build, test or render reads it.
#
# Usage:
#   scripts/checks/affected.sh [--all]     read the paths from standard input
#   scripts/checks/affected.sh --self-test
set -euo pipefail
cd "$(dirname "$0")/../.."

# The design sets: the templates first, then every console with a pages.py.
design_sets() {
  local f sets="templates"
  for f in design/consoles/*/pages.py; do
    [ -f "$f" ] || continue
    f="${f%/pages.py}"
    sets="$sets ${f##*/}"
  done
  echo "$sets"
}

# The design sets drawn in BRAND (the BRAND line of each set's console.py).
sets_of_brand() {
  local set src out=""
  for set in $(design_sets); do
    if [ "$set" = templates ]; then src=design/templates; else src="design/consoles/$set"; fi
    if grep -qx "BRAND = \"$1\"" "$src/console.py" 2> /dev/null; then out="$out $set"; fi
  done
  echo "$out"
}

# The test-name words a library or gallery source file stands for.
words_of() {
  case "$1" in
    button) echo "button" ;;
    chart | charts) echo "chart stat" ;;
    code | editor | highlight | aql | json | xml | yaml) echo "editor code diagnostic" ;;
    combobox) echo "combobox" ;;
    dialog) echo "dialog confirm" ;;
    document) echo "document" ;;
    filter | pager | model | table | tables) echo "pager table listing sort filter" ;;
    form | forms) echo "form field combobox picker pick_or_type help" ;;
    layout | layouts) echo "layout grid cards reflow sidebar drawer" ;;
    menu) echo "menu" ;;
    nav | shell) echo "sidebar drawer rail top_bar menu" ;;
    overlays) echo "dialog popover toast menu" ;;
    popover) echo "popover" ;;
    segmented) echo "segmented density" ;;
    side_panel) echo "panel" ;;
    sign_in) echo "sign" ;;
    surface) echo "card stat" ;;
    preference | preferences) echo "preferences density" ;;
    time) echo "timestamp" ;;
    toast) echo "toast" ;;
    overview | templates) echo "every_page" ;;
    *) echo "$1" ;;
  esac
}

classify() {
  local all="$1" path stem dir module
  local rust=false lib=false app=false gallery=false capture=false
  local journeys_all=false page_wide=false design_all=false
  local words="" modules="" sets=""

  everything() {
    rust=true lib=true app=true gallery=true capture=true journeys_all=true design_all=true
  }

  if [ "$all" = true ]; then everything; fi
  while IFS= read -r path; do
    [ -n "$path" ] || continue
    case "$path" in
      *.md) continue ;;
      .github/workflows/ci.yml | .github/actions/* | scripts/checks/affected.sh | scripts/chrome-lock.sh)
        everything
        ;;
      Cargo.toml | Cargo.lock | rust-toolchain.toml | .config/*)
        rust=true lib=true app=true gallery=true capture=true journeys_all=true
        ;;
      clippy.toml | rustfmt.toml | deny.toml)
        rust=true lib=true app=true
        ;;
      scripts/release/gallery-*.sh | scripts/release/journey-tools.sh | scripts/checks/gallery-*.sh)
        gallery=true capture=true journeys_all=true
        ;;
      docs/screenshots/*)
        rust=true app=true
        ;;
      crates/ferrohealth-ui/css/*)
        rust=true lib=true app=true gallery=true capture=true page_wide=true design_all=true
        ;;
      crates/ferrohealth-ui/themes/*.css)
        rust=true lib=true app=true gallery=true capture=true page_wide=true
        stem="${path##*/}"
        sets="$sets $(sets_of_brand "${stem%.css}")"
        ;;
      crates/ferrohealth-ui/tests/*)
        rust=true lib=true
        ;;
      crates/ferrohealth-ui/src/view/* | crates/ferrohealth-ui/src/table/* | crates/ferrohealth-ui/src/highlight/* | crates/ferrohealth-ui/scripts/*.js | crates/ferrohealth-ui/templates/*/* | app/ferrohealth-ui-gallery/src/pages/*)
        rust=true app=true gallery=true capture=true
        case "$path" in app/*) ;; *) lib=true ;; esac
        stem="${path##*/}"
        stem="${stem%.rs}"
        stem="${stem%.js}"
        case "$path" in
          crates/ferrohealth-ui/templates/*)
            dir="${path%/*}"
            stem="${dir##*/}"
            ;;
        esac
        if [ "$stem" = mod ]; then
          dir="${path%/mod.rs}"
          stem="${dir##*/}"
        fi
        words="$words $(words_of "$stem")"
        ;;
      crates/*)
        rust=true lib=true app=true gallery=true capture=true journeys_all=true
        ;;
      app/ferrohealth-ui-gallery/tests/it/*/*)
        rust=true app=true gallery=true capture=true journeys_all=true
        ;;
      app/ferrohealth-ui-gallery/tests/it/*.rs)
        # Every module of tests/it/ is a journey module that runs whole, but
        # the harness every journey uses and the two that open no browser,
        # so a new module needs no line here.
        rust=true app=true gallery=true
        module="${path##*/}"
        module="${module%.rs}"
        case "$module" in
          browser | main | stack) journeys_all=true capture=true ;;
          http) ;;
          screenshots) capture=true ;;
          states) capture=true modules="$modules $module" ;;
          *) modules="$modules $module" ;;
        esac
        ;;
      app/*)
        rust=true app=true gallery=true capture=true journeys_all=true
        ;;
      design/consoles/*)
        dir="${path#design/consoles/}"
        sets="$sets ${dir%%/*}"
        ;;
      design/themes/*.css)
        stem="${path##*/}"
        sets="$sets $(sets_of_brand "${stem%.css}")"
        ;;
      design/*)
        design_all=true
        ;;
    esac
  done
  if [ "$app" = false ] && [ "$lib" = true ]; then app=true; fi

  local packages=""
  if [ "$rust" = true ]; then
    if [ "$lib" = true ]; then packages="ferrohealth-ui"; fi
    if [ "$app" = true ]; then packages="${packages:+$packages }ferrohealth-ui-gallery"; fi
  fi

  local journeys=""
  if [ "$journeys_all" = true ]; then
    journeys=all
  elif [ -n "$words$modules" ] || [ "$page_wide" = true ]; then
    # Every touched area also runs the two journeys that load every page.
    local alternation="every_page_of_every_brand_loads|every_page_keeps_its_content" word
    if [ "$page_wide" = true ]; then alternation="$alternation|every_page|reflow|^painted::"; fi
    for word in $words; do alternation="$alternation|$word"; done
    for module in $modules; do alternation="$alternation|^$module::"; done
    journeys="test(/$alternation/)"
  fi

  local design="" set
  if [ "$design_all" = true ]; then
    design="$(design_sets)"
  else
    for set in $(design_sets); do
      case " $sets " in *" $set "*) design="${design:+$design }$set" ;; esac
    done
  fi

  echo "rust=$rust"
  echo "packages=$packages"
  echo "gallery=$gallery"
  echo "capture=$capture"
  echo "journeys=$journeys"
  echo "design=$design"
}

self_test() {
  local failed=0 every
  every="$(design_sets)"
  expect() {
    local name="$1" paths="$2" want="$3" got
    got="$(printf '%b' "$paths" | classify false)"
    if ! printf '%s\n' "$got" | grep -qxF -- "$want"; then
      echo "self-test: $name: no line '$want' in:" >&2
      printf '%s\n' "$got" | sed 's/^/  /' >&2
      failed=1
    fi
  }
  expect "docs only reach nothing" 'README.md\ndesign/COMPONENTS.md\n' "rust=false"
  expect "docs only skip the gallery" 'README.md\ndesign/COMPONENTS.md\n' "gallery=false"
  expect "docs only skip the design" 'README.md\ndesign/COMPONENTS.md\n' "design="
  expect "a view reaches both packages" 'crates/ferrohealth-ui/src/view/toast.rs\n' "packages=ferrohealth-ui ferrohealth-ui-gallery"
  expect "a script module reaches its journeys" 'crates/ferrohealth-ui/scripts/menu.js\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|menu/)"
  expect "a template reaches its area's journeys" 'crates/ferrohealth-ui/templates/toast/region.html\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|toast/)"
  expect "a view reaches its journeys" 'crates/ferrohealth-ui/src/view/toast.rs\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|toast/)"
  expect "a view reaches no design set" 'crates/ferrohealth-ui/src/view/toast.rs\n' "design="
  expect "a journey module runs whole" 'app/ferrohealth-ui-gallery/tests/it/widgets.rs\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|^widgets::/)"
  expect "the test harness runs every journey" 'app/ferrohealth-ui-gallery/tests/it/browser.rs\n' "journeys=all"
  expect "the gallery server runs every journey" 'app/ferrohealth-ui-gallery/tests/it/stack.rs\n' "journeys=all"
  expect "a journey module no list names runs whole" 'app/ferrohealth-ui-gallery/tests/it/dates.rs\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|^dates::/)"
  expect "a state area runs every journey" 'app/ferrohealth-ui-gallery/tests/it/states/overview.rs\n' "journeys=all"
  expect "the HTTP tests open no browser" 'app/ferrohealth-ui-gallery/tests/it/http.rs\n' "journeys="
  expect "a library test skips the gallery" 'crates/ferrohealth-ui/tests/it/contrast.rs\n' "gallery=false"
  expect "the stylesheet reaches every set" 'crates/ferrohealth-ui/css/ferrohealth-ui.css\n' "design=$every"
  expect "the stylesheet reaches the page-wide journeys" 'crates/ferrohealth-ui/css/ferrohealth-ui.css\n' "journeys=test(/every_page_of_every_brand_loads|every_page_keeps_its_content|every_page|reflow|^painted::/)"
  expect "a crate theme reaches its brand's sets" 'crates/ferrohealth-ui/themes/ferrofed.css\n' "design=templates ferrofed"
  expect "a design theme reaches its brand's sets" 'design/themes/ferroehr.css\n' "design=ferroehr"
  expect "a console's page reaches its set" 'design/consoles/ferroterm/pages.py\n' "design=ferroterm"
  expect "a console's page skips the Rust tier" 'design/consoles/ferroterm/pages.py\n' "rust=false"
  expect "a design tool reaches every set" 'design/tools/overflow.py\n' "design=$every"
  expect "the templates reach every set" 'design/templates/common.py\n' "design=$every"
  expect "the workflow reaches every set" '.github/workflows/ci.yml\n' "design=$every"
  expect "the workflow reaches the gallery" '.github/workflows/ci.yml\n' "gallery=true"
  expect "the lockfile reaches every journey" 'Cargo.lock\n' "journeys=all"
  expect "the journey tools reach the gallery" 'scripts/release/journey-tools.sh\n' "gallery=true"
  expect "the screenshots reach the gallery's tests" 'docs/screenshots/ferrobridge/overview-light.png\n' "packages=ferrohealth-ui-gallery"
  expect "the screenshots skip the browser" 'docs/screenshots/ferrobridge/overview-light.png\n' "gallery=false"
  expect "a view reaches the capture" 'crates/ferrohealth-ui/src/view/toast.rs\n' "capture=true"
  expect "a gallery page reaches the capture" 'app/ferrohealth-ui-gallery/src/pages/forms.rs\n' "capture=true"
  expect "a theme reaches the capture" 'crates/ferrohealth-ui/themes/ferrofed.css\n' "capture=true"
  expect "a state area reaches the capture" 'app/ferrohealth-ui-gallery/tests/it/states/overview.rs\n' "capture=true"
  expect "the capture harness reaches the capture" 'app/ferrohealth-ui-gallery/tests/it/screenshots.rs\n' "capture=true"
  expect "the browser harness reaches the capture" 'app/ferrohealth-ui-gallery/tests/it/browser.rs\n' "capture=true"
  expect "a journey module skips the capture" 'app/ferrohealth-ui-gallery/tests/it/widgets.rs\n' "capture=false"
  expect "a journey module still reaches the gallery" 'app/ferrohealth-ui-gallery/tests/it/widgets.rs\n' "gallery=true"
  expect "the HTTP tests skip the capture" 'app/ferrohealth-ui-gallery/tests/it/http.rs\n' "capture=false"
  expect "docs skip the capture" 'README.md\n' "capture=false"
  expect "the workflow reaches the capture" '.github/workflows/ci.yml\n' "capture=true"
  if ! classify true < /dev/null | grep -qxF "design=$every"; then
    echo "self-test: --all does not reach every design set" >&2
    failed=1
  fi
  if [ "$failed" -ne 0 ]; then return 1; fi
  echo "affected: self-test passed"
}

case "${1:-}" in
  --self-test) self_test ;;
  --all) classify true < /dev/null ;;
  "") classify false ;;
  *)
    sed -n '/^# Usage:/,/^set /p' "$0" | sed '$d; s/^# \{0,1\}//' >&2
    exit 2
    ;;
esac
