// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the federation page.

use super::Msg;

messages! {
    /// `503`
    Fed503 = "fed-503",
    /// `Every node answered`
    FedAllAnswered = "fed-all-answered",
    /// `Every node failing`
    FedAllFailing = "fed-all-failing",
    /// `Answered`
    FedAnswered = "fed-answered",
    /// `It answered 503 Service Unavailable.`
    FedAnswered503 = "fed-answered503",
    /// `asked nothing`
    FedAskedNothing = "fed-asked-nothing",
    /// `Calls the gateway`
    FedCallers = "fed-callers",
    /// `who calls the gateway`
    FedCallersLine = "fed-callers-line",
    /// `Consent`
    FedConsent = "fed-consent",
    /// `Consent registry`
    FedConsentRegistry = "fed-consent-registry",
    /// `National contact point`
    FedContactPoint = "fed-contact-point",
    /// `Planned`
    FedContactPointLine = "fed-contact-point-line",
    /// `Degraded`
    FedDegraded = "fed-degraded",
    /// `Degraded · 503`
    FedDegraded503 = "fed-degraded503",
    /// `Excluded`
    FedExcluded = "fed-excluded",
    /// `Excluded by the operator.`
    FedExcludedByOperator = "fed-excluded-by-operator",
    /// `Federation`
    FedFederationId = "fed-federation-id",
    /// `The identifier OPTIONS declares for this federation.`
    FedFederationIdLine = "fed-federation-id-line",
    /// `FerroFED gateway`
    FedGateway = "fed-gateway",
    /// `2 replicas · in step`
    FedGatewayLine = "fed-gateway-line",
    /// `Identity`
    FedIdentity = "fed-identity",
    /// `The sidebar opens this organisation under Federation, with its nodes and their health, so you move between them without leaving it.`
    FedInsideBody = "fed-inside-body",
    /// `Inside the federation`
    FedInsideTitle = "fed-inside-title",
    /// `4 of 4 asked nodes answered · 412 ms · no deduplication`
    FedLineComplete = "fed-line-complete",
    /// `0 of 4 asked nodes answered · 2,500 ms · all or nothing`
    FedLineFailed = "fed-line-failed",
    /// `0 of 3 nodes asked · every node excluded`
    FedLineNoneAsked = "fed-line-none-asked",
    /// `3 of 4 asked nodes answered · 412 ms · best effort`
    FedLinePartial = "fed-line-partial",
    /// `The federation as a list`
    FedListLabel = "fed-list-label",
    /// `Answers 503 since 09:38. The last 30 probes and the recent errors are on its page.`
    FedLookBody = "fed-look-body",
    /// `Node of Testkliniek Oost`
    FedLookLine = "fed-look-line",
    /// `The federation as a map`
    FedMapLabel = "fed-map-label",
    /// `median 38 ms`
    FedMedian = "fed-median",
    /// `No node answered, so the gateway returned no rows. Do not read a missing row as absent.`
    FedMissingAll = "fed-missing-all",
    /// `Every asked node answered.`
    FedMissingNone = "fed-missing-none",
    /// `node-b-pub answered with an error (503), so these rows are not the whole answer. Do not read a missing row as absent.`
    FedMissingOne = "fed-missing-one",
    /// `390 ms`
    FedMs390 = "fed-ms390",
    /// `Ziekenhuis Voorbeeldstad and its nodes`
    FedNestLabel = "fed-nest-label",
    /// `never probed`
    FedNeverProbed = "fed-never-probed",
    /// `no answer`
    FedNoAnswer = "fed-no-answer",
    /// `Node error`
    FedNodeError = "fed-node-error",
    /// `A node of Ziekenhuis Voorbeeldstad, open in the sidebar under its organisation.`
    FedNodePage = "fed-node-page",
    /// `All nodes reachable`
    FedNodesReachable = "fed-nodes-reachable",
    /// `No node asked`
    FedNoneAsked = "fed-none-asked",
    /// `not asked`
    FedNotAsked = "fed-not-asked",
    /// `Not probed`
    FedNotProbed = "fed-not-probed",
    /// `One node failing`
    FedOneFailing = "fed-one-failing",
    /// `1 node needs attention`
    FedOneNotWell = "fed-one-not-well",
    /// `Open page`
    FedOpenPage = "fed-open-page",
    /// `Testkliniek Oost`
    FedOrgOost = "fed-org-oost",
    /// `Zorggroep Proefdorp`
    FedOrgProefdorp = "fed-org-proefdorp",
    /// `Huisartsenpost Testmeer`
    FedOrgTestmeer = "fed-org-testmeer",
    /// `Ziekenhuis Voorbeeldstad`
    FedOrgVoorbeeldstad = "fed-org-voorbeeldstad",
    /// `Organisation`
    FedOrganisation = "fed-organisation",
    /// `An organisation of the federation, its nodes open in the sidebar.`
    FedOrganisationPage = "fed-organisation-page",
    /// `Organisations and nodes`
    FedOrganisations = "fed-organisations",
    /// `and their nodes`
    FedOrganisationsLine = "fed-organisations-line",
    /// `PDQm demographics`
    FedPdqm = "fed-pdqm",
    /// `Per-node budget`
    FedPerNode = "fed-per-node",
    /// `How long one node may take, in milliseconds.`
    FedPerNodeLine = "fed-per-node-line",
    /// `PIXm resolver`
    FedPixm = "fed-pixm",
    /// `Probe now`
    FedProbeNow = "fed-probe-now",
    /// `probed 13 s ago`
    FedProbed = "fed-probed",
    /// `Reachable`
    FedReachable = "fed-reachable",
    /// `0 rows · 118 ms`
    FedRows0 = "fed-rows0",
    /// `1 row · 74 ms`
    FedRows1 = "fed-rows1",
    /// `2 rows · 66 ms`
    FedRows2 = "fed-rows2",
    /// `3 rows · 41 ms`
    FedRows3 = "fed-rows3",
    /// `Self-description`
    FedSelfDescription = "fed-self-description",
    /// `What clients see when they ask the gateway what it is.`
    FedSelfDescriptionLine = "fed-self-description-line",
    /// `Services the gateway asks`
    FedServices = "fed-services",
    /// `what the gateway asks`
    FedServicesLine = "fed-services-line",
    /// `The topology with its List twin, the fan-out outcome, and the nested sidebar section inside an organisation.`
    FedSubtitle = "fed-subtitle",
    /// `Topology`
    FedTopology = "fed-topology",
    /// `2 nodes`
    FedTwoNodes = "fed-two-nodes",
    /// `2 nodes · all reachable`
    FedTwoNodesReachable = "fed-two-nodes-reachable",
    /// `2 services · all reachable`
    FedTwoServicesReachable = "fed-two-services-reachable",
    /// `Unreachable`
    FedUnreachable = "fed-unreachable",
    /// `Unreachable · no answer`
    FedUnreachableNoAnswer = "fed-unreachable-no-answer",
    /// `Updated 13 s ago · every 30 s`
    FedUpdated = "fed-updated",
    /// `What each endpoint did`
    FedWhatEachDid = "fed-what-each-did",
    /// `Worst node: reachable`
    FedWorstReachable = "fed-worst-reachable",
    /// `Federation`
    Federation = "federation",
}
