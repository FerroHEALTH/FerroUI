# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The FerroBRIDGE console mockups' page table: each slug of pages.py and the
function that draws it, read by design/tools/build.py."""
import pg_config as c
import pg_edit as ed
import pg_faces as f
import pg_maps as m
import pg_people as p
import pg_shell as s
import pg_tools as t

PAGES = {
    "dashboard": s.page_dashboard, "dashboard-attention": s.page_dashboard_attention, "flow-hover": s.page_flow_hover, "flow-hover-pulse": s.page_flow_hover_pulse, "flow-help": s.page_flow_help,
    "dashboard-loading": s.page_dashboard_loading, "dashboard-empty": s.page_dashboard_empty, "dashboard-operator-off": s.page_dashboard_operator_off,
    "bridge-unreachable": s.page_bridge_unreachable, "dashboard-dev": s.page_dashboard_dev,
    "sidebar-rail": s.page_sidebar_rail, "sidebar-drawer": s.page_sidebar_drawer,
    "user-menu": s.page_user_menu, "user-menu-dark": s.page_user_menu_dark, "user-menu-signout": s.page_user_menu_signout,
    "toast-success": lambda: s.page_toast("ok"), "toast-warning": lambda: s.page_toast("warn"), "toast-error": lambda: s.page_toast("bad"),
    "forbidden": s.page_forbidden, "not-found": s.page_not_found, "unreadable": s.page_unreadable, "about": s.page_about,
    "focus-rings": s.page_focus_rings, "menu-states": s.page_menu_states, "action-group-states": s.page_action_group_states, "shortcuts": s.page_shortcuts,
    "search-states": s.page_search_states,
    "topbar-results": lambda: s.topbar_state_page("focus-filled", "weight", s.search_results_list("some")),
    "topbar-no-result": lambda: s.topbar_state_page("focus-filled", "Immunization", s.search_results_list("none")),
    "mappings": m.page_mappings, "mappings-hover": m.page_mappings_hover, "mappings-menu": m.page_mappings_menu, "mappings-omocl": m.page_mappings_omocl,
    "mappings-templates": m.page_mappings_templates, "mappings-files": m.page_mappings_files, "mappings-files-filtered": m.page_mappings_files_filtered,
    "mappings-files-no-match": m.page_mappings_files_no_match, "mappings-diagnostics": m.page_mappings_diagnostics, "mappings-more": m.page_mappings_more,
    "mappings-reload": m.page_mappings_reload, "mappings-reloaded": m.page_mappings_reloaded, "mappings-reload-refused": m.page_mappings_reload_refused,
    "mappings-loading": m.page_mappings_loading, "mappings-unreachable": m.page_mappings_unreachable, "mappings-forbidden": m.page_mappings_forbidden,
    "mappings-empty": m.page_mappings_empty, "mappings-import": m.page_mappings_import,
    "file": m.page_file, "file-line": m.page_file_line, "file-program": m.page_file_program, "file-more": m.page_file_more, "file-provisioned": m.page_file_provisioned,
    "file-model": m.page_file_model, "file-omocl": m.page_file_omocl, "file-not-utf8": m.page_file_not_utf8,
    "template": m.page_template,
    "edit": ed.page_edit, "edit-checking": ed.page_edit_checking, "edit-check-clean": ed.page_edit_check_clean, "edit-check-errors": ed.page_edit_check_errors,
    "edit-save": ed.page_edit_save, "edit-save-confirm": ed.page_edit_save_confirm, "edit-saving": ed.page_edit_saving, "edit-saved": ed.page_edit_saved,
    "edit-refused": ed.page_edit_refused, "edit-conflict": ed.page_edit_conflict, "edit-provisioned": ed.page_edit_provisioned,
    "edit-provisioned-409": ed.page_edit_provisioned_409, "edit-new": ed.page_edit_new, "edit-new-invalid": ed.page_edit_new_invalid,
    "edit-reason-invalid": ed.page_edit_reason_invalid,
    "playground": t.page_playground, "playground-invalid": t.page_playground_invalid, "playground-running": t.page_playground_running,
    "playground-answered": t.page_playground_answered, "playground-refused": t.page_playground_refused, "playground-off": t.page_playground_off,
    "playground-example": t.page_playground_example, "playground-toopenehr": t.page_playground_toopenehr, "playground-omop": t.page_playground_omop,
    "playground-omop-answered": t.page_playground_omop_answered, "playground-hl7v2": t.page_playground_hl7v2, "playground-hl7v2-answered": t.page_playground_hl7v2_answered,
    "validate": t.page_validate, "validate-issues": t.page_validate_issues, "validate-clean": t.page_validate_clean, "validate-refused": t.page_validate_refused,
    "facade": f.page_facade, "facade-more": f.page_facade_more, "facade-capability": f.page_facade_capability, "facade-security": f.page_facade_security,
    "facade-settings": f.page_facade_settings, "facade-settings-hint": f.page_facade_settings_hint, "facade-off": f.page_facade_off,
    "hl7v2": f.page_hl7v2, "hl7v2-conceptmaps": f.page_hl7v2_conceptmaps, "hl7v2-settings": f.page_hl7v2_settings, "hl7v2-off": f.page_hl7v2_off,
    "omop": f.page_omop, "omop-runs": f.page_omop_runs, "omop-run": f.page_omop_run, "omop-run-refusals": f.page_omop_run_refusals, "omop-run-failed": f.page_omop_run_failed,
    "omop-runs-empty": f.page_omop_empty, "omop-settings": f.page_omop_settings,
    "connections": f.page_connections, "connections-attention": f.page_connections_attention, "connection": f.page_connection,
    "connection-settings": f.page_connection_settings, "connection-terminology": f.page_connection_terminology,
    "connection-terminology-settings": f.page_connection_terminology_settings, "connection-cdm": f.page_connection_cdm, "connection-cdm-settings": f.page_connection_cdm_settings,
    "configuration": c.page_configuration, "configuration-file-mode-dialog": c.page_configuration_file_mode_dialog, "configuration-mappings": c.page_configuration_mappings,
    "configuration-core": c.page_configuration_core, "configuration-console": c.page_configuration_console, "configuration-key-hint": c.page_configuration_key_hint,
    "configuration-actions": c.page_configuration_actions,
    "preferences": p.page_preferences, "preferences-language": p.page_preferences_language, 
    "preferences-saved": p.page_preferences_saved, "preferences-keyboard": p.page_preferences_keyboard, "preferences-account": p.page_preferences_account,
    "preferences-reset": p.page_preferences_reset, "preferences-dark": p.page_preferences_dark,
    "sign-in": p.page_sign_in, "sign-in-redirecting": p.page_sign_in_redirecting, "sign-in-refused": p.page_sign_in_refused, "sign-in-rejected": p.page_sign_in_rejected,
    "sign-in-expired": p.page_sign_in_expired, "sign-in-signed-out": p.page_sign_in_signed_out, "sign-in-unavailable": p.page_sign_in_unavailable, "sign-in-full": p.page_sign_in_full,
}
