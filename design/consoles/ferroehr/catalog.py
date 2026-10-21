# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""The FerroEHR viewer mockups' page table: each slug of pages.py and the
function that draws it, read by design/tools/build.py."""
import pg_comp
import pg_console
import pg_def
import pg_ehr
import pg_party
import pg_plan
import pg_query
import pg_shell

MODULES = [pg_shell, pg_ehr, pg_comp, pg_party, pg_def, pg_query, pg_console, pg_plan]
SPECIAL = {}


def page_fn(slug):
    if slug in SPECIAL:
        return SPECIAL[slug]
    name = "page_" + slug.replace("-", "_")
    for m in MODULES:
        if hasattr(m, name):
            return getattr(m, name)
    raise KeyError(f"no function for {slug} ({name})")
