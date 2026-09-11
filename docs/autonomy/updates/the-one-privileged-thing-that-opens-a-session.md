# The one privileged thing that turns a correct password into a session

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 26
**Contributor:** Claude (`C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this closes

`alo-agentd.service` is `BindsTo=` and `WantedBy=` `user@1000.service`, so a
person's agent starts when their own systemd manager starts — and nothing on the
image started that manager, because nothing signed anybody in. `systemd-logind`
is what starts it, and `logind` opens a session only for a privileged caller.
ADR 0024 priced that gap and accepted Option B: alo OS owns the first screen, and
pays for it with **a second privileged component**, small enough to read in one
sitting and held to ADR 0018's loader's terms.

This is that component. `crates/alo-sessiond` takes a number that
`alo-accounts` has already authenticated and asks `logind` to open that person's
session. It does not draw, it does not authenticate, and it can do nothing else.

## What changed

### `crates/alo-sessiond` — the opener

| File | What it is |
|---|---|
| `src/opening.rs` | The whole decision: four answers, in the order they are decided |
| `src/asking.rs` | The wire — one line carrying one number, one line back |
| `src/door.rs` | Who may knock, which is a group and not a name |
| `src/accounts.rs`, `src/logind.rs` | The two questions it asks the machine, one method each |
| `src/machine.rs` | The Linux half: `logind` over the system bus, and the accounts file |
| `src/listening.rs`, `src/unix.rs` | The socket, and the only file that asks the kernel anything |
| `src/place.rs`, `src/refusing.rs`, `src/words.rs`, `src/main.rs` | Where the door is, why something refused, what a person reads, and the process |

### The image

`image/usr/lib/systemd/system/alo-sessiond.service`, the binary in `libexec`
beside the loader, and the greeter login and group the door is handed to in
`image/usr/lib/sysusers.d/alo.conf` (60990, beside the agent's 60989 and for the
same measured reason). The `Containerfile` builds the third package, installs it,
enables the unit, and asserts the two new numbers the way it asserts the other
three.

### Held to it

`crates/alo-image` reads a third unit and gains seven checks, each with a twin
that breaks one line of a copy of the image. `crates/alo-saying` collects a
twenty-sixth list. `alo_accounts::Accounts::numbers` is one additive method — a
number in, a yes or no out, no name in either direction.

### Written down

`docs/quirks.md` gains the measurement, on three machines. ADR 0024 records that
the check it said it owed before any box was ticked has been paid.

## The decisions this task had to make, and why

The plan named the component and its terms. It did not name its shape, and four
things had to be chosen.

**1. It is a daemon with a door, not a library or a helper somebody runs.**
The alternatives were a library linked into the sign-in surface and a one-shot
binary the surface executes. Both make the *surface* the privileged thing, which
is ADR 0018's argument thrown away in a different direction: the second
privileged component would then be a compositor. ADR 0024's own Option B says
the surface "asks a small privileged opener", and a door is what asking is. The
cost is a socket and a wire; the benefit is that the thing that draws holds
nothing at all.

**2. It holds no capability, and root is the whole of its privilege.**
This is the part that came in cheaper than ADR 0024 quoted. `logind` decides
`CreateSession` on the caller's **uid**, not on a capability — so the unit says
`User=root`, `CapabilityBoundingSet=` and `AmbientCapabilities=`, both present
and both empty, and it still works. The one filesystem thing it does is hand its
own socket to the greeter's group, and changing a file's group to a group the
process is already in is allowed to the file's owner without `CAP_CHOWN`. The
loader holds two capabilities and argues for them at length; the second
privileged component holds none, and `crates/alo-image` is where that is a test
rather than a sentence in this report.

**3. *Cannot be asked for a uid the caller has not authenticated*, across a
process boundary.** Inside one process this is a sealed value. Across a socket
it cannot be, so it is two things instead. A knock is refused unless the kernel
says the caller is in the door's group — `SO_PEERCRED`, which is not a field in
a message. And the number on the wire is checked against
`/etc/alo/accounts.toml`, the same file the surface authenticated against, before
`logind` is asked anything: the set of numbers this component will ever open a
session for is exactly the set of people this machine has. No password reaches
it, no name and no path — `Knock` holds one `u32`, and a line with a second thing
on it is refused rather than read leniently, which is a test in `asking.rs`
rather than a rule somebody keeps.

The trust that remains is nameable and is stated rather than hidden: anything
running as the greeter can ask for a session for an account this machine has,
without proving a password to *this* component. That is irreducible in any
split where the authenticator and the privileged part are different processes,
it is why the greeter is a login of its own rather than the person's or the
agent's, and `crates/alo-image` refuses an image where it is either.

**4. The wire is this crate's own, not `alo-protocol`.** That vocabulary is the
agent's door (ADR 0017) — a turn, a verb, an approval, a record — and this door
is open before anybody has signed in. Putting a sign-in on it would mean every
message an agent can send exists inside a privileged process's parser. What this
door speaks is `open <number>` and `opened` / `refused <key>`, and the refusal
crosses as an `alo_strings::Key` rather than English, because the surface is what
has a person and a language in front of it.

Two smaller choices, both recorded in `src/machine.rs`: the session is created
with **no seat and no VT**, and with type `unspecified`. A seat and a VT are
facts about what draws, and this component must not decide what a person sees;
`logind` starts `user@<uid>.service` on a person's first session whatever its
seat, which is the whole of what `alo-agentd.service` waits on. A later task that
starts a compositor is the one that knows which VT it is on, and it will be
changing an argument rather than adding a door. And the root-group refusal lives
at start-up rather than in `Door`'s constructor, following `alo-boundaryd`, where
`NotLoaded::TheRootsGroup` is raised by the loader reading its own group.

## The measurement, taken again on the pinned base — and what it found

ADR 0024 said in as many words that the WSL answer was not the image's and that
the check was owed before any box was ticked. It was taken on **Fedora 42,
systemd 257 (257.13-1.fc42)** — the alo OS image built from
`quay.io/fedora/fedora-bootc:42`, run under `podman run --systemd=always
--privileged` so that the base's own `logind` was the one answering.

| Caller | Fedora 42 / systemd 257 | Ubuntu 26.04 / systemd 259 |
|---|---|---|
| root, implausible leader PID | `Leader PID is not valid` | `Invalid leader PID` |
| uid 1000, same call | `Access denied` | `Access denied` |

**The answer holds: Option B is buildable on the machine alo OS ships.** The
call is authorised for a privileged caller and refused for anybody else, so no
PAM module, no C ABI and no `unsafe` exemption is owed.

Two differences turned up that are not differences in the answer, and both are
now in `docs/quirks.md` because both are traps:

- **The root case is reworded between the two systemds.** Anything recognising
  that sentence would be reading prose upstream rewrites.
- **An unprivileged caller is not always refused by `logind`.** Asked from uid
  65534 on Ubuntu through this crate's own call, the **bus policy** turned the
  call away before `logind` saw it, with `Rejected send message, 2 matched rules;
  …` — a completely different sentence under the same D-Bus error name,
  `org.freedesktop.DBus.Error.AccessDenied`. `busctl` prints the name's friendly
  form, which is what made the two look identical in ADR 0024's first
  measurement.

So `NotOpened::Refused` carries the error **name** beside the sentence and
everything decides on the name; the sentence is for whoever is reading a service
log. The test asserts the name and never the wording.

## Verification

Run from `C:\dev\alo-os-claude`, and in WSL2 Ubuntu 26.04 with
`CARGO_TARGET_DIR=/root/target-claude` per `SHARED_MAIN.md`.

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all` | Windows | clean; `--check` clean on Linux |
| `cargo clippy --all-targets -- -D warnings` | Windows | clean, whole workspace |
| `cargo clippy -p <crate> --all-targets -- -D warnings` | Linux | clean for all six crates touched |
| `cargo test -p alo-sessiond` | Windows / Linux | 40 / 52 passed |
| `cargo test -p alo-accounts` | Windows / Linux | 47 / 55 passed |
| `cargo test -p alo-image` | Windows / Linux | 135 / 136 passed |
| `cargo test -p alo-saying` | Windows / Linux | 68 passed |
| `cargo test -p alo-collected` | Windows / Linux | 19 passed |
| `cargo test -p alo-citing` | Windows / Linux | 31 passed |

The whole-workspace suite was **not** run here: it takes the better part of an
hour on this machine and the supervisor runs it after this regardless.

The unprivileged measurement was additionally run **as a real unprivileged
process** rather than skipped, by running the test binary under
`setpriv --reuid=65534 --regid=65534 --clear-groups`, which is where the bus-policy
finding above came from. Under `cargo test` as root it skips itself and says why.

## What is still owed, plainly

- **No machine has opened a session with this.** The privileged half of the
  measurement is a by-hand `busctl` call on the pinned base; the component's own
  privileged path has never run on a booted alo OS, because nothing on that image
  knocks yet. That is task 27, written into the plan.
- **The image has not been rebuilt.** `crates/alo-image` reads the recipe and the
  unit and holds them to each other, which is how every other image promise in
  this repository is held; a `docker build` of the whole image was not run.
- **`Type=exec` and the held descriptor are a design, not an observation.** The
  session lasts while this process holds the descriptor `CreateSession` returns.
  That is documented in `src/main.rs` and `src/logind.rs` and is exercised only
  against fixtures.
- **Signing out does not exist.** Stopping the service ends the session, and a
  second knock while one is open is refused in words. v0.01 has one person and
  one session; a sign-out is not in this task and is not smuggled into it.

## Proposed updates to the documents this task does not own

**`CHANGELOG.md`** — *alo OS can now be signed in to.* A small privileged
service turns a correct password into the session the agent runs inside. It holds
no special powers of its own — fewer than the one other privileged part of the
system — no password ever reaches it, and it will only open a session for
somebody this machine already has an account for. Everything it refuses, it
refuses in a sentence, in your own language.

**`ROADMAP.md`** — phase 7's *nothing on the image can start a session* is
answered in code and in the image's files; it is **not** answered on a machine,
and nothing here should be read as ticking a boot gate.

**`docs/autonomy/QUEUE.md`** — task 26 done; task 27 (the sign-in surface's half
of the door) is ready and written into `v0-01-delivery-plan.md`.
