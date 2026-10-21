# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The FerroTERM console mockups' page table: each slug of pages.py and the
function that draws it, read by design/tools/build.py."""
import pg_browse
import pg_console
import pg_edit
import pg_ops
import pg_plan
import pg_res
import pg_shell

MODULES = [pg_shell, pg_res, pg_edit, pg_browse, pg_ops, pg_console, pg_plan]
SPECIAL = {
    "toast-success": lambda: pg_shell.page_toast("ok"), "toast-warning": lambda: pg_shell.page_toast("warn"), "toast-error": lambda: pg_shell.page_toast("bad"),
    "topbar-results": lambda: pg_shell.topbar_state_page("focus-filled", "respir", pg_shell.results_list("some")),
    "topbar-no-result": lambda: pg_shell.topbar_state_page("focus-filled", "http://example.org/fhir/CodeSystem/x", pg_shell.results_list("none")),
    "sync-runs": pg_plan.page_sync_runs, "sync-run": pg_plan.page_sync_run, "sync-run-findings": pg_plan.page_sync_run_findings,
    "sync-run-now": pg_plan.page_sync_run_now, "reload": pg_plan.page_reload, "delete": pg_plan.page_delete,
}


def page_fn(slug):
    if slug in SPECIAL:
        return SPECIAL[slug]
    name = "page_" + slug.replace("-", "_")
    for m in MODULES:
        if hasattr(m, name):
            return getattr(m, name)
    raise KeyError(f"no function for {slug} ({name})")
