# Quay deletes a fedora-bootc digest the day its tag moves

- **Version:** `quay.io/fedora/fedora-bootc:44`, as Quay served it on
  2026-11-13.
- **Whose:** Fedora's and Quay's — how they publish and keep a release that is
  still being built. alo OS configures nothing here.
- **Date:** 2026-11-13.

## Behaviour

ADR 0152 pinned `fedora-bootc:44@sha256:ea70c130…`, the tag's digest that
morning. The first image built from `main` after it landed stopped at its first
line: `reading manifest sha256:ea70c130… in quay.io/fedora/fedora-bootc:
manifest unknown`. Fedora rebuilds the tag daily (it moved at
2026-11-13T10:07Z, to `44.20261113.0`), and the old manifest was no longer
served once the tag stopped naming it. A digest pin of a release Fedora still
builds lasted under a day. Fedora 42's pin never showed this, because 42 is no
longer rebuilt.

What Quay said about the new digest on 2026-11-13, measured once by hand with
its public API (`/api/v1/repository/fedora/fedora-bootc/tag/?specificTag=44`):
`sha256:5a3472162ba7…` is a **manifest list of four platforms** (amd64, arm64,
ppc64le, s390x), and **no tag named `44.20261113.0` exists** — the version is
only in each image's `org.opencontainers.image.version` annotation. Nothing
re-runs this measurement.

**Cause not confirmed.** Whether Quay garbage-collects untagged manifests at once
or after a grace period was not measured; what was measured is that a digest
from that morning was gone that afternoon.

## Our response

[ADR 0154](../decisions/0154-the-base-is-a-copy-alo-keeps.md): alo keeps its own
copy of each base in `ghcr.io/aloworld-org`, copied with every platform so the
digest stays Fedora's (`skopeo copy --all --preserve-digests`), and builds from
that. `image/bases.toml` records each copy and where it came from, and every
build workflow asks whether its base can be pulled before it builds.
