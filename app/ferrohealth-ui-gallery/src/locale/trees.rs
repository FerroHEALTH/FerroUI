// SPDX-FileCopyrightText: Cadasto B.V.
// SPDX-License-Identifier: BUSL-1.1

//! The messages of the trees page.

use super::Msg;

messages! {
    /// `Finding of the abdomen`
    TreesAbdomen = "trees-abdomen",
    /// `Abdominal pain`
    TreesAbdominalPain = "trees-abdominal-pain",
    /// `2026-08 Admission`
    TreesAdmission = "trees-admission",
    /// `Blood pressure`
    TreesBloodPressure = "trees-blood-pressure",
    /// `Temperature`
    TreesBodyTemperature = "trees-body-temperature",
    /// `Caribbean Netherlands`
    TreesCaribbean = "trees-caribbean",
    /// `Changes stay in this page until you save them as one new version.`
    TreesChangesStay = "trees-changes-stay",
    /// `Finding of the chest`
    TreesChest = "trees-chest",
    /// `Chest pain`
    TreesChestPain = "trees-chest-pain",
    /// `Reasons for a chest referral`
    TreesChestReferral = "trees-chest-referral",
    /// `Children`
    TreesChildren = "trees-children",
    /// `{ $count } concepts`
    TreesConceptsFound = "trees-concepts-found",
    /// `Context`
    TreesContext = "trees-context",
    /// `Correspondence`
    TreesCorrespondence = "trees-correspondence",
    /// `Cough`
    TreesCough = "trees-cough",
    /// `Cuff size`
    TreesCuff = "trees-cuff",
    /// `A deep path and an inactive concept`
    TreesDeepPath = "trees-deep-path",
    /// `Device`
    TreesDevice = "trees-device",
    /// `Device name`
    TreesDeviceName = "trees-device-name",
    /// `Diastolic`
    TreesDiastolic = "trees-diastolic",
    /// `A directory`
    TreesDirectory = "trees-directory",
    /// `The directory of this EHR`
    TreesDirectoryOfEhr = "trees-directory-of-ehr",
    /// `Discharge summary`
    TreesDischarge = "trees-discharge",
    /// `Display`
    TreesDisplay = "trees-display",
    /// `Dry cough`
    TreesDryCough = "trees-dry-cough",
    /// `empty`
    TreesEmpty = "trees-empty",
    /// `Vital signs encounter`
    TreesEncounter = "trees-encounter",
    /// `Episodes`
    TreesEpisodes = "trees-episodes",
    /// `Fatigue`
    TreesFatigue = "trees-fatigue",
    /// `Fever`
    TreesFever = "trees-fever",
    /// `Node name or path`
    TreesFindNode = "trees-find-node",
    /// `Synthetic finding`
    TreesFinding = "trees-finding",
    /// `A flat code system`
    TreesFlatSystem = "trees-flat-system",
    /// `{ $count } folders`
    TreesFolders = "trees-folders",
    /// `Folders`
    TreesFoldersSection = "trees-folders-section",
    /// `2026-10 Hypertension follow-up`
    TreesFollowUp = "trees-follow-up",
    /// `2026-10 Hypertension follow-up, week 43`
    TreesFollowUpRenamed = "trees-follow-up-renamed",
    /// `General finding`
    TreesGeneral = "trees-general",
    /// `Hierarchy`
    TreesHierarchy = "trees-hierarchy",
    /// `The hierarchy of Synthetic clinical findings`
    TreesHierarchyOfFindings = "trees-hierarchy-of-findings",
    /// `The hierarchy of Synthetic national findings`
    TreesHierarchyOfNational = "trees-hierarchy-of-national",
    /// `Folders, in an edit`
    TreesInEdit = "trees-in-edit",
    /// `{ $count } items`
    TreesItems = "trees-items",
    /// `Laboratory result report`
    TreesLaboratory = "trees-laboratory",
    /// `A long level and a level being read`
    TreesLongLevel = "trees-long-level",
    /// `Lying`
    TreesLying = "trees-lying",
    /// `Lying with tilt to left`
    TreesLyingTilt = "trees-lying-tilt",
    /// `Malaise, old code`
    TreesMalaise = "trees-malaise",
    /// `Concepts the search matched`
    TreesMatched = "trees-matched",
    /// `Synthetic national finding`
    TreesNationalFinding = "trees-national-finding",
    /// `Nausea`
    TreesNausea = "trees-nausea",
    /// `Netherlands`
    TreesNetherlands = "trees-netherlands",
    /// `Parent`
    TreesParent = "trees-parent",
    /// `Paths`
    TreesPaths = "trees-paths",
    /// `The paths of Vital signs encounter.v1`
    TreesPathsOf = "trees-paths-of",
    /// `Position`
    TreesPosition = "trees-position",
    /// `Productive cough`
    TreesProductiveCough = "trees-productive-cough",
    /// `Pulse`
    TreesPulse = "trees-pulse",
    /// `Rash`
    TreesRash = "trees-rash",
    /// `Rate`
    TreesRate = "trees-rate",
    /// `read from version { $version }`
    TreesReadFrom = "trees-read-from",
    /// `Synthetic referral reason { $index }`
    TreesReason = "trees-reason",
    /// `Reclining`
    TreesReclining = "trees-reclining",
    /// `root`
    TreesRoot = "trees-root",
    /// `Root archetype`
    TreesRootArchetype = "trees-root-archetype",
    /// `Search`
    TreesSearch = "trees-search",
    /// `Search the designations`
    TreesSearchDesignations = "trees-search-designations",
    /// `Search to see concepts.`
    TreesSearchToSee = "trees-search-to-see",
    /// `Setting`
    TreesSetting = "trees-setting",
    /// `Sitting`
    TreesSitting = "trees-sitting",
    /// `Finding of the skin`
    TreesSkin = "trees-skin",
    /// `Reasons for a skin referral`
    TreesSkinReferral = "trees-skin-referral",
    /// `Standing`
    TreesStanding = "trees-standing",
    /// `The concept tree, the folder tree and the path browser, each walked by the keyboard.`
    TreesSubtitle = "trees-subtitle",
    /// `System`
    TreesSystem = "trees-system",
    /// `Systolic`
    TreesSystolic = "trees-systolic",
    /// `Body temperature`
    TreesTemperature = "trees-temperature",
    /// `A template's paths`
    TreesTemplatePaths = "trees-template-paths",
    /// `The concept`
    TreesTheConcept = "trees-the-concept",
    /// `Vital signs`
    TreesVitalSigns = "trees-vital-signs",
}
