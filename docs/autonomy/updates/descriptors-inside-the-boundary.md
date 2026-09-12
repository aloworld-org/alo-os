# A descriptor opened before the turn began cannot move contents past the grant

- **Date:** 2026-09-12
- **Workstream:** kernel enforcement (`alo-bounding-map`, `alo-bounding-kernel`,
  `alo-bounding`, and one test in `alo-files`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **A descriptor opened before the turn began cannot move contents
  past the grant** — `docs/autonomy/kernel-enforcement-plan.md`, task 12
- **Status:** ready for integration. The one gap in the crate that moved
  contents past a grant is closed and measured closed; one remainder is named
  and not closed, with the reason. Nothing is ticked *on the machine*.

## What changed, in one paragraph a person can read

Until today the kernel decided what an agent's turn could *open*, and a file
the agent's service already had open when the turn began was never asked
about. That was measured moving a private key's contents into a folder the
turn was allowed to write. The boundary now decides on **every read and
write**, whether the descriptor was opened inside the turn or before it: a
turn reading a file nobody granted through a handle that already existed gets
`EACCES` before a byte has moved, the service's own record cannot be written
from inside a turn, and a turn can no longer write itself out of its own
boundary. A descriptor to a file inside the grant is untouched, the service's
socket to the person still answers, and a pipe still carries. Nothing about
grants, the indicator, the record or the egress policy changed, and the loader
holds the same two capabilities it held yesterday.

## What changed, for whoever reads the code

### A seventh hook: `file_permission`

`crates/alo-bounding-kernel/src/kernel.rs` declares it; `deciding.rs` decides
it as `decide_use`. It runs on every `read`, `write`, `pread`, `sendfile`,
`splice` and `getdents` on the machine. For every process that is not a turn it
is one hash lookup and a return, the same price as the six hooks before it.
Inside a turn it reads the file's kind and walks up from the file's own
directory entry — the walk `file_open` already makes, through the same
`entry_of` and `upwards_from` — and answers `EACCES` for anything that meets no
granted place. It reads none of the bytes and writes nothing down.

**It is asked of the using thread's control group.** That is what closes the
inherited case rather than restating it, and it is the same reasoning task 13
gave for `socket_sendmsg`: a descriptor the daemon opened outside any turn is,
at the moment a turn reads through it, being used by the turn.

**Two kinds of file are stepped aside from, by what they are.** A socket,
because `socket_sendmsg` decides about every message on one by where the bytes
are going, and a hook that refused a socket by its place in the filesystem
would refuse the daemon its answer to the person and a question its provider
(ADR 0020). And a pipe, because it holds no contents of its own: what comes
through it a process outside the boundary put there, and what goes into it
reaches a process the service already talks to. The kind is read from the
inode's mode, which is one more offset — `i_mode`, `Field::InodeMode`, the
fourteenth slot of `FIELDS` — and the map is still an array of sixteen with
the spare slots held at zero. A terminal, a device, the cgroup filesystem and
the record are *not* stepped aside from: none is a place a grant is over, and
`file_open` already refused each of them by name.

### The verifier's stack, and why the kind is read in its own frame

The first build was refused by the kernel's verifier: *combined stack size of
2 calls is 576. Too large.* A BPF program has 512 bytes of stack across every
call it makes at once; a bound is 128 of them, the walk's seven offsets 64,
and reading the kind inside the walk's frame put the program 64 bytes over.
`kind_of` therefore fetches its own four offsets and returns before the walk
begins, so the walk keeps exactly the shape `decide` has already verified. It
costs three more reads of kernel memory per use inside a turn. The rustdoc on
`kind_of` says this, so the next person who tidies it into `Fields` learns why
not from the comment rather than from the verifier.

### The way out of a turn, and why it is a thread that never went in

Leaving a boundary was the turn's own write into `home/cgroup.threads` through
a descriptor `Turns::under` opened before the first turn ever ran — the gap,
used on purpose, and the reason task 6 called closing it a decision rather than
a patch. That write is now refused like any other, so a turn's thread cannot
end its own boundary at all, which was the fourth row of the gap's table and is
now a refusal the suite measures.

`crates/alo-bounding/src/inside.rs` therefore starts a thread of the service
beside the turn, **before** the turn's thread goes in and from a thread still in
`home`, so `home` is where it stays. It waits on a channel in memory; when the
work is over the turn's thread sends its own number — read by `gettid` before
it went in — and the keeper writes that number into `home/cgroup.threads`
through the descriptor the service holds. The turn's thread waits for the
answer on a futex, which is not a file and opens nothing. On a panic inside the
work, `Inside`'s drop asks the same. A keeper that is gone or does not answer
is `NotBounded::NotBroughtBack`, which fails closed as it always did: the entry
and the control group stay, and the service stops.

**Nothing is started.** A thread is not a program; `tests/a_turn_is_this_thread.rs`
still reads the crate's source for `Command`, `fork`, `exec` and `posix_spawn`
and finds none. One `rustix` call, `gettid`, moves from a dev-dependency to a
dependency of `alo-bounding` with the `thread` feature; the manifest says why.

### Landlock is not the mechanism

The plan asked for this to be a finding if it was one. Landlock decides at
`open` — its filesystem hooks are the open, the path operations and truncation;
there is none on a read or a write — so a descriptor opened before the
restriction is exactly as usable after it, which is the gap restated. And a
Landlock ruleset is irrevocable for the thread it is applied to, so a turn that
is a thread of the daemon could never be released from one; closing this with
Landlock would have meant a turn that is a process, which is a change to what a
turn *is* and belongs in an ADR. The BPF LSM is ADR 0015's mechanism and it
needed one hook more. No second privileged component was added.

### What is not closed, and why it is named rather than built

A **mapping**. `mmap` of a file is `mmap_file`, not a read: a file mapped into
memory is read by the processor rather than by a syscall, so a mapping of an
inherited descriptor made inside a turn is a way to its contents that
`file_permission` does not see. It is not hooked, and it is not reproduced in
the committed suite, because there is no safe spelling of `mmap` in Rust —
`std` has none, `rustix`'s is `unsafe`, and so is every crate's that wraps it —
and `unsafe` is forbidden outside `alo-bounding-kernel`'s one file. That is the
same rule that keeps `truncate(2)` out of the suite. A hook nobody can show
refusing is a hook nobody can show working, so it was not added on belief;
`docs/quirks.md`, the crate documentation, `deciding.rs` and the plan's
hardening table all name it, and `what_a_turn_inherits_is_written_down.rs`
fails the day it arrives while the documents still call it unwatched.

### The documents

`docs/quirks.md`'s entry *A descriptor opened before a turn began is inside no
boundary* is now *A descriptor opened before a turn began is decided about on
every use*, with a table of seven rows — what a turn inherits, what the
boundary refuses it now, what it still permits, where it is reproduced, and
the release — and the mapping named beside it. The two entries around it that
said the gap was open say what closed it. `crates/alo-bounding/src/lib.rs`,
`turns.rs`, `inside.rs`, `deciding.rs`, `kernel.rs`, `imposing.rs` and
`pinned.rs` say the same in the places an auditor of the code reads.

## Files changed

| File | What changed |
|---|---|
| `crates/alo-bounding-map/src/field.rs` | `Field::InodeMode`, the fourteenth offset; `ALL` is fourteen |
| `crates/alo-bounding-kernel/src/kernel.rs` | The seventh hook, `file_permission` |
| `crates/alo-bounding-kernel/src/deciding.rs` | `decide_use`, `kind_of`, `entry_of`; the account of what is watched |
| `crates/alo-bounding/src/imposing.rs` | Seven hooks attached and pinned |
| `crates/alo-bounding/src/pinned.rs` | The seventh pin, `file_permission`; `every_hook` is seven |
| `crates/alo-bounding/src/fields.rs`, `testing.rs` | Fourteen offsets; the fixture's `i_mode` is an `unsigned short`, as `umode_t` is |
| `crates/alo-bounding/src/inside.rs` | A turn is brought home by a thread of the service that was never in it |
| `crates/alo-bounding/src/turns.rs` | The way out, described as it is now |
| `crates/alo-bounding/src/lib.rs` | *What a turn inherits, and what the boundary now says about it* |
| `crates/alo-bounding/Cargo.toml` | `rustix` with `thread`, for `gettid` |
| `crates/alo-bounding/tests/what_a_turn_inherits.rs` | Ten tests through `Turns::doing`: six refusals, each beside the use inside the grant it must not break |
| `crates/alo-bounding/tests/what_a_turn_inherits_is_written_down.rs` | Holds the quirks entry to the programme in both directions |
| `crates/alo-bounding/tests/the_boundary_decides_and_forgets.rs` | Ordinary programs read and write beside their opens |
| `crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs` | The exact list of hooks is seven |
| `crates/alo-bounding/tests/a_turn_is_this_thread.rs`, `what_a_bound_turn_can_still_change.rs`, `what_a_bound_turn_can_still_reach.rs` | Prose that described the gap as open, and the count of hooks |
| `crates/alo-files/src/failed.rs` | A refused read is the sentence a refused open is |
| `docs/quirks.md` | The entry rewritten; two cross-references |
| `docs/autonomy/kernel-enforcement-plan.md` | Task 12 done; the row moved to section 1; the mapping named in section 3 |
| `docs/autonomy/updates/descriptors-inside-the-boundary.md` | This report |

## Decisions taken here

- **`file_permission`, not Landlock**, for the reasons above.
- **A socket and a pipe are stepped aside from; everything else is walked.**
  The pipe was found rather than designed: the first full run refused every
  child-process test its pipe to its parent, and the honest reading is that a
  pipe is the same kind of thing as a Unix socket — a channel between
  processes with no contents of its own — not a file at rest.
- **The keeper is a scoped thread per turn**, started in `Turns::doing`,
  rather than a thread that lives for the life of `Turns`. Its lifetime is the
  turn's, it needs no lifecycle in `given_back`, and it costs one thread per
  turn beside a control group made and removed and a map entry written and
  taken away.
- **`mmap_file` is not hooked**, because it cannot be shown refusing.
- **The quirks heading changed.** The old heading described a gap that no
  longer exists; the entry says what it used to be called.

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started. `CARGO_TARGET_DIR` is this checkout's own, chosen the way the
supervisor chooses it.

### Acceptance evidence — each run on its own

| Acceptance criterion | Test |
|---|---|
| The reproduction flips from *reaches* to *refused*, in the same file, with the refusal named, and no byte moves | `. alo-bounding what_a_turn_inherits a_file_opened_before_the_turn_began_is_refused_inside_it` |
| A write through such a descriptor fails at the syscall, and the file is undisturbed | `. alo-bounding what_a_turn_inherits a_file_opened_for_writing_before_the_turn_began_is_refused_inside_it` |
| The record's own shape — an append — is refused a line from inside | `. alo-bounding what_a_turn_inherits a_record_opened_before_the_turn_began_is_refused_a_line_inside_it` |
| A directory descriptor is refused its listing, and one inside the grant lists | `. alo-bounding what_a_turn_inherits a_directory_opened_before_the_turn_began_cannot_be_listed_inside_it` |
| A descriptor to a path inside the grant is untouched — read through and written through | `. alo-bounding what_a_turn_inherits a_descriptor_to_a_file_inside_the_grant_is_untouched` |
| A socket is left to the message hook: the daemon can still answer from inside | `. alo-bounding what_a_turn_inherits a_socket_opened_before_the_turn_began_is_left_to_the_hook_that_decides_messages` |
| A pipe carries on inside a turn, in both directions | `. alo-bounding what_a_turn_inherits a_pipe_opened_before_the_turn_began_carries_on_inside_it` |
| A turn cannot write itself out of its boundary, and ends anyway | `. alo-bounding what_a_turn_inherits the_way_out_of_a_turn_is_refused_to_the_turn_and_the_turn_ends_anyway` |
| The account is true of the programme and complete, in both directions | `. alo-bounding what_a_turn_inherits_is_written_down what_a_turn_inherits_is_written_down_where_an_auditor_will_find_it` |
| …and the check would catch each way it can rot | `. alo-bounding what_a_turn_inherits_is_written_down the_check_catches_an_account_that_has_stopped_being_true` |
| The hook, outside a turn, leaves no trace — reads and writes beside the opens | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| The loader's capability set is unchanged and `alo-image` holds it to two | `crates/alo-image/src/checking.rs`, `an_opener_given_a_capability_is_caught` — see below |
| The record carries the refusal the way it carries every other, in words `alo-saying` collects | `. alo-files failed a_read_the_kernel_refused_is_the_sentence_a_refused_open_is` |
| The fourteenth field is looked up and held to its width | `. alo-bounding-map field every_field_is_in_the_list_exactly_once` and `. alo-bounding fields every_field_is_found_where_this_kernel_keeps_it` |
| Every hook has a pin and the list is one | `. alo-bounding pinned the_boundary_is_pinned_where_the_decision_says_it_is` |

The `alo-image` line names the test that already holds the loader's unit
file to `CAP_BPF` and `CAP_SYS_ADMIN` and catches a third; nothing in
`alo-image` or in `alo-boundaryd`'s unit file changed, and the handoff names
only tests in files this task touched, so that line is here as the criterion's
answer rather than in the handoff's evidence. `git diff` on the loader's unit
file and on `alo-image` is empty.

### Gates

Filled in from the runs on this machine; the supervisor runs every gate and
then each piece of evidence on its own before anything is published.

- `cargo fmt --all --check` — clean
- `cargo clippy -p alo-bounding-map -p alo-bounding --all-targets -- -D warnings`
  under WSL — clean
- `cargo clippy --workspace --all-targets -- -D warnings` on Windows — clean
- `cargo test -p alo-bounding-map -p alo-bounding -p alo-boundaryd` under WSL —
  passing, every kernel test against the real loaded programme
- `cargo test -p alo-files` — passing, on Windows
- `cargo test -p alo-agentd --test a_turn_is_bounded_by_the_kernel --test a_question_is_bounded_by_the_kernel`
  under WSL — passing: a real verb and a real question inside the boundary,
  with the seventh hook attached
- `cargo doc --no-deps` for the touched crates with `RUSTDOCFLAGS=-D warnings` — clean
- the BPF target's formatting and clippy on the pinned nightly — clean

### Not run here

- **The whole workspace's suite**, which the supervisor runs after this task
  as the instructions require.
- **Windows gates for `alo-bounding`**, which compiles to nothing there.
  `alo-files`' test ran on Windows and is host-independent.
- **Certified hardware.** Every measurement above is WSL2.
  `docs/hardware.md` says that cannot certify a machine, so **no *On the
  machine* box is ticked by this report** and none may be.

## Limitations

- **A mapping is not closed**, as above, and is named everywhere the closure
  is.
- **`file_receive` is not hooked** and no longer needs to be for this gap: a
  descriptor that arrives over a socket is decided about the moment it is
  used, like any other. Nothing in this repository passes one.
- **A terminal is refused from inside a turn.** Under `systemd` the daemon's
  standard error is a journal stream socket, which is left to the message
  hook; run by hand it is a terminal, and a write to it from inside a turn is
  now refused. Nothing prints from inside a turn — `a_turn_is_this_thread.rs`'s
  rule — and `std`'s panic hook ignores a write it cannot make, so what this
  costs is a panic message on a developer's terminal, not a turn.
- **A pipe is left alone**, and the reasoning is in the quirks entry. A named
  FIFO in a folder nobody granted, opened before a turn, is readable inside it
  — but what is read is what a process outside the boundary wrote, never the
  contents of a file at rest.
- **The socket row's reproduction is task 13's file**, cross-referenced rather
  than duplicated; what this file adds is the file hook's carve-out.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **A file the service already had open is inside the boundary too.** Until
> now the kernel decided what an agent's turn could open, and a file that was
> already open when the turn began — the service's own record, a handle a verb
> with a bug in it was holding — was never asked about, which was measured
> moving a private key's contents into a folder the turn was allowed to write.
> The boundary now decides on every read and write, whether the handle was
> opened inside the turn or before it: reading a file nobody granted through
> one is refused before a byte moves, the record cannot be written from inside
> a turn, and a turn can no longer write itself out of its own boundary. A
> handle to a file inside the grant, the service's socket to the person, and a
> pipe are untouched. One thing remains and is written down: a file mapped
> into memory from inside a turn is read by the processor rather than by a
> syscall, and nothing safe in Rust can reproduce that, so it is named rather
> than closed.

### `docs/autonomy/QUEUE.md`

Task 12 of the kernel-enforcement plan is complete. It closes the one gap in
`alo-bounding` that moved contents past a grant. It leaves one remainder,
`mmap_file`, named in the plan's hardening table with the reason it is not in
the committed suite. Tasks 14, 15 and 16 are unchanged and ready.

### `docs/autonomy/STATE.md`

Reference `docs/autonomy/updates/descriptors-inside-the-boundary.md`. The
kernel-enforcement plan's hardening table has one fewer open row and one
sharper one; the honest sentence is still *implementation complete for the
in-scope v0.01 requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed. Nothing here is a v0.01 delivery commitment and nothing
here is measured on certified hardware.
