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

## Three: a settings file that already holds a picture is refused **whole**

Found by taking the two modules out, reading the twelve errors the compiler
listed, and then asking what it cannot see.

`background` is a key in `appearance.toml` whose **value** is a serialized
`Background`. Today that value can be a picture. Once `Picture` is gone, a file
holding one no longer deserializes — and `alo_kept`'s reader does not drop the key
it cannot read. `docs/contracts/person-settings.md` says such a file is **refused
whole**, and `alo_appearance::keeping::at_sign_in` answers with the release's
appearance and the refusal beside it.

So a person who had chosen a wallpaper does not lose their wallpaper. **They lose
their accent, their text scale, their light-and-dark setting and their per-display
exceptions**, all at once, at the next sign-in, because one key's value names a
type that no longer exists.

This is a schema change and `CLAUDE.md` has a rule for it: *expand → migrate →
contract across releases*. It is a real question rather than a formality, and it
has at least three answers:

- **refuse whole**, which is what happens if nothing is done, and is the worst of
  the three for the person;
- **read and drop** — accept the old shape, discard the background, keep every
  other key, and say so in a sentence;
- **read and convert** — accept the old shape and keep the colour where there was
  one, discarding only a picture.

Nothing here chooses between them. The third is only available where the old value
was already a colour, so it is really the second plus a special case.

**Who this actually affects is worth being exact about, because it is easy to
argue it away.** alo OS has not shipped, so there is no field population. But the
development machines have written these files, the certified laptop will have one
before it is certified, and the read path is the same code either way. *Nobody has
one yet* is a reason to choose calmly, not a reason the question does not exist.

## What stopped the removal, and where it stands

The modules come out cleanly: `git rm` of `picture.rs`, `rotating.rs`,
`lock_image_decode.rs`, `lock_image_fit.rs`, `lock_texture.rs`, the contract and
the artwork, and four lines out of `alo-appearance/src/lib.rs`, leaves **twelve
errors across six files** — `appearance.rs`, `background.rs`, `changes.rs`,
`keeping.rs`, `lock.rs` and `shipped.rs`. Every one is an unresolved import, which
is the shape that makes this kind of removal safe: Rust leaves no silent survivor.

What it does not leave is an answer to the two questions above — what
`Background` becomes when a picture is no longer one of the things it can be, and
what a file holding one does. The first is a small design decision, the second is a
schema decision with a rule attached. Both were reached at the point where the
next edit would have chosen one by accident, so the branch was abandoned rather
than pushed.
