# A file's access list, read, is inside the grant

- **Date:** 2026-09-13
- **Workstream:** kernel enforcement (`alo-bounding-kernel`, `alo-bounding`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **A file's access list, read, is inside the grant** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 20
- **Status:** ready for integration. The hook is on the programme and
  measured; the reproduction task 19 left standing is flipped; the next task
  is written in the plan. Nothing is ticked *on the machine*.

## What changed, in one paragraph a person can read

This morning's change made the boundary decide what an agent's turn may
learn *about* a file it was never granted: its size, its attributes, where a
link points. Its own measurement found one question still answered. A file's
access list — who may read it and who may not, by name — is read with the
same call as an attribute, and since Linux 6.2 the kernel takes that read to
a hook of its own, past the one that had just been closed. So a turn refused
the names of a file's attributes was still told who its readers were. The
boundary now sits on that hook too: outside the grant the access list is
refused, inside it the list read back is exactly the one that was put, and a
process that is not a turn is answered as it always was. Nothing about
grants, the indicator, the record or the egress policy changed, and the
loader holds the same two capabilities it held this morning.

## What changed, for whoever reads the code

### One hook, and its shape was confirmed against the kernel on the day

The plan carried the shape task 19's reader had printed for
`bpf_lsm_inode_get_acl` and said it was to be confirmed rather than copied.
It was: the same reader, still at `/root/alo-builds/scratch-protos-19/protos`,
run over this machine's `/sys/kernel/btf/vmlinux` before a line was written,
printed

```text
bpf_lsm_inode_get_acl(idmap: *struct mnt_idmap, dentry: *struct dentry, acl_name: *const char) -> int
bpf_lsm_inode_set_acl(idmap: *struct mnt_idmap, dentry: *struct dentry, acl_name: *const char, kacl: *struct posix_acl) -> int
bpf_lsm_inode_getxattr(dentry: *struct dentry, name: *const char) -> int
```

so the entry is `arg(1)` and the previous module's decision `arg(3)`. The
second and third lines are why it was read rather than assumed in either
direction: the access-list pair carries the mount's identity mapping before
the entry, and `inode_getxattr` one hook up does not. Task 19 was the case
for that rule; this task is the rule kept.

`inode_get_acl` in `crates/alo-bounding-kernel/src/kernel.rs` is the
twenty-third hook. It reads the entry, defers to a previous module's
refusal, and asks `decide_question` in `deciding.rs` — the walk
`inode_getxattr`, `inode_listxattr` and `inode_readlink` already make from
the entry of the file being asked about. No new deciding function: the file
exists, a grant can be over a single file, and the question is the same. The
name of the list is not read; the access list and the default list of a file
outside the grant are refused alike. `decide_question`'s documentation now
says why the access list is a fourth hook there and not the second, and the
module's list of what is not watched loses the row.

### Loader and pins

`imposing.rs` attaches twenty-three hooks; `pinned.rs` pins twenty-three
links, the new one at `/sys/fs/bpf/alo/inode_get_acl`, `0600 root:root` like
the rest, with `read_access_list_hook()`. `every_hook()` and
`every_hook_named()` are still the one list; the pin test names the ten
attached last in order and refuses two pins sharing a name. `in_place.rs`
asks the machine about twenty-three pins before every turn with no change to
its loop. `alo-boundaryd` counts hooks through `every_hook()` and needed no
change; its tests were run.

### The reproduction, before and after

`a_files_access_list_is_not_yet_inside_the_grant` in
`crates/alo-bounding/tests/the_kernel_refuses_what_a_turn_reads_about_a_file.rs`
was task 19's standing reproduction. It was run first, against the programme
at `6f72631` with the four hooks on it and the fifth not, and passed in the
direction it behaved: a bound turn read the forty-four-byte list off a file
it had just been refused `open` on. Then the hook was written and the test
became `a_files_access_list_is_inside_the_grant`, through the same
`refused_outside_and_answered_inside` the five questions beside it use:
`EACCES` at the syscall outside the grant with the secret undisturbed, and
inside the grant the forty-four bytes read back compared to the forty-four
that were put — a wrong answer is reported as `0`, which no refusal has, so
a hook that let the read through and had it answer with a different list
would fail rather than pass. A process that is not a turn is answered all
six questions with the programme loaded.

The list the fixture puts on both files carries a named user and a mask.
That detail is task 19's, and it matters twice: an access list that says no
more than the mode bits is not stored — the kernel folds it into the mode
and a read answers `ENODATA` — so a least list would have left nothing for a
turn to be answered and nothing to be refused.

### What moved elsewhere

- `the_boundary_decides_and_forgets.rs` gives every file of its ordinary
  day a list the kernel has to keep, reads it back, asserts it is the one
  set, takes it away, and finds nothing written down. The least list the
  day already sets and removes is kept for the attribute hooks; the new
  `an_access_list_the_kernel_keeps()` says why a second one is needed.
- `a_turn_without_a_boundary_does_not_run.rs` refuses a turn over the new
  pin with no line of its loop changed; its prose counts are twenty-three
  and `inode_get_acl`'s pin is the first taken.
- `the_unwatched_mutations_are_written_down.rs`'s exact list is
  twenty-three, and `what_a_bound_turn_can_still_change.rs`'s header names
  the five hooks on what a turn reads and says they were never rows there.
- `docs/quirks.md`: every count of twenty-two is twenty-three; the entry
  *What a turn reads about a file is inside the grant* carries the fifth
  hook's shape in the table beside the four, moves the access-list bullet
  and table row from *still* to closed, and says the one thing a bound turn
  can still learn about a file outside its grant is whether a name exists.
- `docs/contracts/agent-verbs.md` says a bounded turn cannot read the
  access list of a file outside the call's own places, and no longer lists
  it among what an adapter may assume a turn can still learn.
- The plan: task 20 is marked done, the section-3 row is struck and moved
  to section 1 with its test, the counts are twenty-three, and task 21 is
  written.

## Decisions taken here, and why

- **No new deciding function.** The plan says `decide_question`, and the
  question is the same: the file exists and is asked about by its entry. A
  fourth caller of the same walk is what keeps the programme reviewable.
- **The name of the list is not read.** `system.posix_acl_access` and
  `system.posix_acl_default` are both who may do what to a file nobody
  granted; reading the name would only be a reason to allow one of them.
- **The ordinary day sets a second list rather than changing the first.**
  The least list the day already uses exercises `inode_set_acl` and
  `inode_remove_acl`; reading a list the kernel does not store would have
  asked `inode_get_acl` nothing while looking as if it did.
- **Task 21 is a decision, not a hook.** The one filesystem row left in
  section 3 that moves contents past a grant is the mapping, and every task
  since 12 has stepped around it because there is no safe spelling of
  `mmap` in Rust and a worker may not add an `unsafe` block. Writing the
  hook without a committed reproduction would weaken the gate this
  workstream holds itself to; lifting the rule is the owner's. So the next
  task is the ADR with the options and a recommendation, and the hook is
  the task after it. That is the shape the loop's own instructions name for
  a decision a worker may not take.
- **The test file's name is unchanged.** The access list is the sixth
  question about a file in the file that measures the other five.

## Acceptance, mapped to tests

Every line is a test that runs against the real loaded programme unless it
says otherwise.

| Criterion | Test |
|---|---|
| The access list of a file outside the grant is `EACCES` inside a bound turn, with the file undisturbed; the same read inside the grant answers with the list that was put, byte for byte | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_files_access_list_is_inside_the_grant` |
| A process that is not a turn is refused none of the six questions, the access list among them, and answered rightly | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_process_that_is_not_a_turn_is_answered_what_it_always_was` |
| The read outside a turn joins the ordinary day, with a list the kernel has to keep, and leaves no trace | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| `Pinned` has the pin, the list is one, the ten attached last are in order | `. alo-bounding lib pinned::tests::the_boundary_is_pinned_where_the_decision_says_it_is` |
| A turn is refused over the new pin, by name, with no line of the loop changed | `. alo-bounding a_turn_without_a_boundary_does_not_run a_turn_is_refused_when_the_programme_is_not_held_on_a_hook` |
| The documentation test's exact list is twenty-three, and the hook is named where an auditor reads | `. alo-bounding the_unwatched_mutations_are_written_down every_mutation_this_boundary_does_not_watch_is_written_down` |

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started, from `/mnt/c/dev/alo-os-claude` with this checkout's own build
directory as the supervisor chooses it
(`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`). **WSL is development
evidence and never certified-hardware acceptance**; no *on the machine* box is
affected.

### The reproduction, before the hook

`cargo test -p alo-bounding --test the_kernel_refuses_what_a_turn_reads_about_a_file a_files_access_list_is_not_yet_inside_the_grant`
against the programme at `6f72631`, before a line was changed:

```text
test a_files_access_list_is_not_yet_inside_the_grant ... ok
```

Passing there means the list was answered about a file outside the grant,
which was the gap.

### After the hook

- `cargo test -p alo-bounding --test the_kernel_refuses_what_a_turn_reads_about_a_file`
  — 7 passed, 1 ignored (the child), `a_files_access_list_is_inside_the_grant`
  among them.
- `cargo test -p alo-bounding` — every target passing: 36 unit tests and
  the twenty integration files, every kernel test against the real loaded
  programme with twenty-three hooks attached;
  `the_boundary_decides_and_forgets` 4 passed, 1 ignored;
  `a_turn_without_a_boundary_does_not_run` 8 passed;
  `the_unwatched_mutations_are_written_down` 4 passed;
  `what_a_bound_turn_can_still_change` 1 passed, 1 ignored;
  `what_a_turn_inherits` 10 passed.
- `cargo test -p alo-boundaryd` — 7 unit tests and 3 integration tests
  passing, the loader attaching and pinning twenty-three hooks.
- `cargo fmt --all` — clean.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean.
- `cargo doc --no-deps -p alo-bounding -p alo-boundaryd` with
  `RUSTDOCFLAGS=-D warnings` — clean.

### Not run here

- **The whole workspace's suite**, by instruction; the supervisor runs it.
  No crate outside `alo-bounding` reads a file's access list on the
  production path.
- **Certified hardware.** Every measurement above is WSL2.

## Limitations

- **Cost.** `inode_get_acl` runs on every access-list read on the machine.
  For a process that is not a turn it is one hash lookup and a miss; a
  `getfacl` is not a `stat`, and the busy hook remains the one task 19
  added. Not measured on a certified machine.
- **One thing a bound turn can still learn** about a file outside its
  grant: whether a name exists, by `access(2)`, because `inode_permission`
  is deliberately not hooked. Named in `docs/quirks.md`.
- **The mapping remains**, as before, and task 21 is the decision about how
  it is reproduced.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **A file's access list, read, is inside the grant.** This morning's
> change let the boundary decide what a turn learns about a file it was
> never granted, and its own measurement found one question still answered:
> a file's access list, who may read it and who may not, which the kernel
> takes to a hook of its own. The boundary now sits on that hook. Outside
> the grant the list is refused, inside it the list read back is the one
> that was put, and a process that is not a turn is answered as it always
> was.

### `docs/autonomy/QUEUE.md`

Task 20 of the kernel-enforcement plan is complete. Task 21, *How a mapping
is reproduced, decided*, is written and ready; its deliverable is an ADR,
and the hook waits on it.

### `docs/autonomy/STATE.md`

Reference
`docs/autonomy/updates/a-files-access-list-read-is-inside-the-grant.md`.
The hardening table's `inode_get_acl` row closes; the filesystem row that
remains is the mapping, now with a task that decides how it is reproduced.
The honest sentence is still *implementation complete for the in-scope
v0.01 requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed.
