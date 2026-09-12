# Sockets already open, and datagrams sent without connecting, inside the boundary

- Date: 2026-09-12
- Workstream: kernel enforcement (`alo-bounding-map`, `alo-bounding-kernel`,
  `alo-bounding`, and one test in `alo-asking`)
- Contributor: Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- Task: **A socket already open, and a datagram sent without connecting, are
  inside the boundary** — `docs/autonomy/kernel-enforcement-plan.md`, task 13
- Status: ready for integration. Two reproduced gaps closed and one named gap
  the connect hook carried closed with them; nothing ticked *on the machine*.

## What changed, in one paragraph a person can read

Until today the kernel decided where an agent's turn could connect, and only
that. A connection made before the turn began was never asked about, and a
datagram sent without connecting asked nothing at all — both were measured
carrying bytes past a bound turn's boundary, and both were written down as
v0.5 work. The boundary now decides on **every message** a turn sends, wherever
the socket came from and whether or not it was ever connected. A turn writing
to a destination nobody showed the person gets `EACCES` before a byte leaves;
a turn writing to the one destination it was shown, or to a service on this
machine, or to the daemon's own Unix socket, is untouched. Nothing about grants,
the indicator, the record or the egress policy changed.

## What changed, for whoever reads the code

### A sixth hook: `socket_sendmsg`

`crates/alo-bounding-kernel/src/kernel.rs` declares it; `deciding.rs` decides
it as `decide_message`. It runs on every `sendmsg`, `sendto`, `send` and
`write` on a socket, machine-wide. For every process that is not a turn it is
one hash lookup and a return, the same price as the five hooks before it. Inside
a turn it reads a handful of words and compares numbers; it does not read the
bytes of the message and writes nothing down.

**It is asked of the sending thread's control group**, which is what closes the
inherited-socket gap rather than restating it. A socket remembers the cgroup it
was *made* in, so a cgroup `skb` programme — the other candidate in
`network-boundary-decisions-proposed.md`, option 4 — would have attributed a
turn's writes on the daemon's inherited socket to the daemon and closed nothing
here. An LSM hook runs in the writing thread's context, and `alo-agentd` puts
exactly that thread into the turn's cgroup. That is ADR 0013's *attribution of
every one to the turn that caused it*, on the message.

**A message has up to two destinations and both are checked.** The address it
names (`msg_name`, filled by `sendto`, null for `send` and `write`) and the peer
the socket is joined to (`skc_daddr`, `skc_v6_daddr`, `skc_dport` on the
`struct sock`). The kernel picks which one the bytes follow by protocol, and a
programme that guessed the protocol would be one somebody could arrange to
guess wrong. A message on a network socket with neither is refused: it has no
destination the programme can read, and a destination that cannot be checked is
refused in every hook of this boundary. So is a named address of a family the
programme cannot read — `AF_UNSPEC`, which the kernel reads as IPv4 on a
datagram socket, is on a network socket a destination that cannot be checked,
not one that stays home.

**The rest is the connect hook's, unchanged.** Loopback is exempt for ADR
0007's reason and with the same cost. A socket whose family is not a network
address — the daemon's Unix socket to the person, D-Bus — is not egress and is
allowed. `Departure` and `Departures` are untouched; one departure is still one
address and one port, and it permits a datagram there exactly as it permits a
connection.

### Six offsets more, and a path through a named member

`alo-bounding-map`'s `Field` grows from seven to thirteen: `socket.sk`,
`sock.__sk_common.skc_family`, `.skc_dport`, `.skc_daddr`, `.skc_v6_daddr`, and
`msghdr.msg_name`. `FIELDS` grows from eight slots to sixteen. **No third map**
— `the_program_has_nowhere_to_write_what_it_sees` still asserts exactly
`["BOUNDS", "FIELDS"]` and is not touched.

`struct sock` keeps everything about its peer inside a **named** member,
`__sk_common`, and inside that the address and port sit in unnamed unions
holding unnamed structures. The type-information reader in `alo-bounding`'s
`btf.rs` followed unnamed members already; it now follows a dotted path through
named ones and adds the offsets up, refusing a path through a pointer. The
fixture puts `__sk_common` eight bytes in rather than first, where the real
kernel keeps it, so the addition is measured rather than passing because every
part of it was zero. Several of the six are genuinely zero on this kernel
(`skc_daddr` opens `sock_common`, `msg_name` opens `msghdr`); the map is an
array, so zero is read as zero and the programme's `NetworkFields::found` says
so beside the older `Fields::found`, whose *refuses rather than defaulting to
zero* is about a slot the map does not have.

### Loader and pins

`imposing.rs` attaches six hooks; `pinned.rs` pins six links, with
`socket_sendmsg` at `/sys/fs/bpf/alo/socket_sendmsg`, `0600 root:root` like the
others. Every attach or no boundary at all, as before. `alo-boundaryd`'s tests
count hooks through `every_hook()` and needed no change; they were run.

### The reproduction file, flipped

`crates/alo-bounding/tests/what_a_bound_turn_can_still_reach.rs`, against the
real loaded programme, every listener the test's own, nothing reaching a
network:

| Case | Before | Now |
|---|---|---|
| Inherited connection, destination nobody showed | bytes arrived | **`EACCES` at the write; the server, which accepted the connection outside the turn, hears nothing** |
| Inherited connection, destination the turn was shown | — | carries on, bytes arrive |
| Unconnected datagram, destination nobody showed | arrived | **`EACCES` at `sendto`; nothing arrives; the same datagram from a process that is not a turn then arrives** |
| Unconnected datagram, destination the turn was shown | — | goes |
| Unconnected datagram, loopback, unshown | — | goes (ADR 0007) |
| Proxy on loopback | carried out | **carried out, unchanged** — ADR 0021's decision, not this hook's |
| Unix socket | connected | connected **and written on** |

Every case keeps the control it had: the same child in the same turn is first
refused a TCP connection to a destination nobody showed it, and nothing is
believed unless that happened.

### What else moved

- `the_boundary_decides_and_forgets.rs` sends twenty datagrams on loopback
  outside any turn beside its opens, each heard back, and still finds two maps,
  no turns, the same offsets and no trace line — the sixth hook held to
  *decides and forgets* the way the first five are, and to *unaffected outside
  a turn*, since a dropped datagram would fail the count.
- `the_unwatched_mutations_are_written_down.rs` names six hooks exactly and
  requires each to be mentioned in `deciding.rs`, `lib.rs` and the quirks table
  — which is how a sixth hook arrives with a person looking at it.
- `what_a_turn_inherits_is_written_down.rs` drops `a socket already connected`
  from the exact list of rows, and `docs/quirks.md`'s table drops the row. The
  four file rows stay, with the decision they wait on untouched.
- `alo-asking`'s `openai.rs` gains one unit test: `EACCES` on a connect and
  `EACCES` on a write are the same `ureq` error and the same `WentWrong`, so a
  refused message reaches the record in the words a refused connection does.
  No production code in that crate changed.
- `docs/quirks.md` gains an entry, *A socket already open, and a datagram sent
  without connecting, are inside the boundary*, with the three things worth an
  afternoon to the next reader; the *Four hooks* and *A descriptor opened
  before a turn began* entries are corrected where they described sockets.
- The plan: task 13 marked done, the audit tables moved three rows from
  section 3 to section 1 (the two named plus connection reuse), and decision 2
  in section 5 narrowed to the file half, which is all that remains of it.

## Decisions taken here, and why

**An LSM hook, not a cgroup programme.** Above: attribution. The proposal's
option 4 offered both; only one closes the inherited case, and the reason is
mechanical rather than a preference. A second kind of programme would also
have been a second attachment point ADR 0018's loader shape does not have.

**Both destinations checked, not one guessed.** The cheaper design reads
`msg_name` if present and the peer otherwise. On a stream socket the kernel
ignores `msg_name`, so a turn could have named a shown address on a socket
inherited to an unshown peer and had the bytes go to the peer. Checking both
costs a few reads on `sendto` and closes that.

**A message with no readable destination is refused, not allowed.** The kernel
refuses it too (`EDESTADDRREQ` or `ENOTCONN`), so nothing legitimate is lost,
and *a destination that cannot be checked is refused* is already the rule the
connect hook follows for an unreadable family.

**`AF_UNSPEC` named on a network socket is refused.** The kernel reads it as
IPv4 on a datagram socket. Treating it as *not egress* would have been a way
out one line long.

**No exemption list.** The proposal warned that a message hook would hit the
same exemption problem as `file_permission`: the daemon's own socket. It does
not, because that socket is a Unix socket and *a family that is not a network
address is not egress* was already the connect hook's rule. The way out of a
turn is a file, not a socket. So nothing about what a turn is changed, no
approval line was crossed, and the file half of decision 2 is exactly as open
as it was.

**No `unsafe` outside `kernel.rs`'s one permitted file, no kernel patch, no
third map, no new capability, no widened grant.** The loader still holds two
capabilities; `crates/alo-image` is untouched.

## Acceptance criteria, each with the test that says so

| Criterion | Test |
|---|---|
| Both reproductions flip to *refused* in the same file | `what_a_bound_turn_can_still_reach` — `a_connection_made_before_the_boundary_is_refused_inside_it`, `an_unconnected_datagram_is_refused_inside_a_bound_turn` |
| A datagram to a destination inside the grant still goes | same file, `a_datagram_to_a_shown_destination_still_goes` — and `a_connection_made_before_the_boundary_to_a_shown_destination_carries_on` for the inherited case |
| `alo-egress`'s accounting unchanged; the refusal is a kernel refusal, not an egress event | `alo-egress` untouched; the refusal is `EACCES` at the syscall with nothing shown and the listener hearing nothing, asserted in `an_unconnected_datagram_is_refused_inside_a_bound_turn` |
| Nothing leaves the machine in the test | every listener is the test's own on loopback or this machine's `eth0`; the same tests |
| Refusals recorded in the same words as a refused `connect` | `alo-asking` `a_message_the_kernel_refused_is_the_sentence_a_refused_connection_is` |
| Hooks on the turn's own cgroup; a syscall outside a turn is checked and leaves no trace | `the_boundary_decides_and_forgets` `ordinary_programs_run_under_the_boundary_and_nothing_is_written_down`, now with datagrams |
| The six hooks are documented where an auditor reads | `the_unwatched_mutations_are_written_down` `every_mutation_this_boundary_does_not_watch_is_written_down`; `what_a_turn_inherits_is_written_down` `what_a_turn_inherits_is_written_down_where_an_auditor_will_find_it` |
| The loader attaches and pins six, and refuses over any one of them | `alo-boundaryd` `a_machine_that_already_has_a_boundary_keeps_it`, `taking_a_boundary_away_leaves_none_of_its_hooks_attached` |
| The six offsets are found, through a named member, and zero survives | `alo-bounding` `fields::tests::every_field_is_found_where_this_kernel_keeps_it`, `btf::tests::a_path_through_a_named_member_adds_the_offsets_up`, `btf::tests::a_path_does_not_step_through_a_pointer` |

## Verification

Ubuntu on WSL2, kernel `6.18.33.2-microsoft-standard-WSL2`, `bpffs` mounted,
`bpf` in the started LSM list, run from `/mnt/c/dev/alo-os-claude` with the
loop's own build directory for this checkout. Executed:

- `cargo fmt --all` — clean; `crates/alo-bounding-kernel`: `cargo fmt --all
  --check` clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D
  warnings` in the kernel half — clean.
- `cargo clippy --all-targets -p alo-bounding-map -p alo-bounding -p
  alo-boundaryd -p alo-asking -- -D warnings` — clean.
- `cargo test -p alo-bounding-map`, `-p alo-bounding-kernel`, `-p alo-asking`,
  `-p alo-boundaryd`, `-p alo-bounding` — see the handoff's evidence block and
  the supervisor's run; the flipped file passed 7 of 7 against the real loaded
  programme on first run, in ten seconds.
- Cross-crate, as a sanity check and not as this task's gate:
  `alo-agentd`'s `a_question_is_bounded_by_the_kernel` and
  `a_turn_is_bounded_by_the_kernel`, and `alo-turn`'s
  `whether_a_question_runs_inside_the_boundary` — the production path writes
  on its request socket from inside the turn to the destination it registered,
  which the message hook must permit. The supervisor runs the whole workspace
  regardless.
- `ls /sys/fs/bpf` after the runs: nothing of this checkout's left pinned. The
  other checkout's supervisor was running `cargo test --workspace` on this
  same kernel throughout the last two runs, and its live fixtures' pins came
  and went under their own names; nothing was touched, and every kernel test
  here passed alongside it, which is what `Waited::on_this_kernel` is for.

Not executed: the full workspace suite, by instruction. Not measured: anything
on certified hardware. **WSL is development evidence and never
certified-hardware acceptance**; no *on the machine* box is affected.

## Remaining limitations

- **A file descriptor opened before the turn began** is exactly as open as it
  was, and it is the only remaining gap in this crate that moves contents past
  a grant. Task 12 and decision 2 of the plan own it; nothing here narrows or
  widens that.
- **A proxy on loopback** is reproduced as it was. ADR 0021.
- **Cost.** `socket_sendmsg` runs on every message the machine sends. Outside
  a turn that is one hash lookup and a miss, the same as `file_open` on every
  open. It has not been measured on a certified machine, and `docs/hardware.md`
  is where that measurement belongs when one exists.
- **A hook on the message reads no contents.** It does not, and the test that
  counts maps and trace lines is what says so; but the reader of `kernel.rs`
  should know that it *could*, which is why ADR 0015's *decides and forgets*
  matters one hook more than it did.

## Proposed shared-document updates

Not made here — the integration owner owns these four files.

- **CHANGELOG.md** — *The kernel boundary now decides on every message an
  agent's turn sends, not only on every connection it makes: a socket that was
  open before the turn began, and a datagram sent without connecting, are
  refused a destination the person was not shown, before a byte leaves. A
  connection kept open past the withdrawal of its destination is refused on
  its next message. Loopback and local sockets are unaffected.*
- **ROADMAP.md** — the line that says the boundary's network half is
  `socket_connect` checking non-loopback addresses now has a second hook
  beside it; no box moves.
- **docs/autonomy/QUEUE.md** — nothing new. Task 14 is next in the plan.
- **docs/autonomy/STATE.md** — this report's path; three rows moved in the
  kernel plan's audit; decision 2 is now the file half only.
