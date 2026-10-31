<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Versions

## CI tools

The CI tools a workflow installs (zizmor, actionlint, shellcheck and the
cargo tools) are pinned in the workflow step that runs them. The clone
detector of `scripts/checks/duplication.sh` (#536) also runs on a developer
machine, so its pins live here and the guard reads them from this table:
the release, the official image of that release by digest, and the sha256 of
each release archive `scripts/checks/duplication.sh --install` fetches into
`target/tools/` for a machine with neither the binary on `PATH` nor docker
or podman. The guard refuses a binary that answers another version. Bumping
jscpd moves every row together, plus a fresh `--update` of the count when
the new release counts differently.

| Tool | Pin | Read by |
|---|---|---|
| `jscpd` | `5.4.2` (release v5.4.2 of github.com/kucherenko/jscpd, 2026-10-20) | `scripts/checks/duplication.sh` |
| `jscpd image` | `ghcr.io/kucherenko/jscpd:5.4.2@sha256:112bc771493f25ec412ebf61bef738dc1fbe80d3793090e6a5eeac827ef0ee79` | `scripts/checks/duplication.sh` (docker, podman) |
| `jscpd darwin-arm64` | `b51d22d7c7610d27d8fd022577021ff3266a10f13295a2e545a8b8e126133a88` (`jscpd-darwin-arm64.tar.gz`) | `scripts/checks/duplication.sh --install` |
| `jscpd darwin-x64` | `ae8d24aba2719df7682b1bb2c5021ab096116b8240c45fdae88654fc1ff36b17` (`jscpd-darwin-x64.tar.gz`) | `scripts/checks/duplication.sh --install` |
| `jscpd linux-x64` | `5f18a6b70b4210782fc3c401c780ba994a2777249cd44d97d6fa460f6fff4570` (`jscpd-linux-x64-musl.tar.gz`) | `scripts/checks/duplication.sh --install` |
| `jscpd linux-arm64` | `3635d652ae08ae10bfbc6821bfa8ac14080101f317fb5f45f4b590d5d32b9339` (`jscpd-linux-arm64-musl.tar.gz`) | `scripts/checks/duplication.sh --install` |

The archive sums are the `checksums.txt` of release v5.4.2, read on
2026-10-31.
