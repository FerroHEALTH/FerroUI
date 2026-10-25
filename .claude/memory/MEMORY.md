<!-- SPDX-FileCopyrightText: Cadasto B.V. -->
<!-- SPDX-License-Identifier: BUSL-1.1 -->

# Memory index

- [Family architecture](family-architecture.md): the FerroHEALTH family
  is five separate servers that reach each other only over published
  protocols, plus shared crates for cross-cutting work; nothing is
  hand-written that a shared crate or another product provides; owner
  2026-10-25
- [Memory lives in the repo](memory-lives-in-repo.md): every learning is a
  tracked file here, never a per-user note
- [Crate name](crate-name.md): the crate is `ferrohealth-ui`; `ferroui` on
  crates.io belongs to an unrelated project and is never used
- [Licence: BUSL 1.1](license-busl.md): the FerroHEALTH terms, Cadasto B.V.
  the Licensor, Apache 2.0 four years after each version
- [Brand palettes](brand-palettes.md): each product keeps its own accent
  (BRIDGE indigo, FED azure, EHR and TERM teal, PIX plum), a shade moved only to pass
  contrast; owner 2026-10-18
- [Sign-in screens](sign-in-screens.md): two forms, a basic form and an OAuth2
  button alone (and both together), FerroEHR's card the reference design
- [Signed commits and the merge queue](merge-queue-signed.md): `main` takes
  only signed commits through pull requests and the merge queue
- [PR licence box](pr-licence-box.md): tick the contribution-licence box in
  every pull request body; the owner's standing consent
- [Release tag](release-tag.md): the tag is signed on the merge commit and the
  crates-io environment waits for the owner's approval
- [All consoles in lockstep](ferroui.md): every release rolls out to all five
  consoles at once, one uniform design, screenshots read before a tag
- [Release closes milestone](release-closes-milestone.md): release.yml refuses
  a tag whose milestone is missing or not empty and closes it after every
  leg; check it after a manual or recovery release; owner 2026-10-19 (#124)
- [Reactive console UI](reactive-console-ui.md): owner 2026-10-25, no full page reload for an interaction; WebAssembly is the normal path, the no-WebAssembly page the fallback (FerroUI #410)
- [Contribution licence attestation](contribution-licence-attestation.md): owner 2026-10-25, always tick the licence checkbox on PRs opened for the owner, never ask
