# What the greeter does, before there is anything to draw

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, tasks 27 and 28 — one seam, written down twice
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this closes

Task 26 built the door: `crates/alo-sessiond` takes a number `alo-accounts`
has already authenticated and asks `systemd-logind` to open that person's
session. **Nothing knocked at it.** `alo-accounts` decides who is here,
`alo_sessiond::Knock` and `Answered` are the line, `/run/alo-sessiond/sign-in.sock`
is open — and a surface had to authenticate, then knock, then render what came
back, by hand, in that order. A surface that authenticated and forgot to knock
is a screen that takes a correct password and does nothing at all, and nothing
in this repository would have said so.

`crates/alo-greeting` is that composition as one value: **everything the greeter
does that is not drawing.** The screen is task 13's and the desktop lane's, and
nothing here opens a window, measures a font or decides a layout.

## What changed

### `crates/alo-greeting` — the greeter's logic

| File | What it is |
|---|---|
| `src/greeting.rs` | The composition: authenticate, agree about the number, knock once |
| `src/standing.rs` | What the screen asks for before anybody types |
| `src/greeted.rs` | What comes back: a session, or a sentence in the reader's language |
| `src/knocking.rs`, `src/door.rs` | The opener's door, and the real socket behind it |
| `src/refusing.rs` | Every way the machine stops a sign-in, with both its readers |
| `src/words.rs` | The four sentences this crate declares, and the English beside each |

### Held to it

`crates/alo-saying` collects a twenty-seventh list (`EVERY_LIST`, the
`declare_into` call, the key each crate is proved by, and the count that only
adds up while nothing was dropped). `crates/alo-collected` needed nothing: it
reads the workspace's own member list, which is what that check is for.

### The plan

`docs/autonomy/v0-01-delivery-plan.md` had **two sections numbered 27** — task
26's report wrote *The sign-in surface's half of the door*, and a later change
wrote *What the greeter does* for the same work under a different name. They
are answered by one crate here, both are marked done, and the numbering is
repaired: 27, 28 (the greeter), 29 (recovering a parked task). See *The two
tasks that were one* below.

## The decisions this task had to make, and why

The plan named the seam and its terms. It did not name a shape, and six things
had to be chosen.

### One crate answers both sections, rather than one of them being skipped

The two sections describe the same value from two ends: *a `Session` becomes
exactly one knock, and every way the conversation fails is told in words*, and
*read a name and a password, ask `alo-accounts`, and on yes knock*. The second
contains the first. Building one and leaving the other for the next worker
would have sent somebody at work already finished — the failure the plan warns
about in its own rules — so `crates/alo-greeting` keeps every line of both
acceptance lists, and both are marked done with this change.

### The order is held by shape, not by a comment

`Greeting::signs_in` is the only door into the crate. It is the only place
`alo-accounts` is asked anything and the only place a `Knocking` is reached, and
the one call to `alo_sessiond::Knock::on_behalf_of` in the whole crate is inside
a private `for_whom(session: &Session)`. So a knock cannot be made from a number
somebody typed, from text, or from a name that did not verify: the only value
that can become one is an `alo_accounts::Session`, which cannot exist unless a
password verified **and** the machine description agreed about the uid.
`the_only_knock_this_crate_makes_is_made_from_a_session` reads this crate's own
source for that, the way `alo-saying`'s rented check reads for a rented name — a
second call added tomorrow is a failing build rather than a door onto a uid
nobody authenticated.

### The composition is where the timing promise would have been lost

`alo-accounts` answers a wrong password and an unknown name with one value after
the same work, and measures it. A composition above it is the obvious place for
that to be undone: knocking for a known name and not for an unknown one
enumerates this machine's accounts over a wire in milliseconds, whatever the
sentence says. So both refusals return from the first `match` in `signs_in`,
before anything touches a socket, carrying `alo-accounts`' own sentence — and
the measurement is taken again here, of the composition, interleaved and by
medians. `a_wrong_password_never_reaches_a_real_door` is the same fact from the
other end: a real listener is bound, a wrong password is typed, and nothing ever
connects to it.

### Four ways the conversation fails, and two sentences

Nothing listening, a knock that could not be delivered, an answer that never
came, and a line that is not an answer are four values in `NotAnswered`, each
carrying the door and what the machine said. They are **two** sentences at the
screen, and the split is by what the reader can do: *the part of this machine
that starts a session is not running* against *…was reached and did not answer*.
Four sentences would be four wordings of a thing the person cannot act on; one
would throw away the only distinction that sends whoever maintains the machine
to a different place. Both say *Nothing you typed was wrong*, which is
`alo-sessiond`'s own argument for the same clause — without it a person retypes,
concludes they have forgotten their password, and changes it.

`NotAnswered` is also carried in `Greeted` rather than flattened into a key, so
nothing is lost: the person gets the sentence and whoever maintains the machine
gets the English. **This library prints nothing itself.** `alo-sessiond` writes
to standard error because it *is* the service; a library that did would be
choosing somebody's log for them.

### A machine with no store, and a machine whose store will not be believed

`Standing` is derived from the accounts and has two cases — *make an account*
and *sign in* — with no constructor that can name one, which is
`alo_overlay::Standing`'s shape one floor up. A store that is **there** and is
refused (a link, somebody else's file, a file others may write, text that is not
a store) is deliberately **neither**: it is `NotReadable`, a refusal, because a
password box on that machine asks for a keystroke nothing can check, and reading
it as first boot would offer to make a second account on a machine that already
has one. `Greeting::at` is where the two are told apart, and
`alo_accounts::NotKept::NotThere` is the only variant that is not an error.

### What is printed, and what a field may be called

Nothing here holds, logs or returns the password, and that is read out of the
crate rather than asserted: comments and string literals are taken out — with
the *inside a string* state carried between lines, because every sentence in
`src/words.rs` spans several — and in what is left the identifier appears
exactly twice, as the parameter of the one method that takes one and on the line
that hands it to `alo-accounts`. Beside it, two checks the identifier alone
would not catch: nothing on those lines copies, owns, formats or prints
anything, and **every field this crate declares is held to a list of five**,
because a field called `secret` would pass an identifier check and be the same
bug.

`Greeting`'s `Debug` is hand-written for the same reason. `alo_accounts::Accounts`
derives `Debug`, so a derived one here would put every stored hash into whatever
prints a greeter — a panic message on a sign-in screen, a trace, a bug report
from somebody's machine. A hash is not a password, but it is what an offline
guess runs against.

## The two tasks that were one, and what it cost the loop

The plan had two `### 27` headings from 2026-09-11 until this change, and that
is not a tidiness problem. `tools/kernel-loop/src/plan.rs` has a test —
`every_plan_this_repository_drives_holds_only_tasks` — asserting that every plan
this repository drives numbers its tasks from one **in order**, because
`**Depends on:** 1` has to mean one thing. Read against the published plan, the
numbers were `…25, 26, 27, 27, 28`: the first disagreement was at the fourth-last
heading, so that test was **already failing on `main`** before this change, for a
reason nobody had noticed. Renumbering repairs it, and the test is one of this
task's evidence lines for exactly that reason.

The deeper cost is the one the plan's own rules name: a task finished and not
marked is a task the loop selects again. Two sections describing one seam is the
same failure wearing a different coat — whichever of them had been built, the
other would have sent the next worker at work that was already done.

## Verification

Run from `C:\dev\alo-os-claude`, and in WSL2 Ubuntu (`CARGO_TARGET_DIR=/root/target-claude`).

| Command | Platform | Result |
|---|---|---|
| `cargo fmt --all` | Windows | clean; `--check` clean on Linux |
| `cargo clippy -p alo-greeting -p alo-saying --all-targets -- -D warnings` | Windows / Linux | clean on both |
| `cargo doc -p alo-greeting --no-deps` | Windows | no warnings |
| `cargo test -p alo-greeting` | Windows / Linux | 40 / 51 passed |
| `cargo test -p alo-saying` | Windows / Linux | 68 passed |
| `cargo test -p alo-collected` | Windows / Linux | 19 passed |
| `cargo test plan` (in `tools/kernel-loop`) | Windows | 11 passed |

The eleven-test difference between the platforms is
`tests/the_greeter_knocks_at_a_real_door.rs` (eight) and `src/door.rs`'s own
three, all of which are Unix: there is no Unix socket on Windows, and
everything this crate *decides* is arithmetic and runs on both. That is the division `alo-sessiond` itself makes, and it is why
the refusals are in the gate on either machine.

**The whole workspace suite was deliberately not run here.** It takes the better
part of an hour on this machine and the supervisor runs it after this worker
regardless; two finished tasks have died at a deadline waiting on it.

## Limitations, and what is deliberately not here

- **No pixels, and none are claimed.** There is no window, no font and no
  layout, and no guess about what the screen looks like. Task 13 is the drawing
  and it is the desktop lane's. *On the machine* does not move.
- **Nothing on the image changed.** This crate is a library with no binary: the
  process that will hold it is the greeter surface, which task 13 writes, and
  adding a unit for something that does not exist would be the stub law 3
  forbids.
- **No account is created here.** *Make an account* is a sentence and a state;
  the flow behind it is a surface's, and `alo_accounts::Accounts::created` is
  what it will call. This crate says what the screen asks for, not how somebody
  fills it in.
- **A session is not held open.** `alo-sessiond` holds the descriptor `logind`
  gave it; what this crate hands back is the `alo_accounts::Session` value that
  says who was signed in.

## Proposed updates to the shared documents

For the integration owner (this contributor does not edit these):

- **CHANGELOG.md** — *The greeter's half of signing in.* A name and a password
  checked against this machine's own accounts, and — only then — one knock at
  the service that opens a session. A wrong password and an unknown name are
  refused in the same words and in the same time, a machine with nobody on it
  asks for an account to be made rather than for a password, and every way the
  machine can fail to finish says so rather than leaving somebody at a screen
  that appears to have done nothing.
- **ROADMAP.md** — no gate moves. The v0.01 exit gate still opens with a screen
  nobody has drawn.
- **docs/autonomy/QUEUE.md** — nothing to add; this is the delivery plan's
  workstream.
- **docs/autonomy/STATE.md** — this report, and the note that the delivery
  plan's duplicate task numbering had been failing
  `tools/kernel-loop`'s own plan test on `main` since 2026-09-11 and is
  repaired here.
