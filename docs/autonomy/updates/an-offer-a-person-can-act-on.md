# An offer a person can act on

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 7
**Contributor:** this development PC's worker lane
**Status:** ready for integration

## What this was about

Task 6 left a gap and named it. A machine running the pinned release is offered
whatever the place offers, and *an update is ready* is a sentence a person acts
on — but the act stages under `--enforce-container-sigpolicy` (ADR 0036), and a
build the policy will not have is refused there. A person told an update is
ready and then told it could not be prepared has been told two true things and
learned nothing.

This task closes that on the machine's side, and measures what actually happens
on the real place and a real base. The measurement turned out to be the most
important thing in the change.

## The decision, and the one discarded

The plan left two ways open and asked for both in writing.

**Chosen — the offer carries the doubt.** A build nothing at the place vouches
for is still offered; what changes is the sentence. `Standing::said` answers
`keeping-up.ready-not-vouched-for` instead of `keeping-up.ready`:

> A new version of this machine's system is available, but this machine cannot
> confirm that it came from alo OS. It will not be applied unless it can be
> confirmed

**Discarded — narrowing the offer to what the place vouches for.** It reads
safer and is worse, for three reasons:

1. A machine running an old release would read *this machine is up to date*
   while a newer one sat at the place. Task 1's rule is that the choice a person
   has is *when*, never *whether to be told*; hiding a version that exists is
   that rule turned inside out, and a person has no way to discover it.
2. It would hide the window between a release being pushed and being signed from
   the only people who can close it — which is not hypothetical, as the
   measurement below shows.
3. A fleet sitting quietly on an old build because a signature went missing is a
   worse failure than a cautious sentence, and a harder one to notice.

**What neither option does.** Nothing here decides that a build *may* be staged.
Vouching is read from a name at a place: it fetches nothing, verifies nothing
and permits nothing. The machine's signature policy remains the only authority,
asked by the base at the moment of staging. That line is drawn deliberately —
a second, weaker verifier that looked authoritative is the thing somebody later
trusts instead of the policy.

## What changed

### `crates/alo-keeping-up` — the decision

- **`src/vouching.rs`** (new). `Vouching` is two members,
  `ThePlaceVouchesForIt` and `NobodyHasVouchedForIt`. A third cannot arrive
  without the file changing: *probably*, *partly* and *not checked yet* are each
  a way of telling a person something they cannot act on. Missing is always read
  as the second, because the two mistakes do not cost the same.
- **`src/checking.rs`**. `Offered::heard` takes the vouching as an argument
  rather than gaining it afterwards — the value a caller gets by forgetting it
  is the comfortable one, and a question whose comfortable answer is its default
  is a question that has to be asked.
- **`src/standing.rs`**. `Ready` carries it; `Standing::said` says the other
  sentence for an offer nobody vouched for. An update still carries no priority,
  severity or deadline: the serialised shape is four fields and a test names
  them.
- **`src/words.rs`**. Two words, both with translator's notes:
  `keeping-up.ready-not-vouched-for` and `keeping-up.not-genuine`.
  `keeping-up.not-prepared`'s note no longer claims to cover *what arrived was
  not a genuine alo OS*, which is now its own sentence.

### `crates/alo-looking` — reading it off the place

- **`src/vouching.rs`** (new). `vouched_for(build, names)` reads the answer out
  of the names the place **already answered with** during the one check that was
  on the indicator: no third question, no second departure. It counts exactly
  one name, `sha256-<the build>.sig` — the attachment the machine's own base
  would read. Why only that one is the measurement below.
- **`src/looking.rs`**. `look` computes it and hands it to `Offered::heard`.
- **`src/kept.rs`**. The kept answer carries it too, so a surface reading it back
  says what the check said. An older answer with nothing written about it reads
  as *nobody vouched for it*.

### `crates/alo-updating` — the refusal a person meets

- **`src/genuine.rs`** (new). `refused_for_its_signature` tells the signature
  policy's refusal apart from every other way preparing an update can fail, by
  reading what the base said — because the base gives no other sign. Two rules
  keep it honest: every marker is a whole clause, never the word *signature*
  (the base prints *Getting image source signatures* on the way to a
  **success**, and a test holds it to that exact line); and what it does not
  recognise it does not guess at, which stays *the update could not be prepared*.
- **`src/refusing.rs`**, **`src/applying.rs`**. `NotApplied::TheBuildWasNotGenuine`
  is its own member with its own sentence.

## The measurement

### The place, 2026-09-19

`cargo test -p alo-looking --test against_the_real_registry -- --ignored --nocapture`,
four tests, 2.64 s:

```text
it holds 7 names: ["0.0.1", "sha256-d3f05b60…", "0.0.2", "sha256-8f9c36e0…",
                   "0.0.3", "sha256-41d43c7e…", "0.0.4"]
the newest release it offers is 0.0.4
it says 0.0.4 is sha256:48bd5f319abcecfa832eb9a5b0b2f7cd06815b1c30c43b781499500ec14c3858
it holds sha256-48bd5f31….sig for that build: NobodyHasVouchedForIt
this machine stands: A new version of this machine's system is available, but
this machine cannot confirm that it came from alo OS. It will not be applied
unless it can be confirmed
and the pinned release 0.0.3 is sha256:41d43c7e…
```

So: a machine at the pinned `0.0.3` is offered `0.0.4`, which the owner has
pushed and nothing at the place vouches for, and it is **told about it** with the
cautious sentence rather than left to find out after choosing.

### The base, 2026-09-19

`cargo test -p alo-updating --test an_offer_a_person_can_act_on -- --ignored --nocapture`,
3.72 s. It runs the base's **own** `skopeo` 1.22.2 and `containers-common`
0.67.0, out of `quay.io/fedora/fedora-bootc:42@sha256:077182b6…` — the image
`image/Containerfile` pins, whose `bootc` 1.15.1 stages a build with that exact
library — against the real place, under a policy requiring
`image/signing/alo-os.pub`:

```text
=== ghcr.io/aloworld-org/alo-os:0.0.4
level=fatal msg="Source image rejected: A signature was required, but no signature exists"
=== ghcr.io/aloworld-org/alo-os@sha256:41d43c7e…
level=fatal msg="Source image rejected: A signature was required, but no signature exists"
```

Both refusals arrive **before a single layer is fetched**, so the machine is
unchanged, and the test asserts that too (`Copying blob` never appears).

## What was found, and who it belongs to

**No alo OS release published today can be staged under
`--enforce-container-sigpolicy`** — including `0.0.3`, the one
`image/pinned.toml` pins, which the owner signed with `cosign` 3.1.3 exactly as
ADR 0036's procedure describes.

The mechanism: `cosign` 3 publishes a signature as an **OCI 1.1 referrer**. On a
registry without the referrers API — `ghcr.io` among them — that lands under the
fallback tag `sha256-<the build>`, with nothing after it, holding an index whose
one manifest is `artifactType: application/vnd.dev.sigstore.bundle.v0.3+json`. A
real signature, and the place holds it.
`containers/image` does not look there: a `sigstoreSigned` policy fetches the
**attachment** `sha256-<the build>.sig`, which `cosign` 3 does not write unless
it is told to.

**This is why `vouched_for` counts only the `.sig` name.** The obvious reading —
count `sha256-<the build>`, which is right there in the list and is a genuine
signature — would have made this machine say *an update is ready* about a build
its own base refuses. That is the failure this task exists to end, arriving
through the back door.

### For the installer lane (ADR 0036)

Two things, neither of them ours to do, both handed over here:

1. **The release process should publish the signature where the policy looks.**
   `cosign sign --registry-referrers-mode=legacy` (or the equivalent attachment
   mode) writes `sha256-<build>.sig` beside the build, which is what
   `containers/image` fetches. Verifying with `cosign verify` will keep working;
   what changes is that the machine can verify too. Whichever way it is done,
   the check that it worked is cheap and is already written:
   `alo_looking::vouched_for` against the real place, and the `#[ignore]`d test
   in `alo-updating` that asks the base.
2. **The shipped image still carries no `policy.json`** — task 2 handed this over
   on 2026-09-15 and it is still open. Until both land, `apply` refuses every
   real release, safely and with a clear sentence, which is the right direction
   to fail in and not one to leave a fleet in.

Nothing was changed at the place, in `image/`, or in the release procedure by
this task. The plan forbids it and so does the ADR.

## What a person reads, added by this change

| When | Key | Sentence |
|---|---|---|
| A newer version exists and nothing vouches for it | `keeping-up.ready-not-vouched-for` | A new version of this machine's system is available, but this machine cannot confirm that it came from alo OS. It will not be applied unless it can be confirmed |
| They chose it anyway and the machine refused it | `keeping-up.not-genuine` | This machine could not confirm that the new version came from alo OS, so it was not installed and nothing was changed. The next restart starts this machine as it is now |

Both are in the machine's one vocabulary with a translator's note, both are
reachable from a public `said()`, neither names the machinery, and the second
says the machine is as it was — each checked by a test in
`crates/alo-updating/tests/what_a_person_is_told.rs`, which was task 5's and now
covers these two as well.

## Verification

Run from this checkout's Linux side (`/root/alo-trees/this-machine`, the copy
the gates read), Ubuntu 24.04 in WSL2 on Windows Server 2022.

| What | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets --workspace -- -D warnings` | clean, 31 s |
| `cargo test -p alo-keeping-up` | 74 + 10 + 6 + 9 passed |
| `cargo test -p alo-looking` | 64 + 17 + 5 passed, 4 ignored |
| `cargo test -p alo-updating` | 5 + 10 + 9 + 7 + 4 + 6 passed, 1 ignored |
| `cargo test -p alo-saying` | 63 + 4 + 1 passed |
| `cargo test -p alo-looking --test against_the_real_registry -- --ignored` | 4 passed, 2.64 s, output above |
| `cargo test -p alo-updating --test an_offer_a_person_can_act_on -- --ignored` | 1 passed, 3.72 s, output above |

**Not run here:** the whole workspace suite, which the supervisor runs after
this; and any measurement on a real boot — see the limitation below.

## Limitations

- **The policy decision was measured, the staging was not.** What was asked is
  `containers/image` — the library the base stages with, at the base's own
  version, out of the base's own image — rather than `bootc` on a booted
  machine. It is the component that makes the decision, and it made it against
  the real place with the real key; it is not a certified machine and does not
  claim to be. A real boot is task 8 of the plan, and it is blocked on the
  release process, because there is no release today that would get past the
  policy for it to measure.
- **Telling a refusal apart by what a program said** is weaker than an exit code
  and is written down as such in `docs/quirks.md`. What is not recognised is not
  guessed at.
- **`cosign verify` was not run** — `cosign` is not on this machine — so this
  report does not claim the referrer bundle is a valid signature by the owner's
  key. It claims what it measured: that the bundle is there, that it is a
  sigstore bundle, and that the base's policy does not read it.

## Proposed changes to the shared documents

The integration owner's to make; nothing here edits them.

**`CHANGELOG.md`**, under v0.5:

> **An update a person can act on.** When a newer version of alo OS exists that
> this machine cannot confirm came from alo OS, it now says so *before* the
> person chooses, instead of offering the update and refusing it afterwards. And
> when the machine does refuse a version because it could not confirm where it
> came from, that is its own sentence rather than the same line a failed
> download gives — so a person learns the one thing they can act on. Measured
> against the real place alo OS builds come from and the real system underneath:
> a version published but not yet countersigned is offered honestly, and a
> machine refuses to install it, unchanged.

**`ROADMAP.md`**: *updates that never interrupt* gains its last decidable piece;
nothing is ticked on the machine. The v0.5 line cannot be called complete while
no published release can be staged — see the finding above.

**`docs/autonomy/QUEUE.md`**: task 7 of the machine-keeps-itself plan done; task
8 added and **blocked** on the installer lane's release process.

**`docs/autonomy/STATE.md`**: reference this report, and carry the finding for
the installer lane so it is not lost between plans.
