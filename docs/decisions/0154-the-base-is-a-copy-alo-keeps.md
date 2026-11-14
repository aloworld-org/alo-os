# ADR 0154 — the base is a copy alo keeps

**Status:** accepted, 2026-11-13, by the owner, relayed by the coordinator.
Offered where a copy of the pinned base should live, the owner answered:
*"alo's GitHub registry"* — copy each pinned base into `ghcr.io/aloworld-org`,
GitHub's registry beside the code, build from that copy, and have a check
refuse a build whose base is not copied.
**Date:** 2026-11-13

## What it changes for the person

**Nothing on their machine.** The bytes alo OS is built on are Fedora's, exactly
as before, and the digest the recipe names proves it.

What changes is that **alo can be built, and so updated, tomorrow.** On
2026-11-13 every image build from `main` failed, because the base the recipe
pinned had been deleted by the registry it lived on. An update nobody can build
is an update nobody receives, so a base that can vanish is a machine that can
stop being fixed. After this, the base stays for as long as alo keeps it.

## Why

Measured on 2026-11-13 (`docs/quirks/quay-deletes-a-fedora-bootc-digest-the-day-its-tag-moves.md`):

1. ADR 0152 pinned `quay.io/fedora/fedora-bootc:44@sha256:ea70c130…`, the tag's
   digest that morning. The first image build from `main` after it landed
   stopped: `reading manifest sha256:ea70c130… in quay.io/fedora/fedora-bootc:
   manifest unknown`.
2. Fedora rebuilds `fedora-bootc:44` every day — the tag moved at
   2026-11-13T10:07Z, to `44.20261113.0` — and Quay no longer served the
   manifest its tag stopped naming. A pin of a release Fedora still builds
   lasts about a day.
3. Moving the pin (#1235, to `sha256:5a3472…`) is a one-day fix: the same
   thing happens to the new digest when the tag next moves.

**A digest still proves what it names; it does not keep it.** Pinning by
content was never the fault. The fault was relying on somebody else to keep
the content.

## The decision

1. **The recipe's base is alo's copy, by the maker's digest:**
   `ARG THE_BASE=ghcr.io/aloworld-org/<name>@sha256:<the same digest as
   upstream>`, in `image/Containerfile` and `image/installing/Containerfile`
   alike.
2. **The copy is byte for byte.** `skopeo copy --all --preserve-digests` copies
   every platform the maker's manifest list names, and refuses rather than write
   a copy whose digest differs. So the digest in the recipe still proves the
   bytes are Fedora's, and anybody can check it against Fedora.
3. **Each base is recorded with where it came from**, in `image/bases.toml`:
   the copy, its tag (the maker's own version of that build), the digest, the
   upstream repository with the tag that named the digest, and the day. One
   base per line, because the copying workflow reads lines.
4. **A person makes the copy.** `.github/workflows/copying-a-base.yml` runs only
   when asked, only from `main`, with a digest the record names. It reads the
   original by digest and hashes it, copies it, reads the copy back and hashes
   it, and **fails until the copy can be pulled without signing in**, because a
   lane machine building a walk image does not sign in. It never builds, pushes
   an image or signs.
5. **A check refuses a recipe whose base is not recorded**
   (`crates/alo-image`, `tests/the_base_is_a_copy_alo_keeps.rs`), and every
   workflow that builds a recipe — `image.yml`, `installer-candidate.yml`,
   `release.yml` — asks whether the base can be pulled before it builds, and
   says in one sentence what to do when it cannot. A pull request that changes
   a recipe or the record asks the same question
   (`.github/workflows/the-base-can-be-pulled.yml`). **That check is not one
   the merge queue requires**; which checks it requires is the owner's setting.

### Landed in two changes

The copy cannot exist before the workflow that makes it is on `main`, and a
recipe naming a copy that does not exist is a recipe nobody can build. So:

- **First**, the record, the workflow, the build workflows' step and the check.
  The check accepts a recipe naming **either** the recorded copy **or** the
  recorded original (`TheBases::original_of`), and says so where it does.
- **Then a person runs the workflow** with the digest. Where the new package is
  not public, they make it public once on its settings page, which GitHub
  offers no way to do from a workflow.
- **Second**, the recipes and every test that holds their base move to the
  copy, and the allowance for the original is removed. Its pull-request check
  stays red until the copy exists, which is the point; the gate's own tests
  would pass either way, because they do not reach the network.

### The first copy, 2026-11-13

The coordinator ran *Copying a base* on `main` with
`sha256:5a3472162ba7781a01d249b9a56bc5a9b9fa01c08edae1f73c77dfa1e504e868`
(run 43566119644). It succeeded on its first run. **Every one of its own checks
passed**: it read Fedora's original by that digest and hashed it, copied every
platform, read the copy back and hashed it, and read it again without signing
in. **The package was already public**, so nobody had to change its visibility.
What the workflow's comment and this decision expected was a private package; on
this organisation a new one came out public, and whether that holds for the next
package was not measured.

Then, once and by hand, the coordinator ran an anonymous
`skopeo inspect --no-creds --raw` of
`ghcr.io/aloworld-org/fedora-bootc@sha256:5a3472…`, and its bytes hash to exactly
that digest. So alo's copy is Fedora's manifest list byte for byte, and anybody
can pull it. That hand check is not repeated anywhere. What is repeated is the
build workflows' first step and *The base can be pulled* on a pull request,
which ask whether the base can be pulled (not whether it hashes right); the
hash is checked again only when *Copying a base* runs again.

## What it does not do

- **It does not keep alo up to date with Fedora.** A newer Fedora build is a
  new line in the record, a run of the workflow, and a change to the recipe —
  the same decision as any other move of the base, made by somebody, never by a
  schedule.
- **The tests do not check that the copy exists.** They do not reach the
  network. The build workflows' first step does, every time they run, and so
  does the pull-request check, whenever a recipe or the record changes.
- **The builder image is not covered.** `THE_BUILDER`
  (`docker.io/library/ubuntu:26.04@sha256:…`) is pinned the same way and could
  vanish the same way; whether Docker Hub keeps it was not measured. Copying
  it is the same record and the same workflow, and is not done here.
- **No image is pushed from a lane machine.** Only the workflow writes to
  `ghcr.io/aloworld-org`.

## What it costs

- **Four platforms are copied to use one.** Fedora's manifest list names amd64,
  arm64, ppc64le and s390x; preserving its digest means keeping all four. The
  storage is GitHub's, which says it does not charge for public packages.
- **alo now holds what it builds from.** Deleting a package in
  `ghcr.io/aloworld-org` is now the act that would stop builds, and it is one
  only the organisation's owners can do.
