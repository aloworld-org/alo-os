# ADR 0070 — The signature a machine can read, and the road that is closing

**Status:** proposed, 2026-09-26.

Written by the Mac lane from task 8 of
`docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, whose first half —
*the shipped image carries no `policy.json` at all* — turned out not to be
writable without answering this. A `policy.json` **is** the statement of what a
machine will accept, so its content is this question's answer and not a step
after it.

## The decision in one line

`cosign` 3 writes a signature no alo OS machine can read. One undocumented flag
makes it write one they can, and **that flag is itself deprecated with nothing
behind it** — so sign **both** ways now, and treat a base that reads the bundle
form as the exit rather than a tidy-up. The currently pinned release becomes
verifiable **without a rebuild or a republish**.

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
One argument, and **not durable.** cosign 3.1.3 prints, on every run:

> Flag `--new-bundle-format` has been deprecated, **this will be the only
> supported format in future versions.**

So the only road to a verifiable release is the one cosign is removing. When the
flag goes, **signing will still succeed** and publish a signature no alo OS
machine can read: nothing fails, nothing warns, and the first symptom is a
refused update months later. An earlier draft of this file recommended this
option as the safe one. That was wrong, and the measurement below is why.

**And there is no supported road behind it.** `--signing-config` — the
replacement cosign names for the other deprecated flag — **cannot produce the
classic form at all**: it refuses with *must provide `--new-bundle-format` or
`--bundle` where applicable with `--signing-config`*. Measured. So the readable
signature has **no non-deprecated path**, and that is not a tidiness problem; it
is a road closing with nothing behind it.

**2. The machine reads the bundle form** — **the only exit.** Given the
deprecation above, this stops being the elegant long-term option and becomes the
one road that still exists once the flag is gone — a `containers/image` new enough
to verify Sigstore bundles. That is the **pinned rented base**, which
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) rents under
`CLAUDE.md`'s rule that **engines are configured, never patched** — the rule is
the constitution's and ADR 0011 is where it was applied to the base. So this
means a newer base or a patch, and either needs its own ADR. **Not proposed here**, and deliberately not measured — whether any
base version can read bundles belongs in the build on x86_64, not in a reading of
release notes.

**3. Sign both, and keep signing both.** ✅ **Recommended now**, with option 2
as the **exit rather than the tidy long-term answer**. One extra invocation.

The two attachments live at **different tags** — `sha256-<digest>`
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

## The policy is two files, and the second is invisible

`containers/image` fetches a sigstore signature from a registry **only** when
that registry is configured for it. With `policy.json` alone, the **correctly
signed** 0.0.5 was refused — with *A signature was required, but no signature
exists*, **the same sentence a genuinely unsigned image gets.** Measured against
the real registry.

So a machine shipping the policy without
`registries.d/…use-sigstore-attachments: true` refuses every update while
telling the person the release is unsigned, and whoever debugs it goes and checks
the signature, the key and the registry — all of which are fine. The two ship
together in `image/Containerfile`, the negative case is held forever by
`crates/alo-image/tests/the_shipped_policy_accepts_and_refuses.rs`, and
`docs/quirks.md` carries the indistinguishable message in those words.

It also means the error is ambiguous three ways: unsigned, signed in the
unreadable form, or signed correctly and never looked for. Task 7's measurement
could have been any of the three.

## What the owner should see, from the consumer's side

`cosign verify` is the producer asking itself, and it said yes for five releases
while nothing could read a thing. The confirmation is:

- the tag `sha256-<digest>.sig` now returns 200 where it returned 404; and
- `skopeo --registries.d image/registries.d --policy <the shipped policy> copy
  docker://ghcr.io/aloworld-org/alo-os@<DIGEST> dir:/tmp/check` reaches *Copying
  blob* rather than *Source image rejected*.

That second one runs the same library `bootc` runs. **Use `copy`, never
`inspect`:** `skopeo inspect` does not apply the policy — it prints the config of
an image the policy would refuse and exits zero.

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

**Older releases are not being back-signed.** 0.0.1 to 0.0.4 carry only the
bundle form and the policy refuses them, correctly — measured: 0.0.4 is refused
with *A signature was required, but no signature exists*. They are superseded and
nothing pulls them, so re-signing history buys nothing. `pinned.toml` says so
plainly, because otherwise somebody pins one for a rollback or a bisect and loses
an afternoon to a refusal that is working as designed.

### Rollback, which this migration breaks until the next signing

0.0.5 is the **first** machine-verifiable release, so **it has nothing to roll
back to.** A machine that checks signatures will refuse 0.0.4, and *back to
yesterday's machine* is a promise in this release. This is a known consequence of
the migration rather than a fault in it: before it, nothing was verifiable and
rollback was equally unavailable to a checking machine; after it, one release is.

What resolves it is **the second signed release**, at which point there are two
verifiable images and a rollback has somewhere to go. That also corrects the
shape of task 8 of `../autonomy/v0-5-the-machine-keeps-itself-plan.md`: its
acceptance is *one signed release updating to another*, so it needs **two
signings, not one**. The migration makes the policy provable now; the task still
waits on the next release.

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

- **Whether any `containers/image` version can read Sigstore bundles.** This is
  no longer a side question: given the deprecation, it **decides the long-term
  signing story**, because option 2 is the only road that outlives the flag. It
  belongs in the build, on x86_64, and it is worth doing early rather than when
  the flag is removed.
- **Whether the boot environment fetches the attachment at all.** Verification
  needs that tag pulled, and a staging road that can fetch an image but not its
  attachment fails where nobody is watching. This matters more now, not less,
  because the signature is a referrer.

Neither is asserted in either direction.
