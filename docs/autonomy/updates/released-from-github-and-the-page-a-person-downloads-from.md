# Released from GitHub, and the page a person downloads from

**Date:** 2026-09-16
**Workstream:** `docs/autonomy/v0-5-the-installer-plan.md`, task 5
**Contributor:** Claude Code worker, checkout `C:\dev\alo-os-shell`
**Status:** ready for integration. Task 5 is marked **Done, 2026-09-16** in the
plan. One half of one acceptance line — *signs the executable* — is answered by
a decision record rather than by a workflow step, and that is said plainly
below.

## What a person gets, in one paragraph

Pushing a tag that names the pinned release builds the boot environment and the
Windows installer, puts them into one archive with the checksum of every asset
beside it, and creates a **draft** GitHub Release whose body is a file committed
in this repository. That file names the registry, the tag, the digest the
installer really pulls and the Secure Boot state it really accepts — each held
to `image/pinned.toml` and to `alo_installer::decide` by tests, so none of them
can drift into a sentence that is no longer true. The README now has a *Try it*
that says what a computer needs, that Windows stays, and one sentence about
Secure Boot, and every requirement in it is read out of `docs/hardware.md`'s own
table rather than written a second time. Nothing in the road leaves a person's
machine, and nothing in it signs.

## The thing that had to be decided: what signs the installer

The acceptance says the workflow *signs the executable*. Authenticode needs a
code-signing certificate; this repository has none, and nothing in it said who
should hold one. A `signtool` step calling a secret nobody has created is a step
that fails or silently skips, and a certificate an agent obtained would make the
publisher of alo OS somebody nobody chose.

So the decision is the work:
`docs/decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md`
(**proposed**, recommendation **A**) puts it the way ADR 0036 put the image's
key — **a machine builds, a person signs** — with B (a certificate in a
repository secret) and C (a Sigstore build attestation) written out and refused,
and with what it costs: until there is a certificate, the Release carries an
unsigned executable and says so, and no sentence anywhere claims a signature it
does not have. The repository half is implemented in this change, and
`crates/alo-image` fails this repository's build if the workflow ever signs or
reaches for a secret of any kind.

**The ADR was renumbered.** It was drafted as 0045, and `0045-what-undoing-
rewinds-to.md` landed on main first; two files claiming one number is refused by
`crates/alo-citing`, so it is **0046** here and every pointer at it was moved
with it.

**What stays the owner's**, and neither is a worker's: obtain the certificate,
and for each Release sign `alo-installer.exe`, put it back into the archive,
refresh `SHA256SUMS` and publish the draft. Accepting or amending ADR 0046 is
also theirs.

## What changed

| File | What it is |
|---|---|
| `.github/workflows/release.yml` | New. On a tag: refuse a tag `image/pinned.toml` does not name, build the environment, list it, build `alo-installer` with that list compiled in, archive, checksum, create a **draft** Release from the committed notes. Never signs, never reads a secret; its head comment records why signing is a person's step |
| `image/release-notes.md` | New. The Release's body, committed: registry, tag, digest and Secure Boot state in one indented block, with *Windows stays* and the Secure Boot sentence in prose |
| `crates/alo-image/src/notes.rs` | New. Reads those four facts and **every** `sha256:` digest anywhere in the text; a fact stated twice reads as stated by nobody |
| `crates/alo-image/src/released.rs` | New. Holds the notes to the pin and to `ACCEPTED`, and the README's *Try it* to `docs/hardware.md` |
| `crates/alo-image/src/releasing.rs` | New. Reads the release workflow for the nine things it must and must never do |
| `crates/alo-image/src/trying.rs` | New. Reads the README's *Try it* — its requirements, its paragraphs — and `docs/hardware.md`'s *What to buy first* table |
| `crates/alo-image/src/image.rs` | Reads `image/release-notes.md` beside the recipe and the pin; `Image::notes()` |
| `crates/alo-image/src/checking.rs` | `everything_wrong_with` now also asks whether the notes agree with the pin |
| `crates/alo-image/src/wrong.rs` | Ten new disagreements, each naming the promise it is about |
| `crates/alo-image/src/workflow.rs` | `live()` and `comments()`, so `releasing.rs` reads GitHub's file format through the one reader this crate has |
| `crates/alo-image/src/lib.rs` | The four modules, their public surface, and four rows in the crate's table |
| `crates/alo-image/tests/how_the_installer_is_released.rs` | New. The shipped workflow, the decision, the notes, the README and the plan's mark |
| `crates/alo-installer/tests/the_release_says_what_it_accepts.rs` | New. The Secure Boot state the notes promise, against what `decide` refuses on a machine in each of the three states |
| `crates/alo-installer/Cargo.toml`, `Cargo.lock` | `alo-image` as a **development** dependency, in that direction only |
| `README.md` | The *Try it* section |
| `docs/decisions/0046-…-a-person-holds.md` | New (renumbered from 0045) |
| `docs/autonomy/v0-5-the-installer-plan.md` | Task 5 marked **Done, 2026-09-16**, with what stays the owner's |

## Decisions I made, and why

- **The notes are a committed file, not typed into the workflow or the web
  form.** The digest in the notes is the digest in the pin; two spellings of it
  drift, and the one a person reads is the one nobody tests. `--notes-file
  image/release-notes.md` is held by a test, and `--notes ` is refused.
- **Four facts in an indented block, prose everywhere else.** Ordinary prose is
  full of colons; a reader that took every `key: value` line would read a
  sentence as a promise. But **every** `sha256:` in the whole document is
  collected, because a digest in a sentence is one a person will pull by, and
  notes naming two are notes nobody can act on.
- **The Secure Boot state has one home, `alo_image::ACCEPTED`, and is asked from
  the installer's end.** `alo-installing` depends on `alo-image`, so `alo-image`
  cannot ask `alo-installer` what it accepts without a cycle. The rule is
  written once and `crates/alo-installer/tests/the_release_says_what_it_accepts.rs`
  puts a machine into each of the three states Windows can report and holds the
  notes to what `decide` really does. When task 9 changes what is accepted, that
  test is what fails.
- **The README's requirements are read out of `docs/hardware.md`'s table, not
  copied.** A README saying 16 GB where the table says 32 is somebody buying the
  wrong machine on our word. The README may say *less* than a row (it says
  *UEFI, TPM 2.0* where the row says *UEFI, Secure Boot, TPM 2.0*, because this
  release does not install with Secure Boot on) and may never say something the
  table does not.
- **The *Try it* section is capped at four paragraphs, and Secure Boot may
  appear in exactly one.** The README's rule since 2026-09-13 is no noise, and
  the Secure Boot sentence is the one sentence somebody will one day make
  helpful. ADR 0033 §4 is absolute, so *turn it off*, *switch it off* and
  *disable* are refused in it however they are phrased.
- **The tag is refused before anything is built.** The notes, the pin and the
  environment each name one release; the tag is the only one of the four a
  person types, so it is the only one that can be wrong.
- **The Release is a draft.** A Release that exists is a Release somebody can
  download, and nobody has signed what is in it yet.

## Acceptance, line by line

| The plan says | Where it is |
|---|---|
| a workflow builds `alo-installer` for Windows on a tag | `.github/workflows/release.yml`; `releasing.rs::runs_only_on_a_tag`, `holds_the_build_to_the_environment` |
| signs the executable | **ADR 0046**: a machine builds, a person signs. The workflow never signs and is tested never to |
| publishes it as a Release asset with a checksum beside it | `writes_the_checksum_before_it_publishes`, `publishes_a_draft` |
| the notes say which image digest it installs | `image/release-notes.md`; `released.rs::everything_wrong_with_the_notes` |
| … and which Secure Boot state it accepts | the same, plus `crates/alo-installer/tests/the_release_says_what_it_accepts.rs` |
| the README gains a *Try it* … and nothing else | `README.md`; `everything_wrong_with_the_try_it` |
| a test refuses a Release whose notes name a digest the pinned file does not | `released::tests::notes_naming_another_digest_are_refused` and `…a_second_digest_in_prose…` |
| no telemetry, nothing leaves that the screen did not show | unchanged: nothing was added to the installer, and the workflow's only network calls are to GitHub's own artifact store and Release API |

## Verification

Windows 11, `C:\dev\alo-os-shell`, run in the foreground, exit codes waited for.

```
cargo fmt --all                                         (clean)
cargo clippy --all-targets -p alo-image -- -D warnings  Finished, 0 warnings
cargo doc -p alo-image --no-deps                        Generated, 0 warnings
cargo test -p alo-image
    lib          244 passed; 0 failed
    how_the_image_is_published        6 passed; 0 failed
    how_the_installer_is_released     6 passed; 0 failed
    what_the_image_owes_the_daemons  25 passed; 0 failed
cargo test -p alo-installer
    lib                                     44 passed; 0 failed
    reading_this_windows                     1 passed; 0 failed
    the_installer_checks_consents_and_stages 23 passed; 0 failed
    the_release_says_what_it_accepts          3 passed; 0 failed
cargo test -p alo-citing                    21 + 10 passed; 0 failed
cargo test -p alo-installing                36 + 14 + 5 passed; 0 failed
cargo test -p alo-accounts                  41 + 2 + 4 passed; 0 failed
cargo test -p alo-greeting                  21 + 7 + 5 + 4 + 3 passed; 0 failed
```

`cargo test -p alo-installer` is the run the plan's header asks a worker on this
Windows PC to paste, and it is above: 71 tests, none failing, including the
three new ones.

**One thing `cargo clippy --all-targets -p alo-installer -- -D warnings` cannot
say today, and it is not this change's.** That crate's *development*
dependencies reach `alo-saying`, which reaches `alo-remembering`, whose committed
`src/names.rs` (`6234c2e`, another lane) has four items nothing constructs or
calls; `alo-portals` has one more. Under `-D warnings` those fail before
`alo-installer` is reached. I ran clippy on the crate without the flag and
counted the diagnostics naming a file under `crates\alo-installer`: **zero**.
`alo-image`, which I changed most, gates clean with the flag. The dead code is
another lane's in-flight work; I did not touch it, and it is named here rather
than worked around.

**Not run:** the full workspace suite (the supervisor's, per the task), and
nothing that starts a virtual machine or builds an image — this task builds no
disks, so the plan's disk-space rule had nothing to hold.

## Limitations, honestly

- **The workflow has never run.** It cannot be: it needs a tag pushed to
  `aloworld-org/alo-os` and a runner, and no tag exists. What is proven is that
  the file says what it must and nothing it must not, which is what
  `.github/workflows/image.yml` was held to on 2026-09-15 for the same reason.
  The first real run is the owner's, with the first tag.
- **The executable is unsigned until the owner has a certificate**, and the
  notes, the README and the program all stay silent about signatures rather than
  claim one. A person downloading today gets Windows' unsigned-program warning,
  honestly.
- **`podman build --output type=local` in the environment job** is the shape the
  boot environment's recipe already builds with locally; it has not been run on
  `ubuntu-24.04`. If GitHub's runner disagrees, the first tag says so and the
  fix is a step in this file, not a change to anything it publishes.
- **The `ghcr.io` package is still private** (task 1's own remaining item), so an
  installer built by this workflow could not pull without a login until the owner
  makes it public.

## Proposed for the shared documents

*(For the integration owner; I did not edit `CHANGELOG.md`, `ROADMAP.md`,
`QUEUE.md` or `STATE.md`.)*

**CHANGELOG** — *The installer is released from GitHub.* Pushing a tag builds the
Windows installer and the boot environment it stages, writes the checksum of
every asset, and creates a draft Release whose notes are committed in the
repository: the registry, the release, the exact image digest that will be
installed, and the Secure Boot state the installer accepts. Every one of those is
checked against what the installer really does, so a note that has quietly become
untrue fails the build. The README says what a computer needs before somebody
downloads it — and says that Windows stays. What signs the download is a person,
never a machine (ADR 0046).

**ROADMAP / QUEUE** — v0.5 installer plan: task 5 done 2026-09-16. Tasks 8, 9
and 10 remain, 9 and 10 waiting on a machine with 50 GB free. New owner actions:
a code-signing certificate, and signing plus publishing each draft Release.

**STATE** — reference this report and
`docs/decisions/0046-the-installer-is-signed-by-a-certificate-a-person-holds.md`
(proposed, awaiting the owner).
