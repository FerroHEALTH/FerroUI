// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The topology's state as the federation page's query names it.

use std::collections::BTreeSet;

use ferrohealth_ui::view::flow::FlowState;
use ferrohealth_ui::view::topology::TopologyView;

use crate::visit::Visit;

/// The state of the topology the query names.
#[derive(Debug, Clone, Default)]
pub(super) struct Asked {
    pub(super) view: TopologyView,
    pub(super) look: Option<String>,
    pub(super) collapse: BTreeSet<String>,
    pub(super) all_folded: bool,
    show: Option<BTreeSet<String>>,
    pub(super) large: Option<u8>,
    pub(super) minimap: bool,
}

impl Asked {
    /// The query of this state, with `change` applied, as a link's query.
    pub(super) fn query(&self, change: impl FnOnce(&mut Self)) -> String {
        let mut next = self.clone();
        change(&mut next);
        let mut parts = Vec::new();
        match next.view {
            TopologyView::Auto => {}
            TopologyView::Map => parts.push(String::from("view=map")),
            TopologyView::List => parts.push(String::from("view=list")),
        }
        if let Some(look) = &next.look {
            parts.push(format!("look={look}"));
        }
        if next.all_folded {
            parts.push(String::from("collapse=all"));
        } else if !next.collapse.is_empty() {
            let folded: Vec<&str> = next.collapse.iter().map(String::as_str).collect();
            parts.push(format!("collapse={}", folded.join(",")));
        }
        if let Some(show) = &next.show {
            parts.extend(show.iter().map(|state| format!("show={state}")));
        }
        if let Some(large) = next.large {
            parts.push(format!("large={large}"));
        }
        if next.minimap {
            parts.push(String::from("minimap=on"));
        }
        if parts.is_empty() {
            String::new()
        } else {
            format!("?{}", parts.join("&"))
        }
    }

    /// Whether the group `id` is folded.
    pub(super) fn folded(&self, id: &str) -> bool {
        self.all_folded || self.collapse.contains(id)
    }

    /// Whether a card in `state` is shown.
    pub(super) fn shows(&self, state: FlowState) -> bool {
        self.show
            .as_ref()
            .is_none_or(|show| show.contains(state.key()))
    }
}

/// The state the page's query names.
pub(super) fn asked(v: &Visit) -> Asked {
    let query = &v.query;
    let view = match query.get_str("view") {
        Some("map") => TopologyView::Map,
        Some("list") => TopologyView::List,
        _ => TopologyView::Auto,
    };
    let collapse = query.get_str("collapse").unwrap_or_default();
    let show: Vec<String> = query
        .pairs()
        .iter()
        .filter(|(name, _)| name == "show")
        .map(|(_, state)| state.clone())
        .collect();
    Asked {
        view,
        look: query.get_str("look").map(str::to_owned),
        all_folded: collapse == "all",
        collapse: collapse
            .split(',')
            .filter(|id| !id.is_empty() && *id != "all")
            .map(str::to_owned)
            .collect(),
        show: (!show.is_empty()).then(|| show.into_iter().collect()),
        large: match query.get_str("large") {
            Some("1") => Some(1),
            Some("50") => Some(50),
            _ => None,
        },
        minimap: query.get_str("minimap") == Some("on"),
    }
}
