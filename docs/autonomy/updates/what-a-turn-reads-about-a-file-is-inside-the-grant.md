# What a turn reads about a file is inside the grant

- **Date:** 2026-09-13
- **Workstream:** kernel enforcement (`alo-bounding-kernel`, `alo-bounding`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **What a turn reads about a file is inside the grant** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 19
- **Status:** ready for integration. The four hooks are on the programme and
  measured; one question the task did not name was found by its own
  reproduction, left standing in the direction it behaves, and written as
  task 20. Nothing is ticked *on the machine*.

## What changed, in one paragraph a person can read

Until today an agent's turn could not open a file nobody granted it, could
not read or write one it already held, could not move, remove, rename or
change one — and could still ask the machine everything it knew *about* one.
Inside a turn, asking the size, owner, mode and times of a file outside the
grant was answered; so was the value of an extended attribute an application
had put on it, the names of its attributes, and where a symbolic link
pointed. A turn refused a folder's listing could ask each name in it whether
it was there and how big it was, which is most of what the listing would have
said. The boundary now decides those questions by the file being asked about:
outside the grant each is refused, inside it each is answered with the right
answer, and a process that is not a turn is answered as it always was.
Nothing about grants, the indicator, the record or the egress policy changed,
and the loader holds the same two capabilities it held yesterday.

## What changed, for whoever reads the code

### Four hooks, and their shapes were read from the kernel first

The arguments of `bpf_lsm_inode_getattr`, `inode_getxattr`,
`inode_listxattr` and `inode_readlink` were read out of this machine's
`/sys/kernel/btf/vmlinux` with a throwaway reader (about a hundred lines of
dependency-free Rust, kept in the scratchpad and not in the repository)
before a line of the programme was written. The reader was checked first
against `inode_unlink`, `inode_link`, `inode_setattr` and `file_ioctl`, whose
shapes `kernel.rs` already documents, and agreed with all four. Then:

| Hook | Arguments | Walked from | Previous decision |
|---|---|---|---|
| `inode_getattr` | `(const struct path *path)` | the path's entry, one read in from `arg(0)` | `arg(1)` |
| `inode_getxattr` | `(struct dentry *dentry, const char *name)` | `arg(0)` | `arg(2)` |
| `inode_listxattr` | `(struct dentry *dentry)` | `arg(0)` | `arg(1)` |
| `inode_readlink` | `(struct dentry *dentry)` | `arg(0)` | `arg(1)` |

**The plan's constraint guessed one of these wrong, and the reader caught
it.** It said `inode_getxattr` begins with the mount's identity mapping as
the attribute hooks do; it does not. A programme written from
`inode_setxattr`'s shape would have read a `struct dentry *` as a `struct
mnt_idmap *` and refused everything for reasons nobody could see, which is
the rename hook's trap in `docs/quirks.md` exactly. That is why the plan
insists the arguments are read first, and this task is the case for the
rule.

### Two deciding functions, one walk

`decide_asking` in `crates/alo-bounding-kernel/src/deciding.rs` is what
`inode_getattr` asks. It is handed a `struct path` rather than an entry, so
it reads the entry out of the path through the `path.dentry` offset the map
already holds, reads the inode's kind, steps aside from a socket and a pipe
for the reason `decide_use` does — neither holds contents of its own,
neither is a place a grant is over, and a copy in the standard library asks
the kind of both its ends before it moves a byte — and walks upwards from the
entry as an open would. `decide_question` is what `inode_getxattr`,
`inode_listxattr` and `inode_readlink` ask: the file exists, so it is the
walk `decide_attribute` makes from the entry being asked about, and a grant
over a single file is a grant over asking about it. `kind_of` was split into
the file half and an entry half, `kind_at`, so a `stat` handed a path and a
read handed a file ask the same question of the same inode; `entry_inside`
is the fetch-and-walk the entry hooks share. Not a turn is one hash lookup
and a miss, as everywhere.

### Loader and pins

`imposing.rs` attaches twenty-two hooks; `pinned.rs` pins twenty-two links,
the four new ones at `/sys/fs/bpf/alo/inode_getattr`, `inode_getxattr`,
`inode_listxattr` and `inode_readlink`, `0600 root:root` like the rest, with
`stat_hook()` and its three siblings. `every_hook()` and `every_hook_named()`
are still the one list; the pin test names the nine attached last in that
order and refuses two pins sharing a name. `in_place.rs` asks the machine
about twenty-two pins before every turn with no change to its loop.
`alo-boundaryd` counts hooks through `every_hook()` and needed no change; its
tests were run.

### The reproductions, before and after

`crates/alo-bounding/tests/the_kernel_refuses_what_a_turn_reads_about_a_file.rs`
was written first in the direction the boundary behaved and run against the
programme at `5836d9c`, before a hook existed: five questions — `lstat` by
name, `fstat` through a descriptor opened before the turn began, `getxattr`
of `user.alo.origin`, `listxattr`, and `readlink` — each answered about a
file the same turn had just been refused `open` on. All five passed in that
direction. The assertion is flipped: five refusals, each `EACCES` at the
syscall with the secret undisturbed, beside five answers inside the grant.

**The inside half asserts the answer is the right one**, not merely that no
error came back: the size is the file's length, the value is the one that
was put, the attribute is among the names, the link's target is the file it
was made to point at. A wrong answer is reported by the child as `0`, which
no refusal has, so the parent tells a hook that let a `stat` through and had
it answer wrongly from one that refused it. A process that is not a turn is
answered all five with the programme loaded.

### What the reproduction found that the task did not name

A file's POSIX access list is read with `getxattr` under
`system.posix_acl_access`, and since Linux 6.2 the kernel routes that read to
`inode_get_acl` — a hook of its own, exactly as the write is routed to
`inode_set_acl` past `inode_setxattr`. This programme does not sit on
`inode_get_acl`, so a bound turn refused the names of a file's attributes is
still answered its access list. `a_files_access_list_is_not_yet_inside_the_grant`
measures it in the direction it behaves and stands until task 20 flips it,
the shape task 14 left a file's flags in and task 17 was handed. One detail
for whoever writes task 20: an access list that says no more than the mode
bits is not stored — the kernel folds it into the mode and a read answers
`ENODATA`, which is how this task's first run found it — so the reproduction
carries a named user and a mask.

### What moved elsewhere

- `what_a_turn_inherits.rs` used `fstat` on an inherited handle as the proof
  that the handle was still a handle after a refused write. That `fstat`
  reaches `inode_getattr` through the descriptor's own path and is refused
  now — measured in the new file — so the proof is `fcntl`, which asks no
  hook this boundary sits on. The quirks row for a file open for writing says
  so.
- `the_boundary_decides_and_forgets.rs` asks every file of its ordinary day
  its size by name and through a descriptor, an attribute's value, its
  attributes' names and where a link beside it points, outside any turn,
  asserts each answer is right, and finds nothing written down.
- `a_turn_without_a_boundary_does_not_run.rs` refuses a turn over each of the
  four new pins with no line of its loop changed; its prose counts moved from
  eighteen to twenty-two.
- `the_unwatched_mutations_are_written_down.rs`'s exact list is twenty-two,
  and `what_a_bound_turn_can_still_change.rs`'s header names the four and
  says they were never rows there, because a question is not a mutation.
- `docs/quirks.md` gains *What a turn reads about a file is inside the
  grant*, and every count of eighteen in it is twenty-two.
  `docs/contracts/agent-verbs.md` says an adapter may no longer assume a path
  it was not granted can be checked from inside a turn, and names the two
  things it can still learn.

## Decisions taken here, and why

- **`inode_getattr` steps aside from a socket and a pipe.** The plan says
  the walk is `decide_attribute`'s; for `stat` it is, with one thing in
  front of it. `fstat` reaches this hook on any descriptor, and the standard
  library's copy asks the kind of both its ends before it moves a byte, so a
  hook that refused `fstat` on a socket would break a verb copying a file
  inside its grant towards the daemon's own socket while looking like a
  boundary. The reasoning is `decide_use`'s and the code mirrors it.
- **The `fstat` of an inherited descriptor is in this task.** It is the same
  hook and the same path; leaving it out would have left the inherited test
  asserting the opposite of what the kernel does.
- **`inode_get_acl` is task 20, not part of this one.** The acceptance names
  four hooks; the fifth was found by measurement and is small, and the
  honest shape is the one task 14 set — a standing reproduction, a quirks
  row, and a task the next worker can start from a failing test.
- **`inode_permission` is named, not hooked**, as the constraint says.
- **The test file's name** follows the task's title, as task 18's did.

## Acceptance, mapped to tests

Every line is a test that runs against the real loaded programme unless it
says otherwise.

| Criterion | Test |
|---|---|
| `stat` on a file outside the grant is `EACCES`; inside the grant it answers with the size | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_files_size_mode_owner_and_times_are_inside_the_grant` |
| `fstat` through a descriptor opened before the turn is `EACCES` outside the grant, and answers inside it | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_descriptor_opened_before_the_turn_is_refused_its_size_inside_it` |
| `getxattr` outside the grant is `EACCES`; inside it the value is the one that was put | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file an_extended_attribute_is_inside_the_grant` |
| `listxattr` outside the grant is `EACCES`; inside it the attribute is among the names | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file the_names_of_a_files_attributes_are_inside_the_grant` |
| `readlink` outside the grant is `EACCES`; inside it the target is answered | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file where_a_symbolic_link_points_is_inside_the_grant` |
| A process that is not a turn is refused none of the five, and answered rightly | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_process_that_is_not_a_turn_is_answered_what_it_always_was` |
| A file's access list is still answered inside a turn, and the reproduction stands | `. alo-bounding the_kernel_refuses_what_a_turn_reads_about_a_file a_files_access_list_is_not_yet_inside_the_grant` |
| The four outside a turn leave no trace | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| An inherited writable handle is still a handle, by `fcntl`, after its write is refused | `. alo-bounding what_a_turn_inherits a_file_opened_for_writing_before_the_turn_began_is_refused_inside_it` |
| `Pinned` has the four, the list is one, the nine attached last are in order | `. alo-bounding lib pinned::tests::the_boundary_is_pinned_where_the_decision_says_it_is` |
| A turn is refused over each of the four pins, by name | `. alo-bounding a_turn_without_a_boundary_does_not_run a_turn_is_refused_when_the_programme_is_not_held_on_a_hook` |
| The documentation test's exact list is twenty-two and every hook is named where an auditor reads | `. alo-bounding the_unwatched_mutations_are_written_down every_mutation_this_boundary_does_not_watch_is_written_down` |

The inherited-descriptor account's `fstat` row was rewritten and
`what_a_turn_inherits_is_written_down.rs` still parses and holds it; that
file is unchanged, so it is run in the suite below rather than offered as
evidence of this change.

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started, from `/mnt/c/dev/alo-os-claude` with this checkout's own build
directory as the supervisor chooses it
(`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`). **WSL is development
evidence and never certified-hardware acceptance**; no *on the machine* box is
affected.

### The hooks' shapes

The throwaway reader, run once over `/sys/kernel/btf/vmlinux`, printed the
four prototypes in the table above, and printed `bpf_lsm_inode_unlink`,
`inode_link`, `inode_setattr` and `file_ioctl` beside them in agreement with
what `kernel.rs` already says of those four. It also printed
`bpf_lsm_inode_get_acl(idmap: *struct mnt_idmap, dentry: *struct dentry,
acl_name: *const char)` for task 20, and
`bpf_lsm_inode_permission(inode: *struct inode, mask: int)`, which is why
`inode_permission` could not use the walk even if it were wanted: it is
handed an inode, which does not say where it is.

### The reproductions, before the hooks

`cargo test -p alo-bounding --test the_kernel_refuses_what_a_turn_reads_about_a_file`
against the programme at `5836d9c`, with the outside assertion reading
`Answered`:

```text
test a_descriptor_opened_before_the_turn_is_refused_its_size_inside_it ... ok
test a_files_size_mode_owner_and_times_are_inside_the_grant ... ok
test an_extended_attribute_is_inside_the_grant ... ok
test the_names_of_a_files_attributes_are_inside_the_grant ... ok
test where_a_symbolic_link_points_is_inside_the_grant ... ok
```

Passing there means each question was answered about a file outside the
grant, which was the gap. The access-list test failed that run with
`ENODATA` on both files, because the list was mode-equivalent and not
stored; it was given a named user and a mask, and it passes in the standing
direction against the programme with the four hooks on it.

### After the hooks

- `cargo test -p alo-bounding` — every target passing: 36 unit tests and the
  twenty integration files, every kernel test against the real loaded
  programme with twenty-two hooks attached;
  `the_kernel_refuses_what_a_turn_reads_about_a_file` 7 passed, 1 ignored
  (the child); `what_a_turn_inherits` 10 passed;
  `the_boundary_decides_and_forgets` 4 passed, 1 ignored;
  `the_unwatched_mutations_are_written_down` 4 passed;
  `what_a_turn_inherits_is_written_down` 4 passed;
  `a_turn_without_a_boundary_does_not_run` 8 passed.
- `cargo test -p alo-boundaryd` — passing.
- `cargo fmt --all` — clean, in both workspaces.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean, and both clippy runs were repeated
  after touching a changed source in each crate to see them re-checked.
- `cargo doc --no-deps -p alo-bounding -p alo-boundaryd` with
  `RUSTDOCFLAGS=-D warnings` — clean.

### Not run here

- **The whole workspace's suite**, by instruction; the supervisor runs it.
  The crates that ask about files inside a boundary on the production path
  (`alo-agentd`, `alo-turn`, `alo-files`) ask `symlink_metadata` and `fstat`
  only of paths that were resolved outside the boundary and are among the
  turn's places, which is what the `stat` hook decides by; their kernel tests
  are the supervisor's to run.
- **Certified hardware.** Every measurement above is WSL2.

## Limitations

- **Cost.** `inode_getattr` runs on every `stat` on the machine, the busiest
  hook here after reads and writes; for a process that is not a turn it is
  one hash lookup and a miss. Not measured on a certified machine;
  `docs/hardware.md` is where that belongs.
- **Two things a bound turn can still learn**, both named in
  `docs/quirks.md`: whether a name exists, by `access(2)`, because
  `inode_permission` is deliberately not hooked; and a file's access list,
  which is task 20.
- **The mapping remains**, as before, for the reason task 12 gave.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **What a turn reads about a file is inside the grant.** A turn could not
> open, read, write, move, remove or change a file nobody granted it, and
> could still ask the machine everything it knew about one: its size, owner,
> mode and times, the value and names of its extended attributes, and where
> a symbolic link pointed. The boundary now decides those questions by the
> file being asked about: outside the grant each is refused, inside it each
> is answered with the right answer, and a process that is not a turn is
> answered as it always was. One question is still answered and is written
> down with the task that closes it: a file's access list.

### `docs/autonomy/QUEUE.md`

Task 19 of the kernel-enforcement plan is complete. Task 20, *A file's
access list, read, is inside the grant*, is written and ready, with its
reproduction already standing.

### `docs/autonomy/STATE.md`

Reference
`docs/autonomy/updates/what-a-turn-reads-about-a-file-is-inside-the-grant.md`.
The hardening table gains one open row, `inode_get_acl`, found by this
task's own reproduction; the honest sentence is still *implementation
complete for the in-scope v0.01 requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed.
