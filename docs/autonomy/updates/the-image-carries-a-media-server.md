# The image carries a media server, and refuses one too old to open a camera

**Date:** 2026-09-19
**Workstream:** v0.5 — devices and media, task 4's blocker
**Task:** the blocker under task 4, *The camera and the microphone* — its fifth
acceptance wants WirePlumber 0.5, which the image was to pin deliberately.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** written on an **Apple M3 with 8 GB unified memory**, macOS 26.5.2;
gated in the Lima VM on that Mac — **Ubuntu 24.04 aarch64, 6 CPUs, 4 GB of
memory**. **Nothing here is ticked on the machine.** The recipe builds for
`x86_64` and this lane has no machine to run the built image on; what is claimed
below is what the recipe says and what the build will refuse, not what a booted
workstation did.
**Egress:** none. No package was fetched, no registry was contacted, nothing was
built. The package names and the version floor are read from the recipe's own
pinned base and from measurements already in `docs/quirks.md`.

## I took another plan's directory, and why

`image/` belongs to the installer plan, on the third PC. The owner routed this
blocker there on 2026-09-17 and then, on 2026-09-19, handed it back under the new
rule: **nobody is inside `image/` right now, so take it, and say in the report
that you took it and why.** This is that sentence.

What I did **not** take is in the finding at the end: the release number this
change makes due, and the eight fixtures in `crates/alo-image` that would have to
move with it. That is the installer lane's crate, and a change needed in another
lane's crate is a finding rather than an edit.

## The larger fault, found on the way to the pin

**The recipe shipped no media server at all.**

`alo-sound`, `alo-cameras`, `alo-in-use` and `alo-capturing` all reach the
machine through `crates/alo-media-server`, which starts `pw-dump`, and through
`wpctl`. Neither program is in the pinned Fedora bootc base and nothing in the
recipe installed one. A workstation built from this recipe would have answered
**nothing here handles sound and video** to every one of those crates — and been
right.

That is a machine with no sound, no camera list, and an in-use indicator that
could never show anything. It is worth saying plainly that the four-fact refusal
built last week was working correctly the whole time: `NothingHandlesIt` is
exactly what an image with no `pw-dump` deserves. **The crates were right and the
image was empty**, which is the failure mode an honest refusal produces — it
tells you the truth about a machine nobody had finished.

## What the recipe says now

```
RUN dnf install --assumeyes --setopt=install_weak_deps=False \
      pipewire pipewire-utils wireplumber \
 && dnf clean all \
 && test -x /usr/bin/pw-dump \
 && test -x /usr/bin/wpctl \
 && least=0.5 \
 && have="$(rpm --query --queryformat '%{VERSION}' wireplumber)" \
 && test "$(printf '%s\n%s\n' "${least}" "${have}" | sort --version-sort | head -1)" = "${least}" \
 && echo "WirePlumber ${have}, at or above ${least}"
```

Installed by the base's own package manager, for the converter's reason: its
libraries come from the base rather than from us. `--no-install-recommends`'s
equivalent (`install_weak_deps=False`) for the same reason as everywhere else
here — an image carries what it was asked for.

## Why a floor, and why the build checks it

**0.5 is a floor, not a preference.** Measured 2026-09-17 on WirePlumber 0.4.17
and written up in `docs/quirks.md` — *No client can open a camera through the
media server on WirePlumber 0.4*: the camera is in the graph and complete, and
**nothing can attach to it.** `pw-cat --record` and `gst-launch-1.0 pipewiresrc`
both refuse, by name, by serial and with no target, using the server's own tools
against its own node. The video policy that makes a camera openable is 0.5's.

Below that floor, task 4's fifth acceptance — *an application opens the camera
through the portal and appears on the in-use indicator* — can neither pass nor
fail honestly, and the machine ships a camera nothing can use.

**The floor is checked in the build rather than written down.** The base is
pinned by digest and carries 0.5 today; a base that moved back would otherwise
change what this machine can do without changing a line of the recipe. A floor
nothing enforces is a sentence, not a pin.

**It is a floor and not an exact release**, which is the one place this pin is
shaped differently from the other three in the file. The runtime, the office
engine and the weights are artefacts fetched by digest: they say exactly what
arrived. A distribution package is not one, and pinning a single release of it
would pin the base's security updates out along with it. A build that refuses
anything below 0.5 is the better trade, and it is the same trade the base itself
is already making.

## What this does not do

- **It does not take task 4's acceptance.** The acceptance needs a machine
  running a built image. This lane gates on aarch64 against WirePlumber 0.4.17,
  and the recipe builds for `x86_64`; the pin makes the acceptance *possible*,
  and somebody with the machine has to take it.
- **It starts nothing.** PipeWire and WirePlumber ship their own user units and
  are started by the session. Whether `alo-sessiond` starts them on sign-in is
  that plan's question, and it is a real one — see the finding below.
- **It is not a codec decision.** ADR 0051's decode half is still the owner's,
  and nothing here adds a decoder.

## Findings, none of them edited here

1. **This change makes a release number due, and the machinery that wants it
   would break eight tests.** `image/pinned.toml` says the next release is
   declared as `next = "MAJOR.MINOR.PATCH"` in the same change that moves the
   recipe's label to it, and `crates/alo-image`'s
   `tests/how_the_image_is_published.rs` holds the recipe's label to
   `pin.next().unwrap_or(pin.version())`. I wrote that bump, ran it, and **backed
   it out**: eight unit tests in `crates/alo-image/src/publishing.rs` and
   `src/pinned.rs` spell `0.0.2` into their fixtures, and `docs/booting.md`
   spells a tag. That crate already has the right answer for this —
   `testing::the_release_line()`, whose own doc comment says *a test that spells
   it out has to be edited by whoever publishes, a chore at the exact moment care
   is wanted* — and those eight are the ones that do not use it yet. **For the
   installer lane:** move the eight to `the_release_line()`, then declare
   `next = "0.0.3"` and move the label. Until then the recipe names 0.0.2 and no
   longer builds what the pinned 0.0.2 digest contains.
2. **Nothing starts the media server on a booted machine.** The packages are
   aboard; `image/usr/lib/systemd/system/` has no media units and would not be
   where they belong anyway — PipeWire's are user units. Which of them the
   session enables is the session plan's, and until somebody answers it a machine
   with the packages installed is still `NoServerIsRunning`. That refusal is
   already its own fact, deliberately, so the machine will say so accurately.
3. **A test fixture at a fixed path in `/tmp` failed a gate again.**
   `crates/alo-software/tests/the_proxy_on_the_road_out.rs` writes
   `/tmp/alo-software-proxy-road.sh` and execs it; under the full concurrent
   suite it failed with `Text file busy`, and it passes three times running in
   isolation. Same family as the collision fixed on 2026-09-17. The helper is
   copied into `crates/alo-updating/tests/the_proxy_on_the_road_out.rs`; both
   want the made-not-made-if-needed treatment. Reported in pull request #10 and
   repeated here because it will outlive that thread.

## Gate

The recipe cannot be built here — it targets `x86_64` and this lane gates on
aarch64 — so what is gated is what reads it: `alo-image` (261 unit tests, plus
its four test files), `alo-installing` and `alo-updating`, which SHARED_MAIN.md's
table names for a diff that touches `image/**`. Evidence is with the pull request.
