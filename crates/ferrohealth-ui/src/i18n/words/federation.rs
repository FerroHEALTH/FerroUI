// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The library's words of the federation parts: the fan-out outcome, the
//! replica line and the topology.

use crate::i18n::Text;

texts! {
    /// A fan-out's verdict when no node failed.
    FanoutComplete = "fanout-complete",
    /// A fan-out's verdict when every node asked failed.
    FanoutFailed = "fanout-failed",
    /// A fan-out's verdict when some nodes failed.
    FanoutIncomplete = "fanout-incomplete",
    /// The words that open a fan-out's notice when a node failed.
    FanoutIncompleteAnswer = "fanout-incomplete-answer",
    /// The replica line's words before the replica's name.
    ReplicaAnswered = "replica-answered",
    /// A group card's control that folds it: `$title` is its name.
    TopologyCollapse = "topology-collapse",
    /// The zoom cluster's control that folds every group.
    TopologyCollapseAll = "topology-collapse-all",
    /// The accessible name of the zoom cluster.
    TopologyControls = "topology-controls",
    /// A group card's control that unfolds it: `$title` is its name.
    TopologyExpand = "topology-expand",
    /// The zoom cluster's control that unfolds every group.
    TopologyExpandAll = "topology-expand-all",
    /// The zoom cluster's Fit.
    TopologyFit = "topology-fit",
    /// The tooltip of Fit.
    TopologyFitWhole = "topology-fit-whole",
    /// The Status menu's line under its states.
    TopologyLegend = "topology-legend",
    /// The tooltip of a card's quick look.
    TopologyLook = "topology-look",
    /// The quick look's close link.
    TopologyLookClose = "topology-look-close",
    /// The accessible name of a card's quick look: `$title` is its name.
    TopologyLookOf = "topology-look-of",
    /// The map's accessible name for its keys.
    TopologyMapKeys = "topology-map-keys",
    /// The sentence the map's description ends with, naming the List.
    TopologyMapList = "topology-map-list",
    /// The zoom cluster's control that shows or hides the minimap.
    TopologyMinimap = "topology-minimap",
    /// The accessible name of the minimap.
    TopologyMinimapFrame = "topology-minimap-frame",
    /// The Status menu's submit button.
    TopologyShow = "topology-show",
    /// The accessible name of the Status menu.
    TopologyStates = "topology-states",
    /// The Status button with every state shown.
    TopologyStatus = "topology-status",
    /// The Status button with some states shown: `$shown` of `$total`.
    TopologyStatusSome = "topology-status-some",
    /// The accessible name of the view switch.
    TopologyView = "topology-view",
    /// The view switch's List.
    TopologyViewList = "topology-view-list",
    /// The view switch's Map.
    TopologyViewMap = "topology-view-map",
    /// The zoom readout: `$value` is the scale in per cent.
    TopologyZoom = "topology-zoom",
    /// The zoom cluster's Zoom in.
    TopologyZoomIn = "topology-zoom-in",
    /// The zoom readout's accessible name: `$value` is the scale in per cent.
    TopologyZoomLevel = "topology-zoom-level",
    /// The zoom cluster's Zoom out.
    TopologyZoomOut = "topology-zoom-out",
    /// Why the zoom controls wait: they work once the page's script runs.
    TopologyZoomWait = "topology-zoom-wait",
}
