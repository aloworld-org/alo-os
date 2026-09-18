# A port another program held over IPv6 at start is listened on over IPv6 once it is let go of

**Date:** 2026-09-17
**Workstream:** v0.5 — the local network (`docs/autonomy/v0-5-the-local-network-plan.md`, task 36)
**Contributor:** Claude Code worker in `C:\dev\alo-os-claude`, for the repository's owner
**Status:** ready for integration

## What changed

Task 35 made the service try a network again when the kernel says another program
let go of the port. The one IPv6-only listener was left out. It was bound once, at
start. If another program held `[::]` at the port then, the log said the port was
bound over IPv4 alone, and nothing ever tried IPv6 again. A machine whose only
network is link-local found this one and could not reach it until the service
restarted. Now the IPv6 listener is tried again when the port is let go of, and
when a network changes.

- `crates/alo-agentd/src/listening_over_ipv6.rs` (new): the IPv6 listener's own
  state. It is one of four things: listening, held by another program, not to be
  had, or never asked for (a listener a test handed in). `tried_again` binds only
  from *held by another program*. It says the bind once and says nothing when the
  port is refused again. Unit tests cover:
  - a port held over IPv6 is said once, not said again when refused again, and
    bound and said once when free;
  - what is not to be had is never tried again;
  - a listener handed in is never tried;
  - a free port binds and says nothing.
- `crates/alo-agentd/src/listeners.rs`: `Listeners` holds that state instead of
  keeping the IPv6 listener in `fixed`. `changed` and `let_go_of` both try it. A
  let-go re-reads the kernel's networks only when an IPv4 network was refused, as
  before. The let-go socket is now opened **before** the IPv6 bind, so a let-go
  between that refusal and the subscription is still heard. It is also opened on a
  machine that cannot read its networks, because IPv6 needs no network to be tried
  again. `Listener::unheld` and the free function `until` replace repeated code.
  The IPv4 refusal line is byte for byte what it was. New:
  `Listeners::listening_over_ipv6`. New unit test: a port held over IPv6 at start is
  refused once and said once, a let-go that leaves it held is refused again and not
  said, and a let-go that frees it binds and says so once. The IPv4 networks do not
  move throughout.
- `crates/alo-agentd/src/wire.rs`: `Wire::listening_over_ipv6`, and the
  `port_let_go_of` doc names IPv6.
- `crates/alo-agentd/src/a_port_held_over_ipv6_at_start.rs` (new): the fixture on a
  real kernel, described below.
- `crates/alo-agentd/src/lib.rs`: both registered.
- `docs/contracts/local-network-wire.md`: one bullet added under *A port another
  program lets go of*. Nothing on the wire changed.
- `docs/quirks.md`: a new entry covering two things. A dual-stack `[::]` listener
  refuses every IPv4 listener held to an interface (measured). This host's own
  namespace binds IPv6 sockets while IPv6 is off on every interface, with no `::1`.
- `docs/autonomy/v0-5-the-local-network-plan.md`: task 36 marked done, and task 37
  written.

**Change description, for the changelog:** Sometimes another program holds alo OS's
network port over IPv6 while the computer starts, such as an installer or a service
restarting. A computer running alo OS now becomes reachable over IPv6 as soon as
that program lets go. This matters most for two computers joined by a cable with no
router, which only have IPv6 link-local addresses between them. It needs no restart
and no change to the networks. The log says once that IPv6 was taken, and once that
it is reachable again.

## Decisions

- **The IPv6 listener is a separate thing that is tried again, not one of the refused
  networks.** Refused networks are remembered by interface number, and forgotten
  when that interface goes. The IPv6 listener has no interface. It is not among the
  networks the kernel reports, and it must never be dropped because a network went.
  Counting it as a refused network would mean inventing a number and adding an
  exception to the rule that forgets them. It gets the same two retries a refused
  network does, from the same two events.
- **Only `EADDRINUSE` is tried again.** Another program holding the port stops
  when that program lets go. Other failures don't: a kernel without IPv6 answers
  `EAFNOSUPPORT` for as long as the service runs. That kind of failure is logged
  once with the original line, which is unchanged, and never tried again.
- **Where the let-go socket opens moved earlier.** It now opens before any bind, in
  both families, and whether or not the networks can be read. Before, a machine that
  could not read its networks never opened it. That machine now hears a let-go for
  IPv6, and still does not follow networks.
- **The split (law 4).** `listeners.rs` was 860 lines and was gaining a second kind
  of retry. The IPv6 state has its own reason to change, so it went into its own
  file. `Listener` became `pub(crate)` so that file can hold one.
- **How the fixture proves *a let-go that leaves the port held*.** The squatter
  accepts every connection and closes it. The far end's knock that finds the port
  not reached is one such connection: a TCP socket at the port destroyed while the
  squatter still holds it. The far end waits for the squatter to say it closed the
  connection, then for a finished round of the service. The service log is checked
  at the end for exactly one refusal.
- **The unit test connects over `::1` only where `::1` exists.** This WSL host has
  IPv6 disabled on every interface. The refusal, retry and bind are still asserted
  here, and connecting over link-local is measured by the fixture in namespaces
  where IPv6 is on.

## Acceptance criteria and evidence

Each criterion in the plan, and the test behind it:

- One machine serving as `src/main.rs` does, with no capabilities, on a `veth`
  carrying link-local IPv6 only. A program holds the port over IPv6 before the
  service starts. The port is not reached over link-local, and the machine is
  still found there with the same bytes. Once the program lets go, with no network
  changing (read with `ip monitor`), the port is reached over link-local. The log
  says the refusal once and the bind once:
  `a_port_held_over_ipv6_at_start::tests::a_port_held_over_ipv6_at_start_is_listened_on_over_ipv6_once_let_go_of`.
- A let-go that leaves the port held over IPv6 is refused again and not said again:
  the same fixture (the squatter's closed connection, and the single refusal line),
  plus
  `listeners::tests::a_port_held_over_ipv6_at_start_is_listened_on_over_ipv6_once_let_go_of`
  and
  `listening_over_ipv6::tests::a_port_held_over_ipv6_is_said_once_and_bound_once_when_let_go_of`.
- Only a port held by another program is tried again:
  `listening_over_ipv6::tests::what_is_not_to_be_had_is_never_tried_again`, and a
  wire on a listener somebody handed in never asks at all:
  `listening_over_ipv6::tests::nothing_asked_is_never_tried`.
- The decision on separate thing or refused network is written up above and in
  `crate::listening_over_ipv6`'s module doc.

## Verification

**How this change reached main's tree.** The work was first finished on 2026-09-17
and parked by the supervisor on `parked/task-36-1789651243`. It was parked because
WSL could not report free disk space, not because a gate refused it. A second
worker brought that commit onto current `main` uncommitted (`git cherry-pick -n`).
It applied cleanly. That worker reviewed it against the acceptance criteria and ran
every gate below again on the combined tree. `Cargo.lock` is among the files
because the build added `alo-capturing` to one of its dependency lists. The commit
already on `main` before this one (`c4efa25`) had left that entry out. No manifest
changed.

The gates were run in WSL Ubuntu (kernel `6.18.33.2-microsoft-standard-WSL2`) from
`/mnt/c/dev/alo-os-claude`, with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
on the combined tree:

- `cargo fmt --all` and `cargo fmt --all --check`: clean.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo test -p alo-agentd`: exit 0 in 235 s. The lib suite passed 457, failed 0,
  ignored 33, and every integration target passed.
- The evidence tests were each run on their own with `--exact`, and each passed.
  The fixture took 1.8 s.
- Mutation run, repeated on the combined tree: with `self.over_ipv6.tried_again(said)`
  removed from `Listeners::let_go_of`, the fixture failed at *reception never held
  `cable0|-|yes`: it holds `cable0|-|no`*. The line was restored byte for byte, the
  fixture passed again, and formatting was checked again.
- Dual-stack probe (Python, scratch, in `unshare -rn`): an IPv4 listener held to
  `lo` beside a dual-stack `[::]` listener at the same port gets `[Errno 98]
  Address already in use`.
- Not run: the full workspace suite (the supervisor runs it). Nothing was run on
  certified hardware; that image's kernel is not measured here.

## The gate refusal of 2026-09-17, which was the build directory and not this work

A third worker was sent at this task with a refusal from the gate `clippy, warnings
denied`: a non-exhaustive `match` on `alo_opening::Kind` in `alo-applications`' lib
test, with five kinds "not covered" at `crates/alo-opening/src/kind.rs:85`, `:88`,
`:91`, `:93` and `:95`, and the help offering `_ => todo!()` after
`Kind::ZipArchive` at `crates/alo-applications/src/spelled.rs:43`. Neither crate is
in this change, and **the kinds it was said to be missing do not exist in this
checkout**: they are the film and sound kinds `edd751b` added to `alo-opening` on
`main`, in the same commit that added their arms to `alo-applications::spelled`,
eight commits ahead of this tree. `spelled` here names every one of the eighteen
kinds this tree's `alo-opening` has.

The cause is the build directory. Since 2026-09-17 one directory serves every lane
on this machine — `$HOME/alo-builds/this-machine`, reading one source copy beside
it at `$HOME/alo-trees/this-machine` — and this tree's copy was made with `rsync
-a`, which keeps each file's own modification time. Cargo decides what to rebuild
by comparing times, so a file whose content differs from the last lane's but whose
time is *older* than the artefact built from it looks fresh: the other lane's
checkout is at `main`, its `alo-opening` was compiled there, and this lane's
older-dated `kind.rs` was then not rebuilt. `alo-applications` was compiled from
this tree's source against the other lane's `alo-opening` metadata, and the error
is exactly what that mixture means. The owner had already found the same failure
and fixed it on `main` in `f984203`, *the gates' source copy is made by content,
not by timestamp* (`--checksum --no-times`); its message names this refusal —
"another for a clippy error belonging to a different lane's work". That commit is
ahead of this checkout, so the supervisor picks it up when it rebases.

**Nothing was changed for the refusal, because nothing in the work was wrong.**
Adding a wildcard arm to `alo-applications::spelled` would have been wrong twice
over: that `match` is exhaustive here, and its module doc says its whole purpose is
that a kind `alo-opening` adds be a compile error rather than a kind a person
cannot choose for. What was done instead was to establish that, and then to gate
the tree again:

- `cargo fmt --all`: clean, no file changed. `cargo fmt --all --check` in the
  shared copy: clean.
- `cargo clippy --workspace --all-targets -- -D warnings` from
  `/mnt/c/dev/alo-os-claude` with this checkout's own build directory
  (`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`), which no other lane writes
  to: exit 0 in 5 m 02 s, `alo-applications` among the crates checked.
- The same command in the shared copy with the shared build directory, the way the
  supervisor runs it: exit 0 in 59 s, `alo-opening` and `alo-applications`
  rebuilt. The shared copy was first compared file by file against this checkout
  (`diff -rq`, excluding `.git`, `target` and `.kernel-loop`) and is byte-identical
  to it, so the source the gate read was never the mixture — only what had been
  built from it.
- `cargo test -p alo-agentd`: exit 0. The lib suite passed 457, failed 0, ignored
  33; every integration target passed.
- The five evidence tests, each run on its own with `--exact --include-ignored`:
  each passed, exit 0.

The only crate this change touches is `alo-agentd`, and the only other files are
four documents. The full workspace suite was deliberately not run here; the
supervisor runs it.

## Remaining limitations

- **A program holding the port dual-stack at start still stops the service.** The
  refusal covers IPv6 and every IPv4 network, loopback included, so nothing binds
  and `Listeners::bound` returns `NotBound::NoWire`. This is task 37, now written
  in the plan.
- A kernel without `CONFIG_INET_DIAG` hears no let-go. On such a kernel the IPv6
  refusal line says the port is tried again only when the networks next change,
  and that is what happens.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** local network task 36 done, with this report. Task 37 is
  ready.
- **ROADMAP.md:** no change.
