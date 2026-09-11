# The image becomes a disk a machine can actually boot

**Date:** 2026-09-11. **Workstream:** the image (v0.01 delivery plan, task 25).
**Contributor:** Claude, in `C:\dev\alo-os-claude`. **Status:** ready for
integration.

## What changed, in words

Until this change `image/Containerfile` built a bootable container and nothing
in this repository turned that container into a disk. So the only thing anybody
had ever seen was a test saying the recipe was right, and every promise in
`docs/autonomy/v0-01-evidence.md` that reads *still owed: no machine has ever*
was waiting on a step nobody had written down.

There is now one documented command that writes the image onto a disk file, a
document that says what it produces, what has to be installed, how to attach the
result to a Hyper-V generation 2 virtual machine, and — in the same document, by
name — what a virtual machine can never show. `crates/alo-image` holds the
document and the recipe to each other, so the two cannot drift.

- `docs/booting.md` — the document. Four facts in an indented block (the tool,
  the firmware, the virtual machine generation, the disk's filename), the two
  commands, the Hyper-V steps, and a section called *What a virtual machine
  cannot show*.
- `image/Containerfile` — three `alo.disk.*` labels declaring which tool writes
  the disk, which firmware it is installed for, and what the file is called.
- `crates/alo-image/src/disk.rs` — what the recipe says about the disk.
- `crates/alo-image/src/booting.rs` — what the document tells a person to do,
  read as something checkable.
- `crates/alo-image/src/checking.rs` — two new checks, nine new twins.
- `crates/alo-image/src/wrong.rs` — eight new disagreements, each naming the
  decision or the promise it is about.

## Decisions I made, and why

**The tool is `bootc install to-disk`, out of the pinned base itself.** The
acceptance allowed either that or `bootc-image-builder`. `bootc-image-builder`
is a second container image, which would need a second pin and a digest — and a
digest is a measurement, not something a worker with no network may invent.
`bootc install to-disk` is already on the image, because the base is a bootc
base: the version of the partitioner *is* `THE_BASE`'s digest, which this
repository already pins and already reviews. That also makes the no-second-recipe
constraint structural rather than a rule somebody remembers: the tool installs
the container it is running as, so what lands on the disk is what
`image/Containerfile` produced or the command fails.

**The disk is declared as labels in the recipe, not as a file beside it.** A
second file is a second recipe and a divergence waiting to happen. Labels travel
with the image, so somebody holding only the image can ask it what disk it
expects, and the declaration cannot be separated from the thing it declares.

**`crates/alo-image` now reads `docs/booting.md`.** The firmware acceptance
criterion — *the disk's firmware mode matches what the document tells a person to
select* — is a statement about two files, and only something that reads both can
hold it. `Image::at` therefore reads `../docs/booting.md` from the image's own
directory, and the test fixture copies the repository's shape (an `image/` and a
`docs/`) rather than only the image.

**The document's facts are machine-readable; the rest stays prose.** Four
`key: value` lines in an indented block. A fixed key set rather than every line
with a colon in it, because ordinary prose is full of colons and a reader that
took them all would read a sentence as a promise. A fact stated twice reads as
stated by nobody — a person reads the first and a checker would read the last,
which is exactly the drift this crate exists to catch.

**Podman, not Docker, for the disk step.** Upstream's install path reads the
image out of `/var/lib/containers`, which is podman's store; Docker can still
build the image. And Hyper-V attaches VHDX files rather than raw ones, so the
document ends with one `qemu-img convert` — an upstream tool, configured, doing
a format conversion, which is not a partitioner and is not ours.

**What is checked about a partitioner is that there is not one.** Neither the
recipe nor the document may name `sfdisk`, `fdisk`, `parted`, `mkfs` or
`dd if=`. The likelier half of that mistake is the document: not a second recipe,
one helpful extra step added to the page somebody follows.

## What this does not claim

Nobody has run it. This is a documented command and a set of checks, not a disk
anybody has watched come up, and the report says so in the same breath as the
plan's `**Done**` line does. `crates/alo-image`'s own module documentation was
extended to say it again: a recipe that declares how it becomes a disk is not a
disk.

And a virtual machine is not phase 8. `docs/booting.md` says so under its own
heading, by name — the GPU, the firmware, and anything physical — and that
paragraph is now held by a test, because the way such a paragraph goes is one
bullet at a time.

## Verification

Windows 11 Pro, `C:\dev\alo-os-claude`, run before this report was written:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-image --all-targets -- -D warnings` — clean, zero
  warnings.
- `cargo test -p alo-image` — 110 unit tests, 16 integration tests, 0 failures.

Per acceptance criterion:

| Criterion | Test |
|---|---|
| One documented command, by a pinned upstream tool | `the_image_says_what_disk_it_becomes`, `the_document_tells_a_person_the_disk_this_image_really_makes` |
| Never a hand-rolled partitioner | `a_disk_written_by_a_partitioner_of_ours_is_caught`, `a_partitioner_in_the_document_is_caught`, `a_base_on_a_tag_that_moves_is_caught_as_an_unpinned_tool` |
| A check per promise, with a twin | `an_image_that_declares_no_disk_is_caught`, `a_document_naming_a_firmware_the_image_is_not_installed_for_is_caught`, `a_document_that_gives_no_command_is_caught`, `a_document_missing_a_section_is_caught` |
| The firmware matches what the document says to select | `a_document_telling_somebody_the_wrong_generation_is_caught` |
| What the disk cannot show, in the same document | `the_document_says_what_a_virtual_machine_cannot_show`, `a_document_that_stopped_naming_what_a_disk_cannot_show_is_caught` |

The full workspace suite was deliberately not run here; the supervisor runs it.

## Limitations, and what is owed next

- **Nothing has booted.** The owner's Hyper-V run is the next step, and it is
  the owner's. Whatever it finds that contradicts `docs/booting.md` belongs in
  `docs/quirks.md` in the change that finds it — Secure Boot's template in
  particular, which is the one step in that document nobody has yet watched.
- **The document is held to the recipe, not to a build.** A check that
  `podman build` succeeds is a build, not a check, and it is not what this crate
  does.
- `ROADMAP.md`'s hardware line stays empty. Nothing here may tick it.

## Proposed shared-document updates

Not made by me; `SHARED_MAIN.md` gives these to the integration owner.

- **CHANGELOG.md:** *The image can now be turned into a disk. One documented
  command writes alo OS onto a bootable disk image using the `bootc` in its own
  pinned base, `docs/booting.md` says how to attach the result to a Hyper-V
  generation 2 machine, and what a virtual machine cannot show about a certified
  one is written in the same document and held there by a test.*
- **QUEUE.md:** task 25 of the v0.01 delivery plan is done; the physical
  acceptance item is unchanged and still owed.
- **STATE.md:** reference this report.
- **`docs/autonomy/v0-01-evidence.md`:** the *no machine has ever* entries now
  have a path to a machine, and none of them may be ticked from it.
