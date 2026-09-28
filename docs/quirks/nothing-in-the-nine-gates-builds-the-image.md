# Nothing in the nine gates builds the image

**A green gate says nothing about whether this repository can produce the thing
it ships.** On 2026-09-28 `main` could not build an image for about an hour, and
twelve consecutive merges passed all nine gates during it.

## What happened

`#229` added a binary to the recipe:

    cargo build --release --target ${THE_TARGET} \
      --package alo-agentd --package alo-boundaryd --package alo-sessiond \
      --package alo-portald \
      --bin alo-shipping \
      --package alo-converting

`cargo build --bin X` searches for `X` only among the packages `--package`
already named, and `alo-software` — the package that holds `alo-shipping` — was
not among them. Every image build from `cf0ad44f` onward died at the first step:

    error: no bin target named `alo-shipping` in `alo-agentd`, ... packages
    help: available bin in `alo-software` package: alo-shipping

Thirteen seconds of `podman build` would have caught it. Nothing ran it.

## Why no gate could see it

The nine gates are `cargo fmt`, `cargo clippy`, the workspace's tests, `cargo
doc`, the citation check, and the supervisor's own three. Every one of them
operates on the workspace. **None of them invokes `podman build`, and none of
them reads `image/Containerfile` as a thing to execute.** The image build is the
thing nobody runs until a release, which is the point at which it is most
expensive to discover a fault.

This is not a defect in the recipe. It is the shape of what the gate covers.

## Why the existing image tests did not catch it either

`crates/alo-image/tests/` reads the recipe as text and holds it to several
promises — that no daemon links a system library, that every unit with an
`[Install]` is enabled, that every `ExecStart` names a path the recipe creates.
That last one was written the same afternoon, for the same family of fault.

It could not catch this, and the distinction is the useful part: **those tests
check what the recipe *claims*, and this was a fault in what the recipe can
*produce*.** A unit whose `ExecStart` names `/usr/libexec/alo-shipping` is
perfectly consistent with a recipe that copies that path — and still fails, if
the build step that makes the file cannot run.

`crates/alo-image/tests/the_recipe_builds_what_it_asks_for.rs` closes that
specific hole: every `--bin` must be held by a package the same command names,
and every `--package` must exist. It uses `cargo metadata` against `Cargo.lock`,
so it compiles nothing and runs in the ordinary gates.

## What is still not covered, and should be assumed rather than hoped

The new test catches a `--bin` with no package. It does not catch:

- a `COPY --from=` naming a path the build does not produce;
- a `dnf install` of a package the base does not have;
- a `RUN` that fails for any other reason;
- a base digest that no longer exists in the registry;
- anything at all about whether the resulting image **boots**.

Every one of those is only found by building, and — for the last — by booting.
**A green gate is evidence about the workspace and is not evidence about the
image.** Treat any claim about the image that was not produced by building one
as an inference, and say so.

## What to do about it

Build the image on some cadence that is not "when a person cuts a release" —
nightly on `main` is enough, and a failure costs 220 seconds to find. Until
something does, the honest reading of a green gate on an `image/` change is *the
workspace still compiles and its tests still pass*, which is a smaller claim than
it looks.

## How this was found

By building the image in order to boot it, because `#223` had enabled the
compositor unit for the first time, `#218` and `#223` had put `alo-portald` in
the image for the first time, and `#229` had added a first-boot unit that
installs seven applications — three things that had never run, all claimed as
landed on the strength of green gates.

The first thing an actual build did was disprove a fourth claim nobody had
thought to doubt: that the image built at all.
