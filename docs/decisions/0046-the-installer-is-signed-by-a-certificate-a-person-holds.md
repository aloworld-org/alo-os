# ADR 0046 — The installer is signed by a certificate a person holds, and the workflow that builds it never signs

**Status:** proposed, 2026-09-16 — recommendation **A**. The repository half of
it is implemented in the same change: `.github/workflows/release.yml` builds,
checksums and publishes a **draft**, and never signs. What waits for the owner is
a certificate, and the two steps in *What it costs* below.
**Date:** 2026-09-16
**Context:** [ADR 0023](0023-installed-from-the-machine-it-replaces.md) §1 (*a
signed Windows executable, downloaded from the website*),
[ADR 0033](0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
§4 (Secure Boot is never switched off, and a person is never asked to),
[ADR 0036](0036-the-image-is-signed-by-a-key-a-person-holds.md) (who holds the
key the *image* is signed with, and who may publish), and task 5 of
`docs/autonomy/v0-5-the-installer-plan.md`, which asks for a workflow that
builds the installer on a tag, **signs the executable**, and publishes it as a
Release asset with a checksum beside it.

## The question in one line

**The program that repartitions somebody's only computer is downloaded from
GitHub and run on Windows. What signs it, who holds that, and may a workflow?**

## What was true before this decision

ADR 0036 settled the image's key and said, in as many words, that no agent and
no build loop ever holds the private half or publishes. It says nothing about
the installer, and the two are different artefacts signed by different things:
the image is signed with `cosign` against a public key committed in this
repository, and what checks it is our own boot environment. A Windows executable
is signed with **Authenticode**, against a certificate chaining to a certificate
authority Windows already trusts, and what checks it is SmartScreen and the
person's own browser.

This repository has no such certificate, and nothing in it says who should. A
worker sent at task 5 on 2026-09-16 found exactly that: a workflow step calling
`signtool` with a secret nobody has created is a step that fails or silently
skips, and a certificate an agent bought or generated would be the root of trust
for the download page, chosen by nobody.

**What the installer's own guarantees do not rest on.** A signature on the
executable buys reputation with Windows — fewer warnings, a publisher name in
the prompt. It is not what makes the install safe: the release of alo OS that is
written to a disk is pulled **by digest**, pinned in `image/pinned.toml`, and its
signature is verified against the key committed at `image/signing/alo-os.pub`
before a byte is written (ADR 0023 §3, ADR 0036); the boot environment the
installer stages is held to the list its own release was built with. Those hold
whether or not the `.exe` carries an Authenticode signature. This decision is
about the front door, not about the locks behind it.

## The options

### A. A certificate the owner holds; a machine builds, a person signs

The owner obtains a code-signing certificate in their own name or the company's
and keeps it where the image's private half is kept — on a machine no agent runs
on, and on a hardware token when there is one. The workflow on a tag:

1. builds the boot environment from `image/installing/Containerfile` and lists
   every file it is;
2. builds `alo-installer` for Windows with that list compiled into it;
3. puts the executable and the environment into one archive, writes the
   checksum of every asset beside it, and creates a **draft** Release whose
   notes are `image/release-notes.md` — committed, and held to the pin by a
   test.

Then the owner, once, by hand: sign `alo-installer.exe`, put it back into the
archive, refresh `SHA256SUMS`, and publish the draft.

- **For:** it is ADR 0036 applied to the second artefact, which is the whole
  reason that record exists. Nothing automated can publish a program that
  repartitions a computer. A draft nobody signed is not something a person can
  download by accident.
- **Against:** a publish waits for a person, and until they have a certificate
  the Release carries an unsigned executable — which Windows will warn about,
  honestly, because it is unsigned.

### B. The certificate is a repository secret; the workflow signs

- **For:** the whole road is one push of a tag, and the Release is signed the
  moment it exists.
- **Against:** anybody who can change a workflow on this repository can then
  sign the alo OS installer, and so can whoever holds the secret store. This is
  ADR 0036's option B, refused there for the image, and the artefact here is the
  one that repartitions a disk.

### C. A build attestation instead of a signature

GitHub can attest the build of an artefact through Sigstore, keylessly, and
`gh attestation verify` checks it.

- **For:** no certificate to buy, lose or steal, and a public record of every
  build.
- **Against:** Windows does not read it, so it removes no warning and is not
  what ADR 0023 §1 means by *a signed executable*; it anchors trust in an
  identity GitHub issues, which is ADR 0011's *the dependency is on packages,
  not on a company* reversed; and verifying it is a command a person who
  downloaded a program will not run.

## Recommendation

**A**, and it is one sentence: **a machine builds, a person signs, and the
Release is a draft until they have.** The workflow never reaches for a
certificate, a key or a password of any kind, and `crates/alo-image`
(`releasing.rs`) fails the build of this repository if it ever does.

C may be added later as a second layer — a build attestation beside an
Authenticode signature says where the bytes came from as well as who stands
behind them — and it never replaces one. B is refused for as long as this
program can repartition a disk.

## What it costs, and what follows

- **Until there is a certificate, the Release says what it is.** An unsigned
  executable is published as an unsigned executable; no sentence in the notes,
  the README or the program claims a signature it does not have. The checksum
  beside the download is what a person can actually compare.
- **Two steps are the owner's**, and neither is a worker's: obtain the
  certificate, and sign the executable of each Release before publishing the
  draft. Both belong beside ADR 0036's five, in the same hands.
- **A person still never sees a key** (ADR 0036, as the owner accepted it).
  Nothing here changes what the installer says: no screen, prompt or sentence
  names a key, a password or a signature.
- **The tag is the only thing a person types**, and it is refused before
  anything is built when it does not name the pinned release — so the notes,
  the pin, the environment and the tag cannot come apart.

## Rejected, if A is accepted

- **A certificate an agent obtains or generates.** It would work, and it would
  make the publisher of alo OS somebody nobody chose.
- **B**, for the reasons above; revisited only if signing ever stops being a
  thing one person can do in a minute, and then only as *the workflow builds, a
  person signs*.
- **Publishing a Release that is not a draft.** A Release that exists is a
  Release somebody can download; the draft is what keeps the unsigned build off
  the download page.
