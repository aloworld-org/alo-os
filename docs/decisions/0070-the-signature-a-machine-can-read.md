# ADR 0070 — The signature a machine can read, and the one flag that produces it

**Status:** proposed, 2026-09-26.

Written by the Mac lane from task 8 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, whose first half —
*the shipped image carries no `policy.json` at all* — turned out not to be
writable without answering this. A `policy.json` **is** the statement of what a
machine will accept, so its content is this question's answer and not a step
after it.

## The decision in one line

`cosign` 3 writes a signature no alo OS machine can read. **One undocumented flag
makes it write one they can**, so the release side moves, nothing about the images
changes, and the currently pinned release can be made verifiable **without a
rebuild or a republish**.

## What is published today, measured against the registry

Anonymous HTTPS against `ghcr.io/aloworld-org/alo-os`, all five releases:

| | |
|---|---|
| signature tag | `sha256-<digest>` — **no `.sig`** |
| shape | OCI index → manifest `application/vnd.dev.sigstore.bundle.v0.3+json` |
| annotations | `dev.sigstore.bundle.content: dsse-envelope` |
| `sha256-<digest>.sig` | **404**, on every release |
| tags in the repository | ten; **none** ends in `.sig` |

`containers/image` — what `bootc --enforce-container-sigpolicy` uses — looks for
the tag `sha256-<digest>.sig` carrying a layer of
`application/vnd.dev.cosign.simplesigning.v1+json` with a
`dev.cosignproject.cosign/signature` annotation. The two forms share nothing but
the subject digest.

So task 7's measured *A signature was required, but no signature exists* was
**literally true**. Not a bad key, not a missing transparency log, not the wrong
policy type. And `pinned.toml`'s per-release *verified against
`signing/alo-os.pub`; a different key was refused with Found: 0, Expected 1* is
`cosign` reading `cosign`'s own bundle — a real check of the key and of the
owner's process, and no evidence that any machine can verify anything. Those
sentences are corrected in the same change as this file.

## The measurement

cosign pinned to **v3.1.3** — the version `pinned.toml` records for all five
releases — against a local registry, reproducing the owner's invocation: by
digest, no transparency log, an EC P-256 `cosign generate-key-pair` key.

| invocation | tag produced | layer |
|---|---|---|
| default | `sha256-<digest>` | `…sigstore.bundle.v0.3+json` |
| `--registry-referrers-mode=legacy` | `sha256-<digest>` | `…sigstore.bundle.v0.3+json` |
| `COSIGN_DOCKER_MEDIA_TYPES=1` | `sha256-<digest>` | `…sigstore.bundle.v0.3+json` |
| **`--new-bundle-format=false`** | **`sha256-<digest>.sig`** | **`…cosign.simplesigning.v1+json`** |

The default reproduces the published form exactly, which is what confirms this is
the owner's invocation and not a guess about it.

## The options

**1. The release signs in the readable form** — add `--new-bundle-format=false`.
One argument. Nothing about any image changes, the base is untouched, and ADR
0036's *a machine builds, a person signs* is unaffected.

**2. The machine reads the bundle form** — a `containers/image` new enough to
verify Sigstore bundles. That is the **pinned rented base**, and
[ADR 0011](0011-engines-are-configured-never-written-in.md) says engines are
configured, never patched: this means a newer base or a patch, and either needs
its own ADR. **Not proposed here**, and deliberately not measured — whether any
base version can read bundles belongs in the build on x86_64, not in a reading of
release notes.

**3. Sign both, and keep signing both.** ✅ **Recommended.** One extra
invocation. The two attachments live at **different tags** — `sha256-<digest>`
and `sha256-<digest>.sig` — so they cannot collide, which is measured rather
than assumed: signing both ways against one digest produced both tags with
neither disturbing the other. It keeps option 2 available later without a second
migration, and it costs nothing a machine has to understand, because each reader
looks only for its own form.

### What option 3 costs, and how the cost is removed

Two signatures over one digest must stay consistent, `pinned.toml` records one
line, and there is a **window in which the two could disagree** — somebody runs
the first command and not the second, and a digest ends up half-signed. Trusting
whoever signed to have run both is exactly the producer-side check `LOOP.md` now
warns against.

So **the build checks it**: that both attachments exist for the pinned digest and
that both verify against `image/signing/alo-os.pub` over the **same** digest. A
release half-signed fails the build rather than shipping as a machine that can
verify nothing. That check is the consumer's side of this decision and is where
the real guarantee lives.

## The migration, which needs no new release

**The five published images are fine.** What is missing is a signature in the
readable form over the same bytes. A signature is an attachment, not part of the
image: adding one changes nothing about the image and moves no digest.

So the owner can sign **the digest that is already pinned** —
`0.0.5`, `sha256:6c9abbc5a6a0f5299991f4cca65152452b3cbae339b161059528d72f2aad3ba1` —
again, with the flag, and a second attachment appears at a second tag. **The
currently pinned release becomes verifiable by a machine without rebuilding or
republishing anything.**

That matters beyond tidiness: it turns *no release can be verified* into *one
command, and the one that matters is verifiable*. Task 8 of the keeps-itself plan
can then be met on the current image rather than waiting for the next one, which
is the difference between a release away and a command away.

Older releases can be back-signed the same way, one command each, or left as
they are — nothing pulls them.

## The four traps, none recoverable from the tool's own documentation

Recorded because each cost an hour and the next person would spend the same.

1. **`--new-bundle-format=false` is not in `cosign sign --help`.** Grepping the
   help for it returns nothing. It was found only because `cosign sign-blob`
   errors with *must specify `--bundle` with `--new-bundle-format`* — an
   unrelated command failing for an unrelated reason is what revealed the name.
2. **`--registry-referrers-mode=legacy` is a red herring.** It is in the help, it
   says *legacy*, and it changes nothing about the format. Anybody reasoning from
   the documentation tries it first.
3. **`COSIGN_DOCKER_MEDIA_TYPES=1` is the second red herring**, for the same
   reason and with the same result.
4. **`--tlog-upload=false` is deprecated in 3.1.3 and now conflicts with the
   default `--use-signing-config=true`**, so the command fails outright with a
   message about signing configs. Reproducing *no transparency log* needs
   `--use-signing-config=false` beside it. This is a live trap for the owner's
   next signing run whatever is decided here, which is why the command with
   every flag it needs is printed by `.github/workflows/image.yml` at the moment
   the owner is told to sign, rather than living in this file.

## What this lane did not determine

- **Whether any `containers/image` version can read Sigstore bundles.** Option 2
  rests on it and is not proposed. It is a question for the build, on x86_64.
- **Whether the boot environment fetches the attachment at all.** Verification
  needs that tag pulled, and a staging road that can fetch an image but not its
  attachment fails where nobody is watching. This matters more now, not less,
  because the signature is a referrer.

Neither is asserted in either direction.
