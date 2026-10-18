<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Security Policy

FerroUI is the component library the FerroHEALTH consoles draw their screens
with. The vulnerability-handling policy every FerroHEALTH product shares is on
one page of the family book, written by the manufacturer, Cadasto B.V.:
<https://ferrohealth.eu/docs/security.html>.

## Supported versions

Security fixes apply to the latest released version only.

| Version             | Supported          |
| ------------------- | ------------------ |
| Latest release      | :white_check_mark: |
| Older releases      | :x:                |
| `main` (unreleased) | best effort        |

## Reporting a vulnerability

**Do not open a public issue for a security vulnerability.** Report it
privately through GitHub's private vulnerability reporting:
<https://github.com/FerroHEALTH/FerroUI/security/advisories/new>, also
reachable from the repository's **Security** tab under **Report a
vulnerability**. Describe the issue, the affected version or commit, and a
reproduction if you have one.

**Never include patient data in a report.** The library draws clinical
screens, so a reproduction is easy to build from real content by accident.
Redact it, or synthesize an equivalent case.

## Response times

FerroUI is solo-maintained, so these are best effort: an acknowledgement
within about 5 business days, and an initial assessment within about 10. A
confirmed vulnerability is fixed in a new patch version.

## Scope

In scope: markup a component writes that lets data a console passes as text
run as script or reach an attribute unescaped, a highlighter that emits
unescaped input, a widget that leaks data into the URL or browser storage, a
stylesheet rule that defeats a `style-src 'self'` policy, and the gallery's
server. Out of scope: a console's own server, session or routing, which belong
to that product's repository.

## Verifying a release

Each release attaches the packaged `.crate` with a Sigstore build provenance
attestation. Verify it with:

```
gh attestation verify ferrohealth-ui-<version>.crate \
  --repo FerroHEALTH/FerroUI \
  --signer-workflow FerroHEALTH/FerroUI/.github/workflows/release.yml
```

The crate on crates.io is published by the same workflow through Trusted
Publishing, so no long-lived token exists that could publish it elsewhere.
