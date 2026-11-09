---
name: ci-under-20-minutes
description: "pull-request and merge-queue CI each finish in under 20 minutes; the queue proves the merged tree and leaves the browser gates to the pull request and main; few jobs, started at once"
metadata:
  type: feedback
---

<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

Every FerroHEALTH repository's pull-request CI and merge-queue CI finish in
under 20 minutes of wall time, runner queue included (owner, 2026-11-09).

**Why:** the hosted runner pool is shared across the organisation. A run's
wall time was mostly the wait for runners, paid once per job and again per
wave of jobs that wait on another, so 26 jobs in three waves took 54 minutes
for 13 runner-minutes of work (#669).

**How to apply:** add a step to `guards`, `lint` or `nextest` before adding
a job; give a guard step `if: ${{ !cancelled() }}`. The merge queue runs the
build, clippy, the tests, the guards that read more than one file and the
publish dry run; the gallery journeys, the capture and the design sets run
on the pull request and on the push to `main`, never in the queue. Never
weaken a gate to save time: only the event it runs on may change.
