// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The stylesheet's own rules: every class a component writes is defined in
//! it, so a console needs no scan of this crate; every size of type names a
//! step of the scale; and the layer order is declared before anything else.

use std::collections::BTreeSet;
use std::path::Path;

use ferrohealth_ui::style::{STYLESHEET, TAILWIND_THEME};

/// Every `fui-` class name in `text`; a `data-fui-` attribute name is no
/// class.
fn classes(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = text;
    while let Some(at) = rest.find("fui-") {
        let attribute = rest[..at].ends_with("data-");
        let tail = &rest[at..];
        let end = tail
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .unwrap_or(tail.len());
        if !attribute {
            found.insert(tail[..end].trim_end_matches('-').to_owned());
        }
        rest = &tail[end..];
    }
    found
}

/// The text of every Rust source file and Askama template under `dir`.
fn sources(dir: &Path) -> String {
    let mut text = String::new();
    for entry in std::fs::read_dir(dir).expect("a source folder") {
        let path = entry.expect("an entry").path();
        if path.is_dir() {
            text.push_str(&sources(&path));
        } else if path
            .extension()
            .is_some_and(|extension| extension == "rs" || extension == "html")
        {
            text.push_str(&std::fs::read_to_string(&path).expect("a source file"));
        }
    }
    text
}

#[test]
fn every_class_a_component_writes_is_defined_in_the_stylesheet() {
    let crate_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut written = classes(&sources(&crate_dir.join("src")));
    written.extend(classes(&sources(&crate_dir.join("templates"))));
    let defined: BTreeSet<String> = classes(STYLESHEET)
        .into_iter()
        .filter(|class| STYLESHEET.contains(&format!(".{class}")))
        .collect();
    let missing: Vec<&String> = written
        .iter()
        .filter(|class| !defined.contains(*class) && class.as_str() != "fui")
        .collect();
    assert!(
        missing.is_empty(),
        "classes written but not defined: {missing:?}"
    );
}

#[test]
fn every_size_of_type_names_a_step_of_the_scale() {
    let off_scale: Vec<&str> = STYLESHEET
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with("font-size:"))
        .filter(|line| !line.contains("var(--text-") && !line.contains("inherit"))
        .collect();
    assert!(off_scale.is_empty(), "sizes off the scale: {off_scale:?}");
}

#[test]
fn every_step_of_type_is_written_with_its_leading() {
    let mut unpaired = Vec::new();
    for rule in STYLESHEET.split('}') {
        let body = rule.rsplit('{').next().unwrap_or_default();
        if body.contains("font-size: var(--text-") && !body.contains("line-height:") {
            unpaired.push(body.trim().lines().next().unwrap_or_default().to_owned());
        }
    }
    assert!(unpaired.is_empty(), "a step with no leading: {unpaired:?}");
    for step in ["display", "title", "heading", "body", "small", "micro"] {
        assert!(
            STYLESHEET.contains(&format!("--leading-{step}:")),
            "the step {step} has no leading token"
        );
    }
}

#[test]
fn the_tailwind_theme_writes_no_variable_that_names_itself() {
    // NOTE: CSS Custom Properties Level 1, section 2.3; a reference block is
    // inlined into each utility, so a same-named entry is never written out.
    let reference = TAILWIND_THEME.contains("@theme inline reference {");
    let cycles: Vec<&str> = TAILWIND_THEME
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.split_once(':')
                .is_some_and(|(name, value)| value.trim() == format!("var({});", name.trim()))
        })
        .collect();
    assert!(
        reference || cycles.is_empty(),
        "entries that would unset themselves: {cycles:?}"
    );
}

#[test]
fn the_layer_order_comes_first_and_no_style_is_inline() {
    let first_rule = STYLESHEET
        .lines()
        .find(|line| line.starts_with('@'))
        .expect("an at-rule");
    assert_eq!("@layer theme, base, components, utilities;", first_rule);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let code: String = sources(&src)
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !code.contains(" style="),
        "a component writes a style attribute"
    );
    assert!(
        !code.contains("<style"),
        "a component writes a style element"
    );
}

#[test]
fn the_sticky_top_bar_pads_the_scroll_so_a_focused_control_clears_it() {
    let rule = "@media (width >= 48rem) {\n    html {\n      scroll-padding-top: calc(var(--bar)";
    assert!(
        STYLESHEET.contains(rule),
        "no scroll padding beside the sticky top bar"
    );
}

#[test]
fn a_fieldset_well_keeps_its_frame_after_the_fieldset_reset() {
    let reset = STYLESHEET
        .find("  .fui-fieldset {")
        .expect("the fieldset reset");
    let well = STYLESHEET
        .find("  .fui-fieldset.fui-well {")
        .expect("the fieldset well");
    assert!(well > reset, "the well rule must follow the reset to win");
}

#[test]
fn a_disabled_control_keeps_the_control_edge_on_a_sunken_ground() {
    // WCAG 2.2 SC 1.4.11 exempts a disabled control, but the field must still read as one on a
    // well, whose ground is the disabled control's own.
    let rule = "  :is(.fui-input, .fui-select, .fui-textarea):disabled {\n    border-color: var(--edge-control);";
    assert!(
        STYLESHEET.contains(rule),
        "a disabled control loses its edge"
    );
    let hover = "  :is(.fui-input, .fui-select, .fui-textarea):disabled:hover {\n    border-color: var(--edge-control);";
    assert!(
        STYLESHEET.contains(hover),
        "a disabled control loses its edge under the pointer"
    );
}

#[test]
fn a_timestamp_never_breaks_and_the_compact_stat_cuts_its_label() {
    let time = "  .fui-time {\n    font-variant-numeric: tabular-nums;\n    overflow-wrap: normal;\n    white-space: nowrap;";
    assert!(STYLESHEET.contains(time), "a timestamp can break");
    let compact = STYLESHEET
        .find("  @media (width < 30rem) {\n    .fui-stat {")
        .expect("the compact stat tile");
    let label = STYLESHEET
        .get(compact..)
        .and_then(|rest| rest.find("text-overflow: ellipsis;"))
        .expect("the compact label's ellipsis");
    assert!(label < 1200, "the ellipsis belongs to the compact tile");
}

#[test]
fn a_table_of_several_bodies_keeps_the_edges_and_spacing_between_them() {
    assert!(
        STYLESHEET.contains("  .fui-table tbody:last-of-type tr:last-child > * {"),
        "a body's last row loses its edge before the next body"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-table-stack .fui-table tbody tr + tr,\n    .fui-table-stack .fui-table tbody + tbody {"
        ),
        "the stacked bodies touch"
    );
}

#[test]
fn a_field_and_its_button_take_the_whole_row_below_30rem() {
    let rule = "  @media (width < 30rem) {\n    .fui-field-flow > :is(.fui-field, button, .fui-button) {\n      flex: 1 1 100%;";
    assert!(
        STYLESHEET.contains(rule),
        "a field flow keeps its row on a phone"
    );
}

#[test]
fn a_button_inside_a_field_keeps_its_own_width_at_the_start() {
    assert!(
        STYLESHEET.contains("  .fui-field > :is(button, .fui-button) {\n    justify-self: start;"),
        "a button in a field's grid would stretch across it"
    );
}

#[test]
fn a_forms_help_lines_show_by_its_switch_with_no_script() {
    // WCAG 2.2 technique C7: hidden from sight while off, still read.
    let rule = "  form:has(.fui-help-switch > input:not(:checked)) .fui-explain {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
    border: 0;
  }";
    assert!(
        STYLESHEET.contains(rule),
        "a help line would not follow its form's switch"
    );
}

#[test]
fn the_stylesheet_styles_no_wrapper_the_markup_never_draws() {
    assert!(
        !STYLESHEET.contains("island"),
        "a selector through the old island wrapper matches nothing the server draws"
    );
}

#[test]
fn a_sort_form_and_a_pager_form_add_no_box() {
    assert!(
        STYLESHEET.contains("  .fui-sort-form {\n    display: contents;"),
        "the header lays out as its sort button alone"
    );
    assert!(
        STYLESHEET.contains("  .fui-pager-form {\n    display: contents;"),
        "the pager keeps its row with its forms inside"
    );
}

#[test]
fn a_cards_rhythm_spaces_its_children() {
    assert!(
        STYLESHEET.contains("  .fui-card > * + * {"),
        "the parts of a card would lose the space between them"
    );
}

#[test]
fn a_phone_row_keeps_its_checkbox_on_its_first_line() {
    // design/COMPONENTS.md, § The table: a row's checkbox sits on its first line.
    assert!(
        STYLESHEET.contains(
            "    .fui-table-stack .fui-table tbody tr > td.fui-row-check {\n      position: absolute;"
        ),
        "a stacked row's checkbox hangs at the start of its first line"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-worklist .fui-table-stack .fui-table tbody tr > td.fui-row-check {\n      position: static;\n      order: 0;"
        ),
        "a work-list row's checkbox leads its first line"
    );
}

#[test]
fn a_keyboard_tabs_panel_is_the_sheet_its_tabs_open_into() {
    assert!(
        STYLESHEET.contains(
            "  .fui-tabbed > .fui-tab-panel {\n    min-width: 0;\n    border: 1px solid var(--edge-strong);"
        ),
        "the keyboard tabs' panels would lose the binder sheet"
    );
}

#[test]
fn the_phone_tab_select_replaces_the_row_only_once_it_works() {
    // NOTE: console-ui.md, the no-script floor: with no script the row of links is the floor.
    assert!(
        STYLESHEET.contains(
            "    .fui-tabbed-many:has(.fui-tabs-select select:enabled) > .fui-tabs {\n      display: none;"
        ),
        "the row would hide behind a select that cannot work yet"
    );
    assert!(
        !STYLESHEET.contains("    .fui-tabbed-many > .fui-tabs {\n      display: none;"),
        "the row hides with no condition"
    );
}

#[test]
fn a_document_view_a_link_targets_shows_until_the_module_takes_over() {
    assert!(
        STYLESHEET.contains(
            "  .fui-document-views:not(.is-live):has(.fui-document-view:target) .fui-document-view:not(:target) {\n    display: none;"
        ),
        "with no script the view a link targets replaces the chosen one"
    );
    assert!(
        STYLESHEET.contains(
            "  .fui-document-views:not(.is-live) .fui-document-view:target {\n    display: block;"
        ),
        "the targeted view shows"
    );
}

#[test]
fn every_clipping_code_pane_takes_the_one_scroll_cue_class() {
    let cue = STYLESHEET
        .find("  .fui-scroll-cue,\n")
        .expect("the scroll cue is one class");
    let rule = &STYLESHEET[cue..cue + 600];
    assert!(
        rule.contains("scrollbar-width: thin;"),
        "a thin scrollbar: {rule}"
    );
    assert!(
        rule.contains("no-repeat local") && rule.contains("no-repeat scroll"),
        "the shadow on the clipped edge goes at the end: {rule}"
    );
}

#[test]
fn a_code_viewers_path_is_a_title_in_the_prose_font() {
    let at = STYLESHEET
        .find("  .fui-codeview-path {")
        .expect("the path's rule");
    let rule = &STYLESHEET[at..at + STYLESHEET[at..].find('}').unwrap_or(0)];
    assert!(!rule.contains("font-mono"), "never bold mono: {rule}");
    assert!(
        rule.contains("font-weight: 600;"),
        "the title weight: {rule}"
    );
    assert!(
        !rule.contains("overflow-wrap: anywhere"),
        "a path breaks after a slash, never inside a name: {rule}"
    );
}

#[test]
fn a_viewer_a_table_and_a_list_of_findings_in_a_section_reach_both_edges() {
    for part in ["fui-table-scroll", "fui-diags", "fui-codeview"] {
        let rule = format!("  .fui-panel-section > .{part} {{\n");
        let at = STYLESHEET
            .find(&rule)
            .unwrap_or_else(|| panic!("{part} in a section"));
        assert!(
            STYLESHEET[at..at + 200].contains("margin-inline: calc(var(--space-loose) * -1);"),
            "{part} runs edge to edge in a section"
        );
    }
}

#[test]
fn a_findings_code_message_mapping_and_path_break_rather_than_widen_a_phone() {
    // WCAG 2.2 SC 1.4.10: a finding reads at 320 px with no sideways scroll.
    for part in ["fui-diag-code", "fui-diag-msg"] {
        let at = STYLESHEET
            .find(&format!("  .{part} {{\n"))
            .unwrap_or_else(|| panic!("the rule of {part}"));
        let rule = &STYLESHEET[at..at + STYLESHEET[at..].find('}').unwrap_or(0)];
        assert!(
            rule.contains("overflow-wrap: anywhere;") && rule.contains("min-width: 0;"),
            "{part} breaks inside a long word: {rule}"
        );
    }
    let link = "  :is(.fui-diags-file, .fui-group-row) a:not(.fui-button, .fui-tag) {\n";
    let at = STYLESHEET.find(link).expect("the file head's link");
    let rule = &STYLESHEET[at..at + STYLESHEET[at..].find('}').unwrap_or(0)];
    assert!(
        !rule.contains("nowrap"),
        "a file's linked path breaks after its slashes: {rule}"
    );
    assert!(
        STYLESHEET.contains("  .fui-diags-file > :is(a, code) {\n    min-width: 0;"),
        "a file's path may be narrower than its longest name"
    );
}

#[test]
fn the_keyboard_tab_leaves_the_row_below_40rem() {
    let phone = STYLESHEET
        .find("    .fui-tabs > .fui-tab[data-tab=\"keyboard\"] {\n      display: none !important;")
        .expect("the keyboard tab's rule");
    let media = STYLESHEET
        .get(..phone)
        .and_then(|before| before.rfind("@media"))
        .and_then(|at| STYLESHEET.get(at..phone))
        .expect("the rule's media query");
    assert!(media.starts_with("@media (width < 40rem)"), "{media}");
}

#[test]
fn the_popover_form_is_anchored_to_its_trigger_on_a_wide_screen() {
    assert!(
        STYLESHEET.contains(
            "    @media (width >= 40rem) {\n      .fui-anchor > dialog.fui-popover {\n        inset: auto;\n        margin: 0;\n        margin-block-start: 0.25rem;\n        position-anchor: --fui-anchor;\n        position-area: block-end span-inline-start;"
        ),
        "a modal dialog in the top layer would lose its trigger"
    );
    assert!(
        STYLESHEET.contains("    .fui-anchor > [commandfor] {\n      anchor-name: --fui-anchor;"),
        "the trigger names the anchor"
    );
}

/// The block of the rule whose selector line is `selector`, up to its
/// closing brace.
fn rule(selector: &str) -> &'static str {
    let at = STYLESHEET
        .find(&format!("{selector} {{\n"))
        .unwrap_or_else(|| panic!("no rule {selector}"));
    let rest = STYLESHEET.get(at..).unwrap_or_default();
    rest.get(..rest.find('}').unwrap_or(rest.len()))
        .unwrap_or_default()
}

#[test]
fn the_popover_form_without_anchor_positioning_is_centred_in_the_viewport() {
    // WCAG 2.2 SC 2.4.11: the fallback must not leave the form below the viewport's foot.
    let fallback = rule("  .fui-anchor > dialog.fui-popover");
    assert!(
        fallback.contains("position: fixed;")
            && fallback.contains("inset: 0;")
            && fallback.contains("margin: auto;"),
        "{fallback}"
    );
}

#[test]
fn every_menu_and_listbox_scrolls_inside_the_viewport() {
    // WCAG 2.2 SC 1.4.10: at 400% zoom or with many items, no item is past the screen's edge.
    for selector in ["  .fui-menu.fui-menu-anchored", "  .fui-listbox"] {
        let block = STYLESHEET
            .match_indices(&format!("{selector} {{\n"))
            .map(|(at, _)| {
                let rest = STYLESHEET.get(at..).unwrap_or_default();
                rest.get(..rest.find('}').unwrap_or(rest.len()))
                    .unwrap_or_default()
            })
            .find(|block| block.contains("max-height"))
            .unwrap_or_else(|| panic!("{selector} has no height limit"));
        assert!(
            block.contains("max-height: calc(100dvh - 2rem);")
                && block.contains("overflow-y: auto;"),
            "{block}"
        );
    }
    let sheet = rule("    .fui-menu.fui-menu-anchored.fui-menu-sheet");
    assert!(
        sheet.contains("max-height: calc(100dvh - 1rem);"),
        "the phone's sheet: {sheet}"
    );
    let user = rule("    .fui-menu.fui-menu-anchored.fui-user-menu:popover-open");
    assert!(user.contains("max-height:"), "the user menu: {user}");
}

#[test]
fn a_long_word_in_a_menu_item_an_option_or_the_user_menu_head_wraps() {
    // WCAG 2.2 SC 1.4.10: a long e-mail or canonical URL wraps in its box, never clipped.
    for selector in [
        "  .fui-menu :is(a, button).fui-menu-item,\n  .fui-column .fui-menu a.fui-menu-item",
        "  .fui-listbox-text > span:first-child",
        "  .fui-menu-header",
    ] {
        let block = rule(selector);
        assert!(
            block.contains("overflow-wrap: anywhere;") && !block.contains("nowrap"),
            "{block}"
        );
    }
}

#[test]
fn the_confirmation_dialog_scrolls_inside_the_viewport() {
    // WCAG 2.2 SC 1.4.10: a long message never hides the dialog's buttons past the screen.
    let dialog = rule("  .fui-dialog");
    assert!(
        dialog.contains("max-height: calc(100dvh - 2rem);")
            && dialog.contains("overflow: auto;")
            && dialog.contains("overflow-wrap: anywhere;"),
        "{dialog}"
    );
}

#[test]
fn a_shown_toast_keeps_the_page_foot_clear() {
    assert!(
        STYLESHEET.contains(
            "  :root:has(.fui-toasts > .fui-toast) .fui-main {\n    padding-block-end: 7rem;"
        ),
        "a toast would cover the page's last row"
    );
    assert!(
        STYLESHEET.contains("  :root:has(.fui-toasts > .fui-toast) .fui-page > .fui-footer {\n    padding-block-end: 7rem;"),
        "a toast would cover the footer"
    );
}

#[test]
fn a_preference_form_adds_no_box_and_its_cards_take_the_phone_row() {
    assert!(
        STYLESHEET.contains("  .fui-preference-form {\n    display: contents;"),
        "the control and its Save button lay out in the row's control column"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-setting-control:has(.fui-choice-cards),\n    .fui-choice-cards {\n      width: 100%;"
        ),
        "the preview cards reach the phone row through the preference's form"
    );
}

#[test]
fn reduce_motion_chosen_in_your_preferences_stills_the_page_as_the_system_does() {
    // NOTE: WCAG 2.2 SC 2.3.3: motion from an interaction can be turned off.
    assert!(
        STYLESHEET.contains(
            "  :root[data-motion=\"reduce\"] *,\n  :root[data-motion=\"reduce\"] *::before,\n  :root[data-motion=\"reduce\"] *::after {\n    animation-duration: 0.01ms !important;"
        ),
        "the preference reduces motion"
    );
    assert!(
        STYLESHEET.contains(
            "  :root[data-motion=\"reduce\"] .fui-setting-row .fui-setting-saved {\n    animation: none;"
        ),
        "a cut fade would hide Saved at once"
    );
    assert!(
        STYLESHEET.contains(
            "  @media (prefers-reduced-motion: reduce) {\n    .fui-setting-row .fui-setting-saved {\n      animation: none;"
        ),
        "a cut fade would hide Saved at once"
    );
}

/// The rules of the stylesheet's forced colours block.
fn forced_colours() -> &'static str {
    let start = STYLESHEET
        .find("@media (forced-colors: active) {")
        .expect("a forced colours block");
    STYLESHEET.get(start..).expect("the block's text")
}

#[test]
fn every_mark_drawn_by_a_ground_or_a_shadow_is_drawn_again_under_forced_colours() {
    // NOTE: CSS Color Adjustment Module Level 1 § 3.1: forced colours replace each colour and drop box shadows and gradient images.
    let forced = forced_colours();
    for (rule, what) in [
        (
            "    .fui-pill::before,\n    .fui-dot,\n    .fui-editor-dirty::before,\n    .is-current > .fui-lifecycle-dot::after {\n      forced-color-adjust: none;\n      background: CanvasText;",
            "a state's shape in the status pill, the health dot and the editor's dirty mark",
        ),
        (
            "    .fui-dot.fui-tone-neutral,\n    .fui-inactive-mark::before {\n      forced-color-adjust: none;\n      border: 1.5px solid CanvasText;",
            "the hollow ring, an inset shadow elsewhere",
        ),
        (
            ":is(.fui-check > input.fui-switch, input.fui-switch)::before {\n      forced-color-adjust: none;\n      background: ButtonText;",
            "the switch's knob",
        ),
        (
            ":is(.fui-check > input.fui-switch, input.fui-switch):checked {\n      border-color: Highlight;\n      background: Highlight;",
            "the switch's on state",
        ),
        (
            "    .fui-segment:is([aria-pressed=\"true\"], [aria-current=\"true\"], [aria-checked=\"true\"], :has(> input:checked)),\n    .fui-chip-check:has(:checked) {\n      forced-color-adjust: none;\n      border-color: Highlight;\n      background: Highlight;\n      color: HighlightText;",
            "a pressed segment, the theme choice and a chosen radio among them",
        ),
        (
            "    .fui-select:not([multiple], [size]),\n    .fui-combobox-input {\n      forced-color-adjust: none;",
            "the select's chevron, a gradient image",
        ),
        (
            "    .fui-nav-entry[aria-current=\"page\"]:not(:focus-visible),",
            "the current page in the sidebar",
        ),
        (
            "    .fui-tab:is([aria-current=\"page\"], [aria-current=\"true\"], [aria-selected=\"true\"]):not(:focus-visible),",
            "the current tab",
        ),
        (
            "    .fui-table tbody tr[aria-current=\"true\"] > :first-child,",
            "a current row's edge, an inset shadow elsewhere",
        ),
        (
            "    .fui-worklist .fui-table tbody tr:is(:focus-visible, :has(.fui-row-link:focus-visible)) {\n      outline: 2px solid CanvasText;",
            "the worklist row's focus ring, a shadow elsewhere",
        ),
        (
            "    .fui-swatch,\n    .fui-chart-line,\n    .fui-chart-gap,\n    .fui-chart-point {\n      forced-color-adjust: none;",
            "a series keeps the colour its swatch names",
        ),
        (
            "    .fui-funnel-swatch {\n      forced-color-adjust: none;\n      border: 1px solid CanvasText;",
            "the run report's swatches, a ground elsewhere",
        ),
        (
            "    .fui-chart-label,\n    .fui-score-mark-label,\n    .fui-score-point-label {\n      fill: CanvasText;\n      stroke: Canvas;",
            "the chart's labels on the canvas, not on a halo of the card's ground",
        ),
    ] {
        assert!(
            forced.contains(rule),
            "{what} is drawn under forced colours"
        );
    }
}

#[test]
fn a_chart_hides_by_container_query_only_the_x_labels_its_width_has_no_room_for() {
    assert!(
        STYLESHEET.contains("  .fui-chart-frame {\n    container-type: inline-size;"),
        "the chart's frame is the container its labels query"
    );
    for width in ["15", "20", "25", "30", "40", "50", "60", "80"] {
        assert!(
            STYLESHEET.contains(&format!(
                "  @container (width < {width}rem) {{\n    .fui-chart-tick-{width} {{\n      display: none;"
            )),
            "a label thinned below {width} rem is hidden there"
        );
    }
}

#[test]
fn a_long_node_name_and_a_long_score_field_wrap_inside_their_row() {
    // NOTE: WCAG 2.2 SC 1.4.10: content reflows at 320 px with no two-dimensional scroll.
    assert!(
        STYLESHEET.contains(
            "  .fui-fanout-line > code {\n    min-width: 0;\n    overflow-wrap: anywhere;"
        ),
        "a node's name never runs into the next node"
    );
    assert!(
        STYLESHEET.contains("  .fui-score-label {\n    min-width: 0;\n    font-weight: 500;\n    overflow-wrap: anywhere;"),
        "a field's long word never runs over its bar"
    );
}

#[test]
fn the_toast_region_stays_above_the_decision_bar_and_scrolls() {
    // Shell edge audit, row 3; WCAG 2.2 SC 1.4.10 and SC 2.4.11: toasts never cover the decisions.
    let region = rule("  .fui-toasts");
    assert!(
        region.contains("overflow-y: auto;")
            && region.contains("align-content: safe end;")
            && region.contains("pointer-events: none;"),
        "{region}"
    );
    assert!(
        STYLESHEET.contains("  .fui-toasts .fui-toast {\n    pointer-events: auto;"),
        "a toast takes the pointer"
    );
    assert!(
        STYLESHEET.contains("    .fui-dock {\n      anchor-name: --fui-dock;"),
        "the decision bar names its anchor"
    );
    assert!(
        rule("    :root:has(.fui-dock) .fui-toasts")
            .contains("position-anchor: --fui-dock;\n      inset-block-end: max(var(--space-loose), calc(anchor(top, var(--space-loose))"),
        "the region's foot is the decision bar's top"
    );
    assert!(
        STYLESHEET
            .contains("  .fui-toast:has(> .fui-toast-close:not([open])) {\n    display: none;"),
        "a toast closed with no script hides"
    );
    assert!(
        rule("  .fui-toast code").contains("overflow-wrap: anywhere;"),
        "a long file name in a toast wraps"
    );
}

#[test]
fn the_navigation_wraps_a_long_word_and_the_rail_shows_its_tooltip_on_focus() {
    // Shell edge audit, rows 11 to 13; WCAG 2.2 SC 1.4.10 and SC 1.4.13.
    let word = rule("  .fui-nav-word");
    assert!(
        word.contains("min-width: 0;")
            && word.contains("overflow-wrap: anywhere;")
            && word.contains("hyphens: auto;"),
        "{word}"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-sidebar-rail:not([data-tips=\"off\"]) .fui-nav-entry:is(:hover, :focus-visible) > .fui-tooltip {\n      display: block;"
        ),
        "the rail's tooltip shows on keyboard focus, until Escape"
    );
    assert!(
        STYLESHEET.contains(
            "    .fui-sidebar-rail .fui-nav-count.is-long > .fui-nav-count-whole {\n      display: none;"
        ),
        "a long count leaves the rail's badge"
    );
    assert!(
        rule("  .fui-sidebar[open]").contains("max-width: none;"),
        "the open drawer is a modal dialog the full height of the screen"
    );
}

/// The declarations of the first rule of `STYLESHEET` whose selector text
/// ends with `selector`.
fn declarations(selector: &str) -> &'static str {
    let at = STYLESHEET
        .find(&format!("{selector} {{\n"))
        .unwrap_or_else(|| panic!("no rule for {selector}"));
    let body = &STYLESHEET[at..];
    &body[..body.find('}').expect("a closed rule")]
}

#[test]
fn a_long_word_wraps_in_a_heading_a_label_and_a_table_header() {
    // WCAG 2.2 SC 1.4.10: a German or Dutch compound never widens the page.
    for selector in ["  h4", "  .fui-label", "  .fui-table thead th"] {
        let rule = declarations(selector);
        assert!(rule.contains("hyphens: auto;"), "{selector}: {rule}");
        assert!(!rule.contains("white-space: nowrap;"), "{selector}: {rule}");
    }
    // break-word breaks only a word that cannot fit its line alone; anywhere
    // would also split words that fit, inside a flex or grid row.
    for selector in ["  h4", "  .fui-label"] {
        let rule = declarations(selector);
        assert!(
            rule.contains("overflow-wrap: break-word;"),
            "{selector}: {rule}"
        );
        assert!(
            !rule.contains("overflow-wrap: anywhere;"),
            "{selector}: {rule}"
        );
    }
}

#[test]
fn a_number_never_breaks_between_its_digits() {
    let rule =
        declarations("  :is(.fui-list-count, .fui-tab-count, .fui-nav-count, .fui-nav-badge)");
    assert!(rule.contains("white-space: nowrap;"), "{rule}");
    assert!(STYLESHEET.contains("  .fui-num,\n  .fui-num > span,\n"));
}

#[test]
fn only_a_table_a_console_bounds_gives_up_its_scroll_box_from_64rem() {
    // WCAG 2.2 SC 1.4.10: a query's result set scrolls in its card at every
    // width; only a bounded table drops the box so its header can stick.
    let wide = STYLESHEET
        .find("  @media (width >= 64rem) {\n    .fui-table-scroll:not(.fui-table-wide) {\n      overflow: visible;")
        .expect("the bounded table's rule");
    let media = &STYLESHEET[wide..];
    let media = &media[..media.find("\n  }\n").expect("the media block's end")];
    assert!(
        !media.contains("\n    .fui-table-scroll {"),
        "every table drops its box: {media}"
    );
    assert!(
        media.contains(
            ".fui-table-scroll:not(.fui-table-wide) .fui-table thead th {\n      position: sticky;"
        ),
        "{media}"
    );
}

#[test]
fn a_cut_line_gives_a_bounded_tables_column_no_width_beside_the_sidebar() {
    // WCAG 2.2 SC 1.4.10: below 80rem a canonical under a name no longer
    // holds its column at 28rem and pushes the table past its card.
    assert!(STYLESHEET.contains(
        "  @media (64rem <= width < 80rem) {\n    .fui-table-scroll:not(.fui-table-wide) .fui-table .fui-truncate {\n      width: 0;\n      min-width: 100%;"
    ));
}

#[test]
fn a_choice_legend_is_a_field_label_and_outranks_the_group_heading() {
    // CSS Selectors 4, specificity: equal weight, so the later rule wins.
    let label = rule("  .fui-fieldset.fui-choice > legend");
    for property in [
        "border-bottom: 0;",
        "font-size: var(--text-body);",
        "font-weight: 500;",
    ] {
        assert!(label.contains(property), "missing {property}: {label}");
    }
    let heading = STYLESHEET
        .find("  .fui-fieldset > legend:not(.fui-sr-only) {")
        .expect("the group heading rule");
    let choice = STYLESHEET
        .find("  .fui-fieldset.fui-choice > legend {")
        .expect("the choice label rule");
    assert!(
        heading < choice,
        "the label rule comes after the heading rule"
    );
}

#[test]
fn a_menu_segment_keeps_its_word_on_one_line() {
    let segment = rule("  .fui-menu .fui-segmented > .fui-segment");
    for property in [
        "min-width: max-content;",
        "white-space: nowrap;",
        "overflow-wrap: normal;",
    ] {
        assert!(segment.contains(property), "missing {property}: {segment}");
    }
}

#[test]
fn a_confirmation_with_fields_keeps_the_dialogs_rhythm_inside_its_form() {
    let rhythm = "  .fui-dialog > * + *,\n  .fui-dialog > .fui-dialog-form > * + * {";
    assert!(
        STYLESHEET.contains(rhythm),
        "the question, the sentence, the fields and the actions keep their spacing inside the form"
    );
}

#[test]
fn a_long_word_wraps_in_a_banner_a_notice_a_settings_description_and_its_source() {
    // WCAG 2.2 SC 1.4.10: a URL or a long compound never runs past the screen at 320 px.
    let banner = rule("  .fui-banner > p");
    assert!(
        banner.contains("min-width: 0;") && banner.contains("overflow-wrap: break-word;"),
        "the banner's text shrinks below its longest word and wraps it: {banner}"
    );
    let notice = rule("  .fui-notice > div");
    assert!(
        notice.contains("min-width: 0;") && notice.contains("overflow-wrap: break-word;"),
        "a notice's body wraps as the banner's text does: {notice}"
    );
    let link = rule("  :is(.fui-notice, .fui-alert) a");
    assert!(
        link.contains("display: inline-block;")
            && link.contains("max-width: 100%;")
            && !link.contains("nowrap"),
        "a notice's link moves whole to the next line and wraps only when wider: {link}"
    );
    let description = rule("  .fui-setting-desc");
    assert!(
        description.contains("overflow-wrap: break-word;"),
        "{description}"
    );
    let source = rule("  .fui-setting-source");
    assert!(
        source.contains("min-width: 0;")
            && source.contains("overflow-wrap: anywhere;")
            && !source.contains("nowrap"),
        "{source}"
    );
}

#[test]
fn a_copy_buttons_glyph_and_words_are_spaced_as_any_buttons_in_both_states() {
    let state = rule("  .fui-copy > .fui-button > :is([data-fui-ready], [data-fui-done])");
    assert!(
        state.contains("display: contents;"),
        "a state's glyph and words take the button's own row and gap: {state}"
    );
    assert!(
        rule("  [hidden]").contains("display: none !important;"),
        "the state a button does not show stays hidden"
    );
    for (button, leading) in [
        ("  .fui-copy > .fui-button", "var(--leading-body)"),
        ("  .fui-copy > .fui-button-small", "var(--leading-small)"),
    ] {
        let glyph = rule(&format!(
            "{button} > :is([data-fui-ready], [data-fui-done]) > .fui-icon"
        ));
        assert!(
            glyph.contains(&format!("height: {leading};")),
            "a state's glyph sits in a box one line tall, as a button's own: {glyph}"
        );
    }
}
