# The published image, pinned by digest and held to the recipe

**Date:** 2026-09-15
**Workstream:** v0.5 installer (`docs/autonomy/v0-5-the-installer-plan.md`,
task 1 — *The image is published from GitHub, signed, and pinned*)
**Contributor:** Claude Code worker in `C:\dev\alo-os-shell`, for the owner
**Status:** ready for integration. The owner's half (build, push, sign, verify)
was done on 2026-09-15 and handed over; this is the repository's half, and it is
finished. One thing remains the owner's and is named below: making the package
public.

## What changed, plainly

alo OS 0.0.1 is in the registry, signed by the owner. Until this change the
digest lived only in the plan's prose. Now it is written in one file, and the
repository checks that everything else agrees with it.

- **`image/pinned.toml`** — the one file the digest is written in: registry
  `ghcr.io/aloworld-org/alo-os`, version `0.0.1`, digest
  `sha256:d3f05b60…a13c`, the full revision it was built from, and the path of
  the public key (`signing/alo-os.pub`). It sits beside the recipe and is not
  copied into the image. An image cannot carry its own digest.
- **`crates/alo-image/src/pinned.rs`** — `ThePin` reads that file strictly.
  It refuses these outright (`NotPinned`, in `refusing.rs`):
  - a digest that is not `sha256:` plus 64 lowercase hex characters;
  - a version that is not `MAJOR.MINOR.PATCH`;
  - a revision that is not a full 40-character commit;
  - a key path that is absolute or climbs out of the image directory;
  - a key it does not know, or a missing key.

  `ThePin::reference()` is always `registry@digest`, never the tag. `Image::at`
  reads the pin and the key file it names. A missing or malformed pin means the
  directory is not an image.
- **`crates/alo-image/src/publishing.rs`** — the checks, called from
  `everything_wrong_with`. Each has a new `Wrong` variant in `wrong.rs`:
  - the pin names the registry ADR 0033 decided;
  - **the recipe's `org.opencontainers.image.version` equals the pinned
    version**, so a disagreement fails a test. The one allowed exception is a
    `next` release declared in the pin (see the decisions below);
  - a declared `next` must come after the pinned version, compared as numbers;
  - the key file is exactly one PEM public key and holds no private key;
  - `docs/booting.md` has an *Installing the published image* section whose
    `registry`, `tag` and `digest` match the pin;
  - that section's `bootc install` command pulls the pinned reference;
  - no command names the registry by a tag or by any other digest;
  - the first command that touches the pinned image is
    `cosign verify --key image/signing/alo-os.pub`, so nothing is fetched or
    written before verification.
- **`crates/alo-image/src/booting.rs`** — three more facts (`registry`, `tag`,
  `digest`). `TheDocument::commands()` joins lines ending in `\` into the whole
  command a person types, so a check can ask what one command pulls.
- **`docs/booting.md`** — the new section *Installing the published image*. It
  gives:
  - the three facts;
  - the `cosign verify` command (with `--insecure-ignore-tlog=true`, and why);
  - `podman pull` by digest;
  - the one `bootc install to-disk` command, which runs the pinned digest and
    writes to a loopback file, never a real disk;
  - a note that a pull needs a login today because the package is private.
- **`.github/workflows/image.yml`** — builds `image/Containerfile` and pushes a
  **candidate** to `ghcr.io/aloworld-org/alo-os:<version>`, printing the digest.
  - It runs only when a person starts it, and only from main.
  - It refuses a version whose tag already exists (`skopeo inspect` before the
    push).
  - It never signs.
  - Its header says why it is not yet the road: 4.5 GB of weights, a 9.6 GB
    disk and roughly 14 GB free on a standard runner.
- **`crates/alo-image/src/workflow.rs`** — `TheWorkflow` reads that file line
  by line and says:
  - whether it runs only when asked;
  - whether it builds only from main;
  - whether it signs (`cosign sign`, or any `COSIGN_` variable);
  - whether it pushes to the decided registry;
  - whether it looks before it pushes;
  - whether it records its reason.

  Comments are ignored, so the sentence *only the owner signs* is not read as
  signing.
- **`crates/alo-image/tests/how_the_image_is_published.rs`**:
  - the shipped pin carries the signed digest, the recipe's release and the
    committed key, and the whole image has nothing wrong with it;
  - the shipped workflow keeps every rule;
  - the plan marks task 1 done and names `image/pinned.toml`. This replaces the
    test that required the task *not* to be marked done.
- **`docs/autonomy/v0-5-the-installer-plan.md`** — task 1 is now
  **Done, 2026-09-15**. Task 2 comes after it, so no new task was written.

**Change description for the changelog:** *The published alo OS 0.0.1 image is
now pinned by digest in `image/pinned.toml`. The repository checks that the
recipe names the same release, that the committed key is one public key, and
that `docs/booting.md` verifies the signature before installing that exact
digest. A workflow for pushing future images is in place. It only pushes
unsigned candidates when started by hand, and says why it is not yet how
releases are made.*

## Decisions taken here, and why

- **Strict equality, plus an explicit `next`.** The plan says a test fails if
  the recipe's version and the pin disagree. Taken literally at every commit,
  that creates a deadlock. ADR 0036 step 1 builds at a *published commit of
  main*, and that commit's recipe must already name the new release. But such a
  commit cannot pass the gates while the pin still names the old one, so 0.0.2
  could never be built.

  So a disagreement is allowed only when it is *written down*: the change that
  moves the label to `0.0.2` also adds `next = "0.0.2"` to the pin. The change
  that pins the new digest removes it.
  - An undeclared disagreement is still refused, in either direction.
  - So is a `next` the recipe does not name.
  - So is a `next` that does not come after the pinned version.

  Installers always pull `digest`, never `next`.
- **The pin is read strictly, the recipe leniently.** A malformed digest is not
  a pin with a fault in it. An installer handed one would pull whatever it made
  of it.
- **The pin names its key.** The boot environment (task 2) and the installer
  (task 3) need both the reference and the key, and one file gives both. The
  path is kept inside the image directory so nothing can point verification at
  a file somebody else placed.
- **The documented install writes to a loopback file.** This matches the
  command already proven on 2026-09-11, and it takes no destructive step. Only
  the image reference changed.
- **The workflow runs only when started by hand.** A workflow triggered on every
  push to main, for a build that does not fit on a runner, would be a red run
  nobody asked for. If it is started, it does real work: it pushes a candidate
  and hands over the digest. It is not a stub. It adds the
  `org.opencontainers.image.revision` label, as the owner's own push did.
- **The workflow checks are in the crate, not in a YAML parser.** They read
  lines, as `unit.rs` does. Each rule is a method with a refusal test.

## Acceptance criteria and what shows them

| Criterion | Test |
|---|---|
| The workflow is written, pushes a candidate, never signs, and records why it is not the road | `alo-image` `how_the_image_is_published` `the_workflow_pushes_a_candidate_never_signs_and_says_why_it_waits` |
| A workflow that signs is refused | `alo-image` lib `workflow::tests::a_workflow_that_signs_is_caught` |
| A workflow that runs without being asked is refused | `alo-image` lib `workflow::tests::a_workflow_that_runs_without_being_asked_is_caught` |
| The public key is in the repository and no private key is | `alo-image` `how_the_image_is_published` `the_public_half_is_committed_and_the_private_half_is_not` |
| A key file that is not one public key is refused | `alo-image` lib `publishing::tests::a_key_that_is_not_one_public_key_is_caught` |
| The signed digest is pinned in one file `alo-image` reads, and the image agrees | `alo-image` `how_the_image_is_published` `the_signed_digest_is_pinned_and_the_image_agrees_with_it` |
| A digest that is not one is refused | `alo-image` lib `pinned::tests::a_digest_that_is_not_one_is_refused` |
| A test fails if the recipe's version and the pin disagree | `alo-image` lib `publishing::tests::a_recipe_naming_a_release_the_pin_does_not_is_caught` |
| …in either direction | `alo-image` lib `publishing::tests::a_pin_naming_a_release_the_recipe_does_not_is_caught` |
| A declared next release the recipe does not name is refused | `alo-image` lib `publishing::tests::a_declared_next_release_the_recipe_does_not_name_is_caught` |
| `docs/booting.md` gives the registry, tag, digest and the one `bootc install`, held to the pin | `alo-image` lib `publishing::tests::the_shipped_pin_agrees_with_the_recipe_the_key_and_the_document` |
| A document pulling by tag is refused | `alo-image` lib `publishing::tests::a_document_pulling_by_the_tag_is_caught` |
| A document that installs before verifying is refused | `alo-image` lib `publishing::tests::a_document_that_writes_before_it_verifies_is_caught` |
| The plan marks the task done | `alo-image` `how_the_image_is_published` `the_plan_marks_the_publish_done_with_the_digest_it_pinned` |

Other refusal tests beside these:
- `pinned::tests` — release, revision, key path, shape;
- `publishing::tests` — another registry, another digest, another tag, a
  missing section, a `next` that does not follow, a recipe with no release;
- `workflow::tests` — any branch, pushing without looking, another registry, a
  missing reason;
- `booting::tests::a_command_across_lines_is_one_command`.

**Mutation check:** with `the_pin_is_the_recipes_release` and
`the_document_installs_what_is_pinned` switched off, 10 of the 15
`publishing::tests` failed. They passed again once restored.

## Verification

Run in WSL Ubuntu (root) from `/mnt/c/dev/alo-os-shell`, with the build
directory
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-shell-cd217193b5311c25`.
Executed 2026-09-15:

    cargo fmt --all -- --check                    exit 0
    cargo clippy --all-targets -- -D warnings     exit 0, no warnings (whole workspace)
    RUSTDOCFLAGS="-D warnings" cargo doc -p alo-image --no-deps   no warnings
    cargo test -p alo-image
      lib                              202 passed; 0 failed
      how_the_image_is_published         6 passed; 0 failed
      what_the_image_owes_the_daemons   25 passed; 0 failed
    cargo test -p alo-citing            21 + 10 passed; 0 failed

Every test in the table above was also run on its own with
`--exact --include-ignored`: 1 passed each. On Windows, `cargo test -p alo-image
--lib` also passes (201 tests).

**Not run here:**
- **The full workspace suite.** The supervisor runs it.
- **`alo-installer`.** It does not exist yet, and nothing on the Windows side was
  touched, so there is no installer test output to paste.
- **The documented `cosign verify` / `podman pull` / `bootc install` against the
  registry.** This machine has no `cosign`, and the package is private. The
  owner ran the `cosign verify` (and the different-key refusal) on 2026-09-15.
  Pulling by digest and installing from the registry have not yet been watched.
  Task 2's VM test is where they will be. If `bootc install` turns out to behave
  differently when run from a registry reference by digest, that goes in
  `docs/quirks.md` in that change.
- **The workflow.** It has never run. It is dispatch-only by design, and the
  reason is in its header.

## Remaining, and what the owner needs to do

1. **Make the `alo-os` package on `ghcr.io` public**, so an installer can pull
   without an account. `docs/booting.md` says a login is needed until then.
2. When 0.0.2 is cut, the same commit moves the recipe's label and adds
   `next = "0.0.2"` to `image/pinned.toml`. After the push, signing and
   verification, the pinning change sets `version` and `digest` and removes
   `next`.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **ROADMAP.md:** nothing moves. *Installer* stays unbuilt until tasks 2–5.
- **QUEUE.md / STATE.md:** installer task 1 done; task 2 (the boot environment
  that installs, tested in a VM) is next and unblocked.
