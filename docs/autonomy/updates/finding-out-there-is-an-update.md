# Finding out there is an update

**Date:** 2026-09-19
**Workstream:** `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, task 6
**Contributor:** Claude (checkout `C:\dev\alo-os-3`)
**Status:** ready for integration

## What this closes

`docs/features.md` promises **updates that never interrupt** at v0.5. Four
fifths of it was built — what an update is and may never do (task 1), applying
one (task 2), going back (task 3), and every sentence a person meets (task 5) —
and **nothing on this machine had ever looked**. `alo-keeping-up`'s own header
said so: *when to check is a later task that builds on these types*.

This is that task. A machine can now find out there is a newer version of its
own system, and the whole promise the rest of the plan makes rests on it: an
update that is never found is an update that never applies, and a machine that
found one by watching would be doing something unasked on somebody's network.

## What changed

### `crates/alo-looking` — new

| File | What it is |
|---|---|
| `src/place.rs` | `Place`: where this machine asks, read through `alo_image::ThePin` from `image/pinned.toml` |
| `src/release.rs` | `Release`: one released version, ordered by its three numbers; every other name a place holds is not one |
| `src/because.rs` | `Because`: why a check is happening, and the whole list is two |
| `src/asking.rs` | `ThePlace`: the two questions a place is asked, and no third |
| `src/registry.rs` | `TheRegistry`: the real place, over the machine's own proxy — the only file that knows an address is a URL |
| `src/looking.rs` | `look`: the one act, on the indicator for the whole of it |
| `src/found.rs` | `Found`: what one check answered, and the build it was about |
| `src/kept.rs` | `Kept`, `TheAnswer`, `NoLongerTrue`: one file under `/var`, read back without asking again, refused when stale |
| `src/said_once.rs` | `SaidOnce`, `Say`: a machine with no way out says so once |
| `src/refusing.rs` | `NoAnswer`: the closed set of five, each with a sentence |
| `src/words.rs` | Four strings, and the English and the translator's note beside each |
| `src/testing.rs` | The place, builds and vocabulary this crate's own tests are written against |
| `tests/finding_out_there_is_an_update.rs` | One test per acceptance criterion, against the machine's real vocabulary |
| `tests/against_the_real_registry.rs` | The sixth criterion, measured against the real place — `#[ignore]`d |

### Registration, which is where this kind of change actually fails

- `Cargo.toml` — the workspace's member list, and `Cargo.lock` with it.
- `crates/alo-saying/Cargo.toml` and `crates/alo-saying/src/collecting.rs` —
  the crate, its `declare_into`, its name in `EVERY_LIST`, its line in
  `ONE_STRING_EACH` and its count in the arithmetic that proves nothing was
  lost and nothing was shared. A crate whose words nothing collects compiles,
  tests, ships and says nothing to anybody in any language; `alo-collected`
  catches it, and was run.

### The plan

- `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md` — task 6 marked done,
  and **task 7 written**, because the plan named none after it and the
  measurement below found something that needs one.

## A user-readable change description

alo OS can now find out for itself that there is a newer version of its system.
It looks in two moments and no others: when you ask it to, and once when the
machine starts. It never looks while you are working, there is no timer in it,
and the moment it does look the indicator says so — *alo OS is checking for an
update at …* — for the whole of the check and not a second longer. What it
fetches is the answer and never the download; the new version is only ever
pulled when you choose to apply it. If it cannot reach anywhere at all it tells
you **once** and then stops mentioning it, and every other way a check can fail
is one plain sentence that ends by saying that nothing on your machine has
changed. The answer is remembered, so the settings panel can tell you an update
is ready without going back out to the network every time you open it — and if
your machine has changed since, it says so and offers to look again rather than
showing you something that stopped being true.

## The design, and the decisions inside it

The task left several things open. Each was decided the way a senior engineer
would, and each is written down here rather than left to be inferred.

### A new crate, and why not `alo-updating`

The plan allowed either. `alo-updating`'s manifest ends with *Nothing else: no
HTTP client and no container library*, and its crate documentation says *No
clock, no schedule and no check for an update*. Both of those are promises a
reader relies on when auditing what applies an update to a machine, and putting
a network client into that crate would have made both false in the same change.
`alo-keeping-up` was never a candidate: the plan's own constraint keeps it at
four dependencies and no socket, and that test is untouched.

So `alo-looking`, beside them: it depends on `alo-keeping-up` for every decision
about what an update *is*, and re-decides none of them.

### Where this machine asks, and why the pin is compiled in

`Place::on_this_machine` reads `image/pinned.toml` through `alo_image::ThePin`.
This crate holds **no address**: a test reads every one of its thirteen source
files for the repository the pin names and for its host, and finds neither. An
organisation's own mirror is then a change to what the pin is read from rather
than a second constant that also has to change — which is the whole reason the
pin is one file.

The pin is taken with `include_str!` at compile time. It has to be: `alo-image`
says outright that the pin *names the image, so it cannot be inside it*, and a
running alo OS machine therefore has no copy on its disk (the installer's boot
environment has one only because it is a different image). A fact that is fixed
when the build is made belongs in the build.

Two fields are read and the third is deliberately not. `registry` is the
repository asked and the place shown on the indicator. `version` is the
**oldest** release this machine will be offered — the floor rather than the
answer, because a machine cannot carry the name of a release that did not exist
when it was built. `digest` is not read at all: what this machine runs is asked
of the base at the moment it is wanted (`alo_updating::running`), and a digest
compiled into a binary would be a second answer to that question, wrong on every
machine that has updated once.

### Which release is offered: the newest the place holds

This is the decision with the most in it, and the alternatives were real.

*The release the pin names* was the first reading of the acceptance, and it does
not work: a machine running `0.0.2` carries `0.0.2`'s pin, would ask about
`0.0.2`, and would answer *up to date* forever. *A name that moves* — a `latest`
or a `stable` tag — is the classic answer and is exactly what `image/pinned.toml`
exists to forbid; `alo-image`'s own words are that *the words people reach for
when a build has to be called something — `latest`, `dev`, `main` — are exactly
the names that are moved on purpose*. Pushing such a tag is also the release
process's, which is another lane's.

So: the place is asked which names it holds, every name that is not
`MAJOR.MINOR.PATCH` is ignored, and the newest of what is left is offered —
never one older than the release this build was cut from, because a way
backwards is `alo_keeping_up::GoingBack`'s, which a person chooses deliberately
and is told the consequences of.

`Standing::between` is untouched and is still *differ, not newer*: which release
the place offers is this crate's question, and whether the offered build differs
from what is running is that crate's.

### Two questions, one act, and a `HEAD`

A check asks the place's distribution interface which names it holds, and then
asks which build one of them is. The second is a `HEAD`, which is that
interface's own way of asking *which build is this* without being sent even the
description of it: what comes back and is read is one header.

Both happen inside one `Underway`, so a person sees one line rather than two
appearing and vanishing. There is no request anywhere in the crate for the bytes
of a build — a test reads every source file for `/blobs/`, `podman`, `skopeo`,
`docker pull` and three more — and only the one file that keeps the answer so
much as names the filesystem, which a second test holds.

### Anonymous, and a token only where a place asks for one

A place that answers `401` is read for the challenge it sent and asked for a
token with the scope it named itself. **The realm must be `https://`**, or the
check refuses: a token is a thing that is sent onwards, and one fetched in clear
is one somebody on the way has. Nothing of the person's is ever sent — this
machine has no account at the place its updates come from, and a check is not a
sign-in.

### Five refusals, and only one of them is held back

`NoAnswer` is closed at five: nothing reachable at all, reached and silent, it
refused, it offers no version of this system, and the answer could not be read.
They are told apart by **what a person can do about them**, not by what went
wrong underneath — which is why a status code appears in none of them.

Four are this crate's own words; the fifth is `alo-keeping-up`'s existing
`keeping-up.answer-not-understood`, rendered rather than written a second time.

Only *no way out* is said once. The other four are about the far end and are
news: a place that answered yesterday and refuses today is something a person
may want to know. *There is no road out of this machine* is the one that stays
true for hours and is unchanged by asking again — and it is suppressed the way
`alo_telling::Telling` suppresses a repeat, by **consuming** the refusal, so a
suppressed telling leaves nothing anywhere in the program that a surface could
word anyway. Any answer at all forgets it, so the next outage is news again.

### What is kept, and what a kept answer deliberately is not

One file, `/var/lib/alo/an-update-was-found`, beside the two facts
`alo-updating` already keeps across a restart, written whole or not at all and
read strictly — the same two rules, for the same reasons, as
`alo_updating::one_build`.

What is read back is **not** an `alo_keeping_up::Offered`. That type does not
deserialise on purpose — *one read back off a disk would be an offer nobody was
shown being asked for* — and that holds here. `TheAnswer` can say what a person
reads and whether an update was ready; it cannot become the `Ready` that
`Staging` needs, and nothing in this crate will give it one.

Which means a person choosing *apply it* is a check away from applying it. That
is correct rather than a gap: the update is staged against the machine as it is
at that moment, and what makes that safe is that the offer was heard while it was
fetched. Carrying a decided update across a process boundary is
[ADR 0053](../../decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)'s
open question (§2, and the constructor owed in §4), and was not pre-empted here.

A kept answer names the build it was about, and is refused when that is no
longer what is running — with `keeping-up.changed-since-it-was-found`, which
already says the machine changed and to check again. *An update is ready* about
a system nobody is running is the kind of stale sentence a person stops trusting
a machine for.

### Why the measurement is `#[ignore]`d

No other test in this workspace's default suite reaches the public internet, and
a network test inside the nine gates is a gate that fails for a reason unrelated
to the change being gated, on a machine whose connection dropped for a minute —
with the cost landing on whoever is trying to merge something else. So the
measurement is `#[ignore]`d, named in the handoff's evidence (which the
supervisor runs with `--include-ignored`), and its numbers are below.

## The measurement, against the real registry

Run on this machine on 2026-09-19, from the Linux side of it (Ubuntu 24.04 in
WSL2), in **2.64 s** for all three tests:

```
cargo test -p alo-looking --test against_the_real_registry -- --ignored --nocapture
```

```
going through http://127.0.0.1:9
through a proxy that is not there: there is no way out of this machine to the place its updates come from
asking ghcr.io about releases, going straight out
a repository nobody publishes: the place this machine's updates come from refused this machine
it holds 6 names: ["0.0.1", "sha256-d3f05b60975edcff51a44c1f21e764a32b286677e306ba24631bad6a00b6a13c",
                   "0.0.2", "sha256-8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9",
                   "0.0.3", "sha256-41d43c7ea491990eea602493e5c645bd7bf8e7d0d9d7a8e9a000bc895a9dd0d7"]
the newest release it offers is 0.0.3
a release it does not hold: the place this machine's updates come from refused this machine
a place that does not exist: there is no way out of this machine to the place its updates come from
a person reads: The place this machine's updates come from would not answer this machine, so nothing on this machine has changed
a person reads: This machine cannot reach the place its updates come from, so it cannot tell whether there is a newer version. Nothing on this machine has changed
it says 0.0.3 is sha256:41d43c7ea491990eea602493e5c645bd7bf8e7d0d9d7a8e9a000bc895a9dd0d7
this machine stands: An update is ready. It will apply when you choose, and nothing you are doing will be interrupted until then
look() answered the same: ready=true about=sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9
and the pinned release 0.0.2 is sha256:8f9c36e0d608eb13d8ba7746b9c549438a939bcbd51e90e2b5fcd5103be90bf9
```

What that shows, line by line:

- **the place the pin names answers this machine**, with six names, three of
  which are releases and three of which are a signature's name and are correctly
  ignored;
- **the indicator carried the check between the two requests** — the line was
  read before the first packet, again between the two, and the indicator was
  quiet and empty after;
- **the pinned release is the digest the pin holds.** `0.0.2` at the registry is
  `sha256:8f9c36e0…`, byte for byte `image/pinned.toml`'s digest, so *what this
  repository published* and *what a machine is offered* are one fact;
- **`look` reaches the same answer** the two questions reach on their own;
- **the road out is the machine's decision.** It was decided by
  `alo_proxy::the_way` for `Road::CheckingForAnUpdate`; with a proxy set to one
  that is not there, a place that answers straight out becomes one that cannot
  be reached, which is the proxy having been taken rather than the place being
  down;
- **three refusals came from the real place** rather than from a stand-in: a
  repository nobody publishes and a release it does not hold are both *it
  refused*; a host that does not exist is *no way out*.

## What was found, and what it is owed to

**A machine running the pinned release is offered `0.0.3`, which this repository
has not pinned.** The registry holds it because the owner pushed and signed it
(its `sha256-41d43c7e…` signature name is in the list above); `image/pinned.toml`
says `next = "0.0.3"` and still pins `0.0.2`, which is exactly the window ADR
0036's release procedure creates between pushing and pinning.

The check is right to offer what the place offers — a machine cannot know about
a release that did not exist when it was built. But *an update is ready* is a
sentence a person acts on, and the act stages under
`--enforce-container-sigpolicy`. A build that policy will not have is refused
there, safely, with the machine unchanged — and the person has been told two
true things and learned nothing. **That is task 7**, written into the plan in
this change, with the measurement above as its starting point. Two options were
left open for it deliberately rather than chosen here: narrowing the offer to
what the place can vouch for, and carrying *not vouched for* into the sentence a
person reads before they choose.

Nothing was done about it in this change, because the alternatives reach into
the release process, which is the installer lane's (ADR 0036), and because the
honest half — the sentence — belongs with the refusal it replaces
(`keeping-up.not-prepared`, which today says both *the download failed* and
*what arrived was not a genuine alo OS*).

## The gate that refused this, and what it showed

The first attempt at this task was refused by the workspace's suite, on one
test:

```
---- looking::tests::the_build_this_machine_runs_is_not_an_update stdout ----
panicked at crates/alo-looking/src/looking.rs:161:
called `Result::unwrap()` on an `Err` value: NothingIsOffered
```

**Nothing was wrong with `look`.** Between the attempt that passed its gates
and the attempt that was combined with `main`, release `0.0.3` was pinned
(`94a88df`, *The image ships the decoders nobody can charge us for, and 0.0.3 is
pinned*). `Place::on_this_machine` reads its floor from that pin, and the unit
tests here took their place from it while writing their own release names down
— so a test whose place held `0.0.2` stopped meaning *a release this machine
would take* and started meaning *a release older than this machine's floor*,
which is exactly the answer it got. The test was right, the code was right, and
the two disagreed because of a number in a third file that neither of them is
about.

That is a real defect in the tests rather than an accident, and bumping the
literal to `0.0.3` would have rebuilt it to go off at `0.0.4`. So the fix is
that **a test which means *a release this machine would take* says which one**:

- `crate::testing::a_place_not_before` makes a place from the shipped pin with
  its `version` line replaced, through the pin's own reader and the real
  `Place::the_pin_names`. `crate::testing::the_pin_where` is the one line
  rewriter in the crate — `place.rs`'s own test helper, which already rewrote
  the `registry` line, now goes through it rather than keeping a second copy.
- Every unit test in `looking.rs` that names releases takes its floor from
  `NOT_BEFORE`, a constant in that test module, and the comment on it says why.
- The two tests that *are* about the shipped pin keep reading it, and they read
  the release **out of the place** instead of writing a number down:
  `the_line_a_person_reads_is_the_check_at_the_place_the_pin_names`, and a new
  `a_machine_at_the_pinned_release_is_offered_what_that_release_is_now` — which
  is the failing test's meaning, stated so that it cannot go off again. It also
  records a real property: the floor is the pinned release *itself* and not one
  above it, so a machine built from a release that was afterwards re-pushed is
  offered the build now behind that name rather than told there is nothing for
  it.
- `place.rs` gains `the_floor_is_the_release_the_pin_states`, so the mechanism
  those tests now rely on is itself tested rather than assumed.

`THE_PIN_AS_BUILT` became `pub(crate)` for this, and that is the whole of the
change to anything that ships: no public surface moved, and `look` and every
type around it are byte for byte what the first attempt handed over.

**What generalises:** a test that reads a file the release process writes is a
test with a second author. `image/pinned.toml` moves whenever a release is
pushed, in another lane, and any test that both reads it and hard-codes
something to compare against it will fail on a change that has nothing to do
with it. The assertions that belong against the shipped pin are the ones about
the pin — and they read their values out of it.

## Limitations, stated

- **One page of names.** A place holding more than 1000 names would need its
  answer read across several pages; nothing needs that yet, and it would be its
  own change. The constant says so.
- **No measurement behind a real proxy.** What is measured is that the road is
  decided by `alo_proxy::the_way` and handed to the client explicitly, and that
  a machine with a proxy set carries it. A company network is owed a measurement
  of its own, as every report in this workstream has said about hardware.
- **Nothing calls `look` yet on a machine.** The two occasions are a call
  something else makes: the shell's *check now* is the shell plan's, and the
  unit that looks once at a start is the image lane's — the same shape as
  `alo_updating::after_a_restart`, which task 2 also left owed.
- **The answer a surface reads back cannot be applied from.** By design; see
  above and ADR 0053.

## Verification

Run from `/root/alo-trees/this-machine` on the Linux side of this machine
(Ubuntu 24.04 in WSL2), with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
which is where this machine's gates build — `docs/autonomy/SHARED_MAIN.md`, *one
build cache and one gate run per machine*.

| Check | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings`, the whole workspace | clean, zero warnings |
| `cargo test -p alo-looking` | 54 unit + 5 acceptance passed, 0 failed, 3 ignored |
| `cargo test -p alo-saying` | 63 unit + 4 acceptance + 1 doctest passed |
| `cargo doc --no-deps -p alo-looking -p alo-saying`, `RUSTDOCFLAGS=-D warnings` | clean |
| each of the eight evidence tests, run alone with `--exact --include-ignored` | 1 passed each |
| `against_the_real_registry`, the three measurements | passed in 2.69 s, 0.88 s and 0.51 s, against the real registry, re-measured 2026-09-19 |

The second attempt's numbers, on the same machine and the same build directory.
One note for whoever gates from a Windows checkout next: `cargo fmt --all` has
to be run through the Linux side as well, because from Windows it stops at *The
filename or extension is too long (os error 206)* — it hands every file in the
workspace to `rustfmt` on one command line, and this workspace is past what
Windows will carry. `cargo fmt --all --check` from `/mnt/c/...` is clean and
writes to the checkout, which is what was run here. This is about the loop's own
machines rather than about alo OS, so it is here and not in `docs/quirks.md`.

**Not run here:** the whole workspace's suite, the supervisor's three gates and
the BPF target's two. The supervisor runs the nine on the combined tree; this
change adds a workspace member and a `Cargo.lock` entry, so by
`docs/autonomy/SHARED_MAIN.md`'s own table it reaches all nine.

**Not measured, and not claimed:** anything on real hardware, and anything about
a certified machine. This is a check over a network from a developer's machine.

## Acceptance, criterion by criterion

| The plan's acceptance | The test |
|---|---|
| where this machine checks is read rather than guessed, from the registry and release `image/pinned.toml` pins, through `alo-image` | `where_this_machine_checks_is_read_from_the_pin_and_written_nowhere_else` |
| a check is one act on the indicator for the whole of it, an `Offered` can be heard no other way, and it fetches the answer and never the build | `a_check_is_one_act_that_fetches_an_answer_and_never_a_build` |
| when a check happens is somebody's act and never a watcher's — two occasions, no thread, no timer | `when_a_check_happens_is_somebodys_act_and_never_a_watchers` |
| a closed set of refusals with a sentence each, and a machine with no way out says so once | `every_refusal_is_a_sentence_and_no_way_out_is_said_once` |
| the answer is kept where a surface reads it back without asking again, and a kept answer names the build it was about | `the_answer_is_kept_and_a_machine_that_has_moved_on_is_not_shown_it` |
| the whole of it measured against the real registry, with the proxy honoured, a refusal exercised, and the indicator read | `a_check_against_the_place_this_repository_pins`, `the_refusals_the_real_place_produces`, `a_machines_proxy_is_on_the_road_a_check_takes` |

## Proposed shared-document updates

`docs/autonomy/SHARED_MAIN.md` gives `CHANGELOG.md`, `ROADMAP.md`,
`docs/autonomy/QUEUE.md` and `docs/autonomy/STATE.md` to the integration owner.
Proposed, not made:

- **`CHANGELOG.md`**, under Unreleased: *alo OS can find out there is a newer
  version of its system. It looks when you ask it to and once when the machine
  starts, never while you are working; the indicator shows the check for the
  whole of it; it fetches the answer and never the download; a machine that
  cannot reach anywhere says so once; and the answer is remembered so a panel
  can show it without going back out to the network.*
- **`ROADMAP.md`**, v0.5 *updates that never interrupt*: the last piece this
  plan can build has landed; what remains of the line on a machine is the two
  callers (the shell's *check now*, and a unit that looks once at a start) and
  task 7.
- **`docs/autonomy/QUEUE.md`**: task 6 of the machine-keeps-itself plan done;
  task 7, *An offer a person can act on*, written and ready.
- **`docs/autonomy/STATE.md`**: reference this report.

## Status

**Ready for integration.** Nothing in this change is partial: every acceptance
criterion has a test in the change that claims it, and the sixth is a
measurement against the real place with its output printed above. The one test
the gates refused now passes, stated so that pinning the next release cannot
break it again, and each of the eight pieces of evidence was run on its own
before this was handed over.
