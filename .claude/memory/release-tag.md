---
name: release-tag
description: "A release is a signed vX.Y.Z tag on the merge commit on main; the crates leg waits on the crates-io environment's required reviewer, the owner"
metadata:
  type: project
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

The release pull request (`release/vX.Y.Z`, after the screenshot refresh
on `chore/screenshots`) bumps the workspace version and the lock and
assembles the changelog section from the fragments (`docs/release.md`). After it merges, the tag `vX.Y.Z` is created signed
(`git tag -s`) on that merge commit and pushed; `release.yml` builds the
draft, attests the `.crate`, publishes the release, and its `crates` leg
pauses on the `crates-io` environment until the owner approves. `plan`
refuses the tag while the milestone titled as it is missing or holds an open
item, and the last job closes the milestone once every leg succeeded
([[release-closes-milestone]]).

**How to apply:** never move or delete a pushed tag (the `release-tags`
ruleset refuses it); a bad cut is fixed by a new patch version. Tell the owner
when the run waits on the approval.
