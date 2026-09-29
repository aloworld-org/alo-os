# Taking the wallpaper out — what it touches, and two things the record does not say

**Date:** 2026-09-29
**Workstream:** v0.5 — *Making it yours*, under
[ADR 0075](../decisions/0075-alo-os-has-no-wallpaper-the-canvass-own-surface-is-the-desktop.md)
**Responsible contributor:** the development PC, lane B
**Status:** a survey, not a change. **Nothing is removed by this report.** The
scope half landed as #249; this is the map for the code half.

## Why this exists as a report

ADR 0075 is accepted and names what comes out. Reading the repository against
that list turned up two things that change the size and shape of the job, and
both are the kind of thing somebody would otherwise discover halfway through a
removal that cannot be landed half-done.

## One: there are two types called `Picture`, and only one of them goes

A plain search for `Picture` across `crates/` returns **45 files**. That number is
wrong for this job by more than half.

| | |
|---|---|
| `crates/alo-appearance/src/picture.rs` | a wallpaper. **This is the one ADR 0075 removes.** |
| `crates/alo-capturing/src/picture.rs` | a screenshot. Nothing to do with it. |

`alo-capturing`, `alo-converting` and `alo-dividing` account for most of the
matches and **none of them names `alo-appearance`'s type** — the single
`alo_appearance` reference in that whole set is
`for_the_agent.rs:232`, which uses `Token::DeepTeal` and is untouched.

Anybody sizing this from a grep will over-estimate it, and anybody removing from a
grep will break screenshots.

## Two: `Rotating` comes out, and the record's list does not name it

`crates/alo-appearance/src/rotating.rs` is, in its own first line, *a folder of
pictures, one at a time*. It is the mechanism of the **folder** half of the
promise.

ADR 0075's *What comes out* names `Picture`, `Of::Shipped`, `Of::File` and
`Fitting` — and, separately, *the v0.5 promise's file, folder half*. So the folder
is named as coming out while the type that implements it is not, and a reader
working only from the bullet list would leave `Rotating` behind.

**It cannot stay.** A folder of pictures with no picture type is a position into a
list of nothing. This is a completeness gap in the record's list rather than a
decision the record left open, and it needs no new decision — but it does need
saying, because the difference between those two is the difference between
finishing a removal and exceeding one.

## What it touches, measured

**`alo-appearance` — the root.** `picture.rs` and `rotating.rs` go; `appearance.rs`,
`background.rs`, `changes.rs`, `keeping.rs`, `lib.rs`, `lock.rs`, `shipped.rs` and
`words.rs` name one or both, as do `appearance_kept_in_its_own_file.rs` and
`the_contract_describes_this_file.rs`.

**`alo-shell` — the lock screen and the surface behind windows.**
`lock_image_decode.rs` (66 lines), `lock_image_fit.rs` (34), `lock_texture.rs`
(47) are the picture path the record names. `lock_background_path.rs`,
`lock_background_tests.rs`, `screen_background_tests.rs`, `examples/lock_screen.rs`
and `examples/support/the_walk_check.rs` name the types.

**`alo-access/src/high_contrast.rs`** names `Rotating`, which is the one place
outside those two crates and would be missed by anybody working crate by crate.

**Outside the crates.** `image/Containerfile` installs
`docs/artwork/wallpapers/alo-quiet-horizon.png` to `/usr/share/alo/wallpapers/alo.png`;
`docs/contracts/shipped-wallpapers.md` governs it; and `appearance.toml`'s section
of `docs/contracts/person-settings.md` describes keys that will no longer exist.

## Why it is one change and not several

There is no separable first step. Removing the type breaks every user at once;
removing the artwork breaks the image build; removing the contract leaves a
contract test describing a file nobody writes. The three test files that hold
`appearance.toml` and the shipped-wallpapers contract to the code will fail on the
first of those edits and stay failing until the last.

That is the argument for doing it as one reviewed change rather than a series, and
the argument for mapping it first.

## What the removal must not quietly do

**A fresh machine must still look composed.** The shipped appearance names a
wallpaper today. After the removal it names something that does not exist, and
nothing in this repository says what surface a machine with no choices shows
instead. `docs/autonomy/v0-5-evidence.md` records that under *A fresh machine
already looks composed* — its only evidence is for the feature being removed,
which is a worse position than having none, because those tests are green.

**The lock screen's rule changes rather than disappearing.** ADR 0075 replaces
*never silently show an unchosen colour in place of a chosen picture* with **the
lock screen shows the surface and no client's pixels**. The second half is already
held — `the_walk_check.rs` says the lock texture imports no client, and
`lock_image_decode.rs` composites against opaque black. Whatever removes that file
must keep that property and leave a test saying so.
