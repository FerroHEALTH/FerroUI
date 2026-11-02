---
name: actions-open-no-prs
description: GitHub Actions may not open or approve a pull request in FerroHEALTH repositories and no lane token exists, so a scheduled lane pushes its branch and a maintainer opens the pull request (owner, 2026-11-02)
metadata:
  type: project
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

The owner keeps the repository setting that lets GitHub Actions create or
approve pull requests off, and gives no lane a token of its own
(FerroHEALTH/FerroEHR#4797, 2026-11-02). A `gh pr create` from a workflow
fails there with "GitHub Actions is not permitted to create or approve pull
requests".

**Why:** FerroEHR's ratchet lane committed its counts and then failed on
the pull request; its first hand-off was opened by hand.

**How to apply:** a lane that proposes a change (`ratchet-tighten.yml`)
commits through the Git Data API to its own branch, ends with a notice
naming the branch, and a maintainer opens the pull request, which also
starts its checks. Send every `gh api` request body through a file
(`--input`): a file's base64 content passed as one argument passes the
kernel's single-argument limit (FerroHEALTH/FerroEHR#4790).
