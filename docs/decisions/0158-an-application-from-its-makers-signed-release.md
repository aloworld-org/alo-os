# ADR 0158 — an application from its maker's signed release

**Status:** accepted, 2026-11-21. Proposed 2026-11-17 by the market lane, on the owner's decision of
the same day relayed by the coordinator: the Craft applications
(github.com/storytold) are *"added, labelled early version"*, *"installed from
their signed `.flatpak` bundle on GitHub releases"*, and *"the signature and
checksum are verified before installing; a bad or missing signature is
refused, and that refusal is tested."* What is the owner's is that; how the
machine decides whom to trust is this lane's, and is what an owner's answer is
asked for below.

*Accepted 2026-11-21, because both things it put to the owner are now
answered.* The door out of the sandbox was answered on 2026-11-17 (*"list them
with a plain warning"*, `docs/features.md`, the market's lines). Whom the
machine trusts was answered on 2026-11-21 by the owner's *"Registered makers,
free apps first"* (ADR 0160): each maker registers a signing key and alo
reviews each listing, which is decision 1 below made for every maker rather
than one. **Accepting it measures nothing:** whether the pinned base carries
`gpgv` is still unmeasured, as *What this costs* says.
**Date:** 2026-11-17

## What it changes for the person

**A person can install an application its maker publishes only on its own
release page**, from the same market as everything else, and the machine still
refuses anything it cannot show came from that maker.

Two things are different from an application that comes from Flathub, and the
person is told both in plain words:

- **It does not update the way their other applications do.** alo offers a
  Flathub application's newer version when Flathub has one. Nothing offers a
  newer version of a release installed from a file: the catalogue pins one
  release, and a newer one reaches the machine only when this repository pins
  it. (The Craft applications carry an updater of their own; see *What this
  costs* below.)
- **It is an early version**, which the market says on its row.

## What the machine trusts, and why that is a decision

Until now the machine trusted exactly one kind of thing: a place applications
come from, set up in the rented tool **with signature checking on**
(`crate::enabled` refuses one with it off). The tool checks every delivery
against that place's key, and the key arrived when the place was set up.

A release page is not such a place. GitHub serves whatever the repository's
owner uploads, and a release can be replaced. So the machine needs its own
answer to *did this come from its maker*, and there are three ways to get one
wrong:

1. **Trusting the key the release ships beside the file.** Every Craft release
   carries `release-signing-key.asc`. A file signed by a key fetched from the
   same page proves only that whoever uploaded one uploaded the other.
2. **Trusting a checksum file from the same page**, for the same reason.
3. **Trusting the address.** An address is where something was, not what it is.

## The decision

1. **The maker's key is pinned in this repository**, as a file
   (`crates/alo-software/makers/storytold.asc`) and by its fingerprint
   (`2B76741B414AAD6CD33E631213A005C0B9D7C3ED`), and every verdict must name
   that fingerprint, so a key file swapped for another is refused too. It is
   never fetched. The key was read on 2026-11-17
   from seven Craft repositories' latest releases, which all carried the same
   one; a release signed by any other key is refused.
2. **The file's SHA-256 is pinned in `catalogue.toml`**, beside the release it
   names. A file that does not hash to it is refused before its signature is
   read. This is the same pin `image/pinned.toml` holds a base to.
3. **Its detached signature must verify against that one key**, using `gpgv`
   — GnuPG's verify-only program, a rented engine, configured and never
   patched — with a keyring holding only the pinned key. The verdict is read
   from gpgv's machine-readable `VALIDSIG` line, and the fingerprint on it must
   be the pinned one. gpgv's exit status is not the verdict.
4. **A missing signature, a bad one, a wrong key and a wrong checksum are all
   one refusal**: *what {source} sent for {application} could not be shown to
   come from {source}*, the sentence the machine already says when Flathub's
   delivery fails its check. Nothing fetched is kept.
5. **Each maker is a place by name**, so an organisation's rule about which
   places may be used (ADR 0016) reaches it exactly as it reaches Flathub, and
   the download is shown on the egress indicator as an installation, to the
   maker's release host.
6. **Only `https://github.com/<maker>/…/releases/download/…` addresses** are
   accepted for a maker, so a catalogue entry cannot point a maker's name at
   another host.

## What this costs

- **Pinning a checksum means pinning a release.** A new Craft release is a new
  line in `catalogue.toml`, decided and reviewed like any other. That is the
  same weaker-than-a-digest-but-stronger-than-a-name trade ADR 0073 made for
  the shipped list, made stronger here.
- **A maker's key can be lost or replaced.** Then every release signed by a new
  key is refused until this repository pins it. That is the point.
- **gpgv has to be on the machine.** It is on the Ubuntu runner that runs this
  repository's tests (`gpgv (GnuPG) 2.4.4`, measured 2026-11-17). Whether the
  pinned Fedora base carries it **has not been measured**, and a machine
  without it refuses every signed release, in words, rather than installing
  one unchecked.
- **The Craft bundles ask for a door out of their sandbox.** Read from
  `packaging/linux/flatpak/ai.storyteller.photocraft.bundle.yml` at tag
  `v0.23.1`: `--talk-name=org.freedesktop.Flatpak`, *"the shared updater
  inspects and updates this installation via flatpak-spawn --host."* That bus
  name lets an application run commands on the machine outside its sandbox.
  Flathub applications can ask for it too (an IDE often does); the rented tool
  grants what an application declares at installation, and this decision does
  not change that. **It is put to the owner** with this ADR, because the market
  says each application *arrives able to reach nothing* and for an application
  declaring this permission that sentence is not true.

## What is not decided here

- Any maker but storytold. A second maker is a second key file and a second
  fingerprint, reviewed as this one is. *ADR 0160 (2026-11-21) decides there
  will be others, as registered makers on exactly these terms; each is still
  pinned and reviewed one at a time.*
- Any architecture but x86_64. The pinned addresses are x86_64 bundles, and a
  machine of another architecture is refused rather than handed the wrong one.
- An update path. Building one — reading a maker's newer signed release and
  offering it — is possible on this footing and is not built.
