# Who signs the image, and the release the recipe names first

**Date:** 2026-09-14
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`,
task 1 — *The image is published from GitHub, signed, and pinned*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-b`, for the owner
**Status:** ready for integration, **as the decision this task waits on**. The
publish itself is not done, and the plan does not say it is.

## What happened, plainly

The task asks for the image to be pushed to `ghcr.io/aloworld-org/alo-os`,
signed with `cosign` with the public key in the repository, and its digest
pinned in a file `alo-image` reads. None of that can be done by a worker, for
two reasons found on the machine that builds the image, on 2026-09-14:

- **No way to publish.** WSL's Ubuntu has podman and local builds
  (`localhost/alo-os:dev`, 6.61 GB, two days old — older than the recipe on
  `main`, so not an image anybody should pin), but there is no `cosign`, no
  login to `ghcr.io` in podman's or Docker's configuration, and no `gh`.
- **Nobody has decided who holds the signing key.** ADR 0011 says the key is
  *ours*; nothing says who generates it, where the private half lives, what
  signs with it, or whether the loop may publish. A worker could install
  `cosign`, make a key pair and pick its password, and the public half would
  become the root of trust for every machine alo OS is installed on without
  anybody having decided that. That is not a worker's call.

A pin written before a real push would be a digest nobody can pull, and a task
marked done over it would send task 2 at a registry that is empty. So, as the
worker instructions direct when the way forward runs through something a
worker may not decide, the decision is the work.

## What changed

- **`docs/decisions/0036-the-image-is-signed-by-a-key-a-person-holds.md`**
  (proposed). Three options — a key the owner generates and holds with a person
  signing by digest; a key as a GitHub Actions secret; keyless Sigstore — with
  the consequences of each. **Recommends A**: public half at
  `image/signing/alo-os.pub`, private half never in the repository, on a loop's
  disk or in CI, signing by digest without a public transparency-log upload, and
  no agent or build loop ever holds the key or publishes. It writes down the
  five steps the owner takes once, and why the pinned digest and the signature
  are two layers worth keeping (a stolen key cannot redirect an installer
  already released; a changed registry cannot get past the boot environment).
- **`image/Containerfile`** names its release:
  `LABEL org.opencontainers.image.version="0.0.1"`. It is the one piece the
  owner's push needs before anything is built — the image that is pushed and
  pinned has to be able to say which release it is. A label, not a change to
  anything the machine runs (ADR 0011 holds).
- **`crates/alo-image/src/version.rs`** — `TheVersion` reads that label; a
  version is exactly `MAJOR.MINOR.PATCH`; two labels read as none.
  `checking.rs` gains `the_image_names_the_release_it_is`, and `wrong.rs` gains
  `Wrong::TheImageDoesNotNameItsRelease`, which cites ADR 0033.
  `Image::version()` exposes it; `lib.rs` exports `TheVersion` and
  `THE_VERSION_LABEL`.
- **`crates/alo-image/tests/how_the_image_is_published.rs`** — the recipe names
  one release; ADR 0036 is recorded, proposed, and recommends the key a person
  holds; the plan's task 1 is blocked on it and not marked done.
- **`docs/autonomy/v0-5-the-installer-plan.md`** — task 1's status is now
  *blocked on ADR 0036 and the owner's first publish*, with what was found, what
  landed, and what the next worker does (launch nothing until both exist).

**Change description for the changelog:** *The image now says which release it
is, in the standard OCI label, and a check refuses a recipe that does not. How
alo OS images will be signed — and that a person, not an automated process,
holds the key and publishes — is written up as a proposed decision for the
owner; publishing the image waits on it.*

## Decisions taken here, and why

- **The ADR route rather than a partial implementation.** The pin file, the
  doc section and the workflow can only be finished against a real digest, and
  the workflow's shape (does CI sign?) is the decision itself. Writing them now
  would be either stubs or guesses at the owner's answer.
- **Version `0.0.1`.** It matches every crate in the workspace and v0.01; three
  numbers because a registry tag is movable and `latest`/`dev` are moved on
  purpose.
- **A literal label, not a build argument.** One spelling in one place, which
  is how `alo.disk.*` is already written and what a reader of the recipe sees.
- **The public half's path, `image/signing/alo-os.pub`,** is named in the ADR so
  the path is chosen once, beside the recipe it verifies and outside what the
  recipe copies into the image today.
- **No workflow file yet.** Under option A the workflow pushes and never signs;
  under B it signs. Writing it before the answer would be writing the answer.
  A workflow on `main` that tries to build a 9 GB image on every push would
  also be a red run the owner never asked for.

## Acceptance criteria and what shows them

| Criterion (this handoff) | Test |
|---|---|
| The recipe names exactly one release, as the OCI label | `alo-image` `how_the_image_is_published` `the_recipe_names_the_one_release_a_pin_is_held_against` |
| A recipe with no release, a moving one, or two is refused | `alo-image` lib `checking::tests::an_image_that_does_not_name_one_release_is_caught` |
| A version that moves or is malformed is not a release | `alo-image` lib `version::tests::a_word_that_names_whichever_build_came_last_is_not_a_release` |
| A release stated twice is stated by nobody | `alo-image` lib `version::tests::a_release_stated_twice_is_not_stated` |
| The key decision is recorded, proposed, and recommends a person-held key | `alo-image` `how_the_image_is_published` `the_decision_on_who_holds_the_signing_key_is_recorded_and_waits_on_the_owner` |
| The plan does not call the publish done while it waits | `alo-image` `how_the_image_is_published` `the_plan_does_not_mark_the_publish_done_while_it_waits` |

The plan's own acceptance for task 1 — pushed, signed, pinned, documented — is
**not met** and is not claimed.

## Verification

Run in WSL Ubuntu (root), from `/mnt/c/dev/alo-os-b`, with the supervisor's
target directory:

    cargo fmt --all
    cargo clippy --all-targets -- -D warnings
    cargo test -p alo-image
    cargo test -p alo-citing

Executed 2026-09-14, all exit 0:

    cargo fmt --all -- --check                  exit 0
    cargo clippy --all-targets -- -D warnings   exit 0, no warnings
    cargo test -p alo-image
      lib                              171 passed; 0 failed
      how_the_image_is_published         3 passed; 0 failed
      what_the_image_owes_the_daemons   25 passed; 0 failed
    cargo test -p alo-citing            21 + 10 passed; 0 failed

Each test in the table above was also run alone with `--exact`: 1 passed each.

Not run here: the full workspace suite (the supervisor's), and nothing on the
Windows side — `alo-installer` does not exist yet and was not touched. No
image was built, pushed or signed; that is the owner's step below.

## Remaining, and what the owner needs to do

1. Read ADR 0036 and accept, amend or reject it.
2. If A: install `cosign`, generate the pair on your own machine, commit the
   public half at `image/signing/alo-os.pub`, log podman in to `ghcr.io` with a
   token that can write packages, and do the five steps — build at a published
   commit of `main`, push `ghcr.io/aloworld-org/alo-os:0.0.1`, sign the digest,
   verify with the committed public half, and hand the version and digest over.
3. Make the `alo-os` package on GitHub public, so an installer pulls without an
   account.
4. Then task 1's repository half is a worker's: `image/` pin file, the
   version-and-pin agreement test, `docs/booting.md`'s registry, tag and
   `bootc install` invocation held to the code, and the workflow written with
   the reason it is not yet the road.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **ROADMAP.md:** nothing moves. *Installer* stays unbuilt.
- **QUEUE.md / STATE.md:** installer task 1 blocked on ADR 0036 (owner) and the
  first publish; tasks 2–5 transitively blocked behind it.
