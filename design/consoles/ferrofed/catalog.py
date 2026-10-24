# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The FerroFED console mockups' page table: each slug of pages.py and the
function that draws it, read by design/tools/build.py."""
import pg_config as c
import pg_fed as f
import pg_orgs as o
import pg_services as v
import pg_topo as t
import pg_people as p
import pg_query as q
import pg_shell as s

PAGES = {
    "dashboard": s.page_dashboard, "dashboard-loading": s.page_dashboard_loading, "dashboard-empty": s.page_dashboard_empty,
    "gateway-unreachable": s.page_gateway_unreachable, "dashboard-dev": s.page_dashboard_dev,
    "sidebar-rail": s.page_sidebar_rail, "sidebar-drawer": s.page_sidebar_drawer,
    "user-menu": s.page_user_menu, "user-menu-dark": s.page_user_menu_dark, "user-menu-signout": s.page_user_menu_signout,
    "toast-success": lambda: s.page_toast("ok"), "toast-warning": lambda: s.page_toast("warn"), "toast-error": lambda: s.page_toast("bad"),
    "forbidden": s.page_forbidden, "not-found": s.page_not_found, "unreadable": s.page_unreadable, "no-admin": s.page_no_admin,
    "about": s.page_about, "focus-rings": s.page_focus_rings, "menu-states": s.page_menu_states, "action-group-states": s.page_action_group_states, "node-more": o.page_node_more, "shortcuts": s.page_shortcuts,
    "search-states": s.page_search_states,
    "topbar-default": lambda: s.topbar_state_page(), "topbar-hover": lambda: s.topbar_state_page("hover"),
    "topbar-focused": lambda: s.topbar_state_page("focus"), "topbar-filled": lambda: s.topbar_state_page(None, "Testkliniek"),
    "topbar-busy": lambda: s.topbar_state_page("busy", "Testkliniek"), "topbar-disabled": lambda: s.topbar_state_page("disabled"),
    "topbar-results": lambda: s.topbar_state_page("focus-filled", "b", s.search_results_list("some")),
    "topbar-no-result": lambda: s.topbar_state_page("focus-filled", "node-x", s.search_results_list("none")),
    "topbar-system-id": lambda: s.topbar_state_page("focus-filled", "legacy-zvs", s.search_results_list("system")),
    "organisations": o.page_organisations, "organisations-hover": o.page_organisations_hover, "organisations-menu": o.page_organisations_menu,
    "organisations-filtered": o.page_organisations_filtered, "organisations-no-match": o.page_organisations_no_match, "organisations-loading": o.page_organisations_loading,
    "organisations-unreachable": o.page_organisations_unreachable, "organisations-nodes": o.page_organisations_nodes, "organisations-large": o.page_organisations_large,
    "organisations-empty": o.page_organisations_empty, "organisations-directory": o.page_organisations_directory, "organisations-file-mode": o.page_organisations_file_mode,
    "organisation": o.page_organisation, "organisation-nodes": o.page_organisation_nodes, "organisation-requests": o.page_organisation_requests,
    "organisation-audit": o.page_organisation_audit, "organisation-settings": o.page_organisation_settings, "organisation-settings-edit": o.page_organisation_settings_edit,
    "organisation-degraded": o.page_organisation_degraded, "organisation-empty": o.page_organisation_empty, "organisation-actions": o.page_organisation_actions,
    "organisation-exclude": o.page_organisation_exclude, "organisation-new": o.page_organisation_new,
    "node": o.page_node, "node-endpoints": lambda: o.page_node_endpoints(), "node-endpoint-menu": o.page_node_endpoint_menu, "node-routes": o.page_node_routes, "node-routes-add": o.page_node_routes_add, "node-routes-filtered": o.page_node_routes_filtered, "node-routes-review": o.page_node_routes_review, "node-requests": o.page_node_requests,
    "node-audit": o.page_node_audit, "node-settings": o.page_node_settings, "node-settings-edit": o.page_node_settings_edit, "node-degraded": o.page_node_degraded,
    "node-probing": o.page_node_probing, "node-exclude": o.page_node_exclude, "node-remove": o.page_node_remove, "node-new": o.page_node_new,
    "node-new-invalid": o.page_node_new_invalid, "node-edit": o.page_node_edit, "endpoint-new": o.page_endpoint_new,
    "endpoint-edit": o.page_endpoint_edit, "endpoint-exclude": o.page_endpoint_exclude, "endpoint-remove": o.page_endpoint_remove,
    "endpoint-credentials": c.page_credential_new, "endpoint-credentials-oauth": c.page_credential_oauth, "endpoint-credentials-nuts": c.page_credential_nuts,
    "endpoint-credentials-invalid": c.page_credential_invalid, "secret-field-states": c.page_secret_states,
    "services": v.page_services, "services-attention": v.page_services_attention, "services-setup": v.page_services_setup, "services-menu": v.page_services_menu, "services-localization-edit": v.page_services_localization_edit, "service": v.page_service,
    "service-connection": v.page_service_connection, "service-connection-edit": v.page_service_connection_edit, "service-requests": v.page_service_requests,
    "service-audit": v.page_service_audit, "service-new": v.page_service_new,
    "topology": t.page_topology, "topology-controls": t.page_topology_controls, "topology-degraded": t.page_topology_degraded, "topology-selected": t.page_topology_selected, "topology-hover": t.page_topology_hover, "topology-hover-pulse": t.page_topology_hover_pulse, "topology-help": t.page_topology_help, "topology-search": t.page_topology_search,
    "topology-filtered": t.page_topology_filtered, "topology-collapsed": t.page_topology_collapsed, "topology-large": t.page_topology_large,
    "topology-list": t.page_topology_list, "topology-traffic": t.page_topology_traffic, "topology-phone-map": t.page_topology_phone_map,
    "topology-loading": t.page_topology_loading, "topology-empty": t.page_topology_empty, "topology-unreachable": t.page_topology_unreachable,
    "integrity": f.page_integrity, "integrity-resolved": f.page_integrity_resolved, "integrity-empty": f.page_integrity_empty,
    "integrity-menu": f.page_integrity_menu, "integrity-clear": f.page_integrity_clear, "integrity-clear-invalid": f.page_integrity_clear_invalid,
    "integrity-clear-busy": f.page_integrity_clear_busy, "integrity-cleared": f.page_integrity_cleared, "integrity-clear-refused": f.page_integrity_clear_refused,
    "query": q.page_query, "query-stored": q.page_query_stored, "query-options": q.page_query_options, "query-invalid": q.page_query_invalid,
    "query-running": q.page_query_running, "query-complete": q.page_query_complete, "query-partial": q.page_query_partial,
    "query-failed": q.page_query_failed, "query-refused": q.page_query_refused, "query-raw": q.page_query_raw, "query-no-rows": q.page_query_no_rows,
    "stored-queries": q.page_stored, "stored-queries-filtered": q.page_stored_filtered, "stored-queries-no-match": q.page_stored_no_match,
    "stored-queries-loading": q.page_stored_loading, "stored-query": q.page_stored_query, "stored-query-missing": q.page_stored_query_missing,
    "stored-query-repair": q.page_stored_repair, "stored-query-repaired": q.page_stored_repaired,
    "stored-new": q.page_stored_new, "stored-new-confirm": q.page_stored_new_confirm, "stored-new-held": q.page_stored_new_held,
    "stored-new-reserved": q.page_stored_new_reserved, "stored-new-literal": q.page_stored_new_literal, "stored-new-not-aql": q.page_stored_new_not_aql, "stored-new-read-only": q.page_stored_new_read_only,
    "self-description": f.page_self_description, "self-description-capabilities": f.page_self_description_capabilities, "self-description-areas": f.page_self_description_areas, "self-description-limits": f.page_self_description_limits, "self-description-endpoints": f.page_self_description_endpoints,
    "configuration": c.page_config_settings, "configuration-fanout": c.page_configuration_fanout, "configuration-key-hint": c.page_configuration_key_hint, "configuration-consent": c.page_configuration_consent, "configuration-stored": c.page_configuration_stored, "configuration-access": c.page_configuration_access, "configuration-fhir": c.page_configuration_fhir, "configuration-auth": c.page_configuration_auth, "configuration-mode-dialog": c.page_configuration_mode_dialog, "configuration-actions": c.page_configuration_actions,
    "configuration-loading": c.page_configuration_loading, "configuration-export-done": c.page_configuration_export_done,
    "config-settings-edit": c.page_config_settings_edit, "config-settings-review": c.page_config_settings_review, "config-restart": c.page_config_restart,
    "config-conflict": c.page_config_conflict, "config-rollback": c.page_config_rollback, "config-replica-refused": c.page_config_replica_refused,
    "config-review": c.page_config_review, "config-apply": c.page_config_apply, "config-applied": c.page_config_applied, "config-pending": c.page_config_pending,
    "config-review-approval": c.page_credential_review, "config-awaiting-approval": c.page_config_awaiting_approval, "config-approver": c.page_config_approver,
    "config-approve-dialog": c.page_config_approve_dialog, "config-approved": c.page_config_approved, "config-apply-trial": c.page_config_apply_trial,
    "config-trial": c.page_config_trial, "config-confirmed": c.page_config_confirmed, "config-rolled-back": c.page_config_rolled_back,
    "config-lifecycle-states": c.page_lifecycle_states,
    "configuration-history": c.page_configuration_history, "configuration-history-filtered": c.page_configuration_history_filtered,
    "configuration-history-range": c.page_configuration_history_range, "configuration-compare": c.page_configuration_compare,
    "configuration-import": c.page_configuration_import, "configuration-import-refused": c.page_configuration_import_refused,
    "configuration-file-mode": c.page_configuration_file_mode, "configuration-file-mode-dialog": c.page_configuration_file_mode_dialog,
    "configuration-file-mode-history": c.page_configuration_file_mode_history,
    "configuration-file-mode-auth": c.page_configuration_file_mode_auth,
    "preferences": p.page_preferences, "preferences-language": p.page_preferences_language, "preferences-saved": p.page_preferences_saved,
    "preferences-reset": p.page_preferences_reset, "preferences-keyboard": p.page_preferences_keyboard, "preferences-dark": p.page_preferences_dark,
    "sign-in": p.page_sign_in, "sign-in-redirecting": p.page_sign_in_redirecting, "sign-in-refused": p.page_sign_in_refused,
    "sign-in-rejected": p.page_sign_in_rejected, "sign-in-expired": p.page_sign_in_expired, "sign-in-signed-out": p.page_sign_in_signed_out,
    "sign-in-unavailable": p.page_sign_in_unavailable, "sign-in-full": p.page_sign_in_full,
}


def route_pages():
    """Every route-only page (r-…): each other object's tabs."""
    out = {}
    for mod in (o, v, q, c):
        out.update(mod.route_pages())
    return out
