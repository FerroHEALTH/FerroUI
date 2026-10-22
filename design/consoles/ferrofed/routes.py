# SPDX-FileCopyrightText: Cadasto B.V.
# SPDX-License-Identifier: BUSL-1.1
"""Where each object's tab lives. Every organisation, node, service and
stored query has its own route for each of its tabs, so a tab, a row or a
sidebar entry never opens another object's page. The captured screens keep
their slugs; every other route is a route-only page (`r-…`), built and
link-checked with the rest but not captured or listed on the contact sheet."""

ORG_TABS = ["overview", "nodes", "requests", "audit", "settings"]
NODE_TABS = ["overview", "endpoints", "routes", "requests", "audit", "settings"]
SVC_TABS = ["overview", "connection", "requests", "audit"]

CAPTURED = {
    ("org", "zvs", "overview"): "organisation", ("org", "zvs", "nodes"): "organisation-nodes",
    ("org", "zvs", "requests"): "organisation-requests", ("org", "zvs", "audit"): "organisation-audit",
    ("org", "zvs", "settings"): "organisation-settings",
    ("org", "tko", "overview"): "organisation-degraded",
    ("org", "zbt", "overview"): "organisation-empty",
    ("node", "node-a", "overview"): "node", ("node", "node-a", "endpoints"): "node-endpoints",
    ("node", "node-a", "routes"): "node-routes", ("node", "node-a", "requests"): "node-requests",
    ("node", "node-a", "audit"): "node-audit",
    ("node", "node-a-lab", "settings"): "node-settings",
    ("node", "node-b", "overview"): "node-degraded",
    ("svc", "mitz", "overview"): "service", ("svc", "mitz", "connection"): "service-connection",
    ("svc", "mitz", "requests"): "service-requests", ("svc", "mitz", "audit"): "service-audit",
    ("sq", "vital_signs", "aql"): "stored-query",
}


def slug(kind, key, tab="overview"):
    if (kind, key, tab) in CAPTURED:
        return CAPTURED[(kind, key, tab)]
    base = f"r-{kind}-{key}".replace("_", "-").replace(".", "-")
    return base if tab in ("overview", "aql") else f"{base}-{tab}"


def href(kind, key, tab="overview"):
    return slug(kind, key, tab) + ".html"


def org_href(slug_, tab="overview"):
    return href("org", slug_, tab)


def node_href(nid, tab="overview"):
    return href("node", nid, tab)


def svc_href(key, tab="overview"):
    return href("svc", key, tab)


def sq_href(short, tab="aql"):
    return href("sq", short, tab)


def compare_href(n):
    return "configuration-compare.html" if str(n) == "5" else f"r-compare-{n}.html"
