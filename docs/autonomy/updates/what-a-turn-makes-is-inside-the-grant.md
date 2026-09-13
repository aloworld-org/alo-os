# What a turn makes is inside the grant

- **Date:** 2026-09-13
- **Workstream:** kernel enforcement (`alo-bounding-kernel`, `alo-bounding`)
- **Contributor:** Claude Code, kernel-enforcement workstream, in
  `C:\dev\alo-os-claude`
- **Task:** **What a turn makes is inside the grant** —
  `docs/autonomy/kernel-enforcement-plan.md`, task 18
- **Status:** ready for integration. The last five rows of the hardening
  table's filesystem line are closed and measured closed; the table of
  unwatched mutations in `docs/quirks.md` is empty and says so. Nothing is
  ticked *on the machine*.

## What changed, in one paragraph a person can read

Until today an agent's turn could leave a name of its choosing anywhere on
the machine: an empty file, a file made without opening it, a folder, or a
symbolic link, in a folder nobody had granted it — and could remove any empty
folder. None of that put a byte of anybody's document anywhere, because
putting bytes into a file is an open and an open was already refused; what it
left was litter in somebody's filesystem, with a name the turn chose, that
outlived the turn. The boundary now decides all five by where the name is:
making a file, folder or link outside the grant is refused before anything
exists, removing a folder outside the grant is refused with the folder still
there, and every one of them inside the grant lands — including the open
that writing an archive is. Nothing about grants, the indicator, the record
or the egress policy changed, and the loader holds the same two capabilities
it held yesterday.

## What changed, for whoever reads the code

### Five hooks, and their shapes were read from the kernel first

The arguments of `bpf_lsm_inode_create`, `inode_mknod`, `inode_mkdir`,
`inode_rmdir` and `inode_symlink` were read out of this machine's
`/sys/kernel/btf/vmlinux` with a throwaway Rust reader before a line of the
programme was written, because the rename hook's trap in `docs/quirks.md` is
what a guessed argument looks like. All five put the folder's inode first and
the entry second — `inode_unlink`'s shape, not `inode_link`'s — and the
previous module's decision after the hook's own arguments:

| Hook | Arguments | Entry | Previous decision |
|---|---|---|---|
| `inode_create` | `(struct inode *dir, struct dentry *dentry, umode_t mode)` | `arg(1)` | `arg(3)` |
| `inode_mknod` | `(struct inode *dir, struct dentry *dentry, umode_t mode, dev_t dev)` | `arg(1)` | `arg(4)` |
| `inode_mkdir` | `(struct inode *dir, struct dentry *dentry, umode_t mode)` | `arg(1)` | `arg(3)` |
| `inode_rmdir` | `(struct inode *dir, struct dentry *dentry)` | `arg(1)` | `arg(2)` |
| `inode_symlink` | `(struct inode *dir, struct dentry *dentry, const char *old_name)` | `arg(1)` | `arg(3)` |

None of the modes, the device number or the link's target is read.

### Four decided by the folder, one by the entry

`decide_making` in `crates/alo-bounding-kernel/src/deciding.rs` is what
`inode_create`, `inode_mknod`, `inode_mkdir` and `inode_symlink` ask. The
entry each is handed is **negative** — it names a place in a folder, and has
no inode — so the function reads the entry's parent and walks upwards from
the folder exactly as an open would, through the same `upwards_from` every
other file hook uses. That is the answer `a_name_being_made` already gives
the rename hook's destination, and it is the folder `alo_files::Reaching`
already puts among a turn's places for anything a verb creates, so no bound
widened and the `O_CREAT` open that writing an archive is lands inside the
grant. `inode_rmdir` asks `decide_delete`, by the entry being removed, as
`inode_unlink` does: the directory exists, and a grant over a single
directory is a grant over removing it. Not a turn is one hash lookup and a
miss, as everywhere.

### Loader and pins

`imposing.rs` attaches eighteen hooks; `pinned.rs` pins eighteen links, the
five new ones at `/sys/fs/bpf/alo/inode_create`, `inode_mknod`,
`inode_mkdir`, `inode_rmdir` and `inode_symlink`, `0600 root:root` like the
rest, with `create_hook()` and its four siblings. `every_hook()` and
`every_hook_named()` are still the one list; the pin test names the five as
attached last in that order and refuses two pins sharing a name.
`in_place.rs` asks the machine about eighteen pins before every turn with no
change to its loop. `alo-boundaryd` counts hooks through `every_hook()` and
needed no change; its tests were run.

### The reproductions, before and after

The four tests in `crates/alo-bounding/tests/what_a_bound_turn_can_still_change.rs`
were task 5's reproductions, each asserting a name **landing** outside the
grant with a refused open beside it. They were run against the programme as
committed at `16eba10`, before a line of this change, and all four passed —
the gap was there. They are now
`crates/alo-bounding/tests/the_kernel_refuses_what_a_turn_makes.rs`, in the
shape `the_kernel_refuses_an_attribute_change.rs` set: a child process joins
the turn's control group, is refused the control open, makes the thing
outside the grant, makes the same thing inside it, and then uses what it made.
Five tests, each asserting the control refused, the making outside `EACCES`
with **nothing made or removed** and the secret undisturbed, the making
inside allowed and there, and the use of it:

- `a_file_made_by_opening_is_inside_the_grant` — the refusal number was
  `EACCES` before and after; what the parent asserts is the name's absence,
  because the number alone passed on the morning the gap was open. Inside,
  the file takes bytes.
- `a_file_made_without_opening_is_inside_the_grant` — `mknod`, which met no
  hook at all before. Inside, the file takes bytes.
- `a_directory_made_is_inside_the_grant` — inside, the directory takes a file.
- `a_directory_removed_is_inside_the_grant` — the directory outside is still
  there; inside, it goes and is made again.
- `a_symbolic_link_made_is_inside_the_grant_and_leads_nowhere_the_turn_can_go`
  — the link inside points at the secret, is made, and the read through it
  is refused; the parent reads through it from outside to prove the refusal
  is the boundary's and not a broken link's.
- `a_process_that_is_not_a_turn_makes_what_it_always_could` — all five, in
  process, with the programme loaded, refused none.

`what_a_bound_turn_can_still_change.rs` keeps what it had left to say — the
control, the legitimate write into the grant, and that `execve` outside the
grant is a refused `file_open` — as one test, and its header says where the
four reproductions went.

### What else moved

- `the_boundary_decides_and_forgets.rs` — the ordinary day gains twenty
  rounds of a file made by opening, a file made without, a directory with a
  file in it and a link, each taken away again, and finds nothing written
  down.
- `the_unwatched_mutations_are_written_down.rs` — `EVERY_HOOK` is the
  eighteen. `the_table_under` now answers `None` for a heading that is gone
  and `Some` of nothing for a heading with no rows, and
  `whether_it_is_written_down` refuses the first and accepts the second: the
  table is empty since today, and a heading that moved is still caught. The
  refusal-catching test covers both, and the parser test shows a present
  heading with no rows read as an empty table rather than a missing one.
- `a_turn_without_a_boundary_does_not_run.rs` — prose counts moved to
  eighteen; the loop reads `every_hook_named()` and did not change, so the
  five new pins are each removed and each refuses a turn by name.
- Counts of thirteen became eighteen in `lib.rs`, `imposing.rs`,
  `in_place.rs`, `failing.rs` and `pinned.rs`; `deciding.rs`'s list of what
  is not watched lost the five and says when and how they closed;
  `lib.rs`'s section on what the boundary watches says the same where an
  auditor of the crate reads.
- `docs/quirks.md` — the entry *Four hooks are not a filesystem* keeps its
  heading, because the contract and every report since task 5 point at it
  and the documentation test holds it by name, and its table is empty with
  the five rows listed beneath it with the date each closed. A new entry,
  *What a turn makes is inside the grant*, has the five shapes, the cost,
  the three things the next reader should know and the measurement table.
  The no-boundary entry's pin count moved from twelve to eighteen.
- `docs/contracts/agent-verbs.md` — no longer tells an adapter author that a
  bounded turn can make files and folders outside its places; says what it
  cannot make since today, and that what remains is a mapping.
- `docs/autonomy/kernel-enforcement-plan.md` — task 18 is marked done, the
  five rows moved from section 3 to section 1, the hook count is eighteen,
  and task 19 is written.

## Decisions taken here, and why

- **The reproductions moved to a file of their own rather than flipping in
  place.** Task 14 set the precedent: `the_kernel_refuses_*` is where a
  refusal beside its allowance lives, and the shape those tests need — the
  same thing made outside and inside, then used — is not the shape the
  reproduction file had. `what_a_bound_turn_can_still_change.rs` stays,
  because it still has two true things to say and the documentation test
  names it, and its header says where the rest went.
- **The unwatched table is empty and its heading is kept.** The alternative
  was removing the entry, which would break the contract's pointer, every
  report's, and the documentation test that reads it by name. An empty
  table under a present heading is the honest state; the test was taught to
  tell that from a heading that has gone, and both directions are shown
  refusing or accepting in `the_check_catches_a_list_that_has_stopped_being_true`.
- **The link's target is not read.** The plan asked that the four be
  decided by the folder, and a link is a name; where it leads is `file_open`'s
  to decide, and the symlink test measures that a link inside the grant
  pointing outside it is allowed and useless. Reading the target would mean
  parsing a path in the kernel, which is a second resolver waiting to
  disagree with the first.
- **`inode_rmdir` reuses `decide_delete`** rather than a function of its own:
  it is the same question of the same kind of thing, and a directory removed
  and a file unlinked cannot then come to different answers about the same
  entry.
- **Task 19 is what a turn reads *about* a file.** The plan named no task
  after 18. What remains on the filesystem is the mapping, which cannot be
  reproduced without `unsafe`; what remains unhooked and reproducible is
  metadata: `stat`, `getxattr`, `listxattr` and `readlink` on a file outside
  the grant all answer inside a bound turn today, and an extended attribute's
  value is bytes somebody put there. Written with acceptance and constraints
  in the plan's shape, `inode_permission` deliberately excluded with the
  reason.

## Acceptance criteria, each with the test that says so

| Criterion | Test |
|---|---|
| A file made by `open(O_CREAT)` outside the grant is `EACCES` with no file left; inside the grant it is made and takes bytes | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_file_made_by_opening_is_inside_the_grant` |
| A file made by `mknod` outside the grant is `EACCES` with no file; inside, made and takes bytes | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_file_made_without_opening_is_inside_the_grant` |
| A directory made outside the grant is `EACCES` with no directory; inside, made and takes a file | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_directory_made_is_inside_the_grant` |
| An empty directory outside the grant is not removed, `EACCES`; inside, removed and made again | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_directory_removed_is_inside_the_grant` |
| A symbolic link outside the grant is `EACCES` with no link; inside, made, and a read through it to a file outside the grant is `EACCES` | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_symbolic_link_made_is_inside_the_grant_and_leads_nowhere_the_turn_can_go` |
| A process that is not a turn makes all five and is refused none | `. alo-bounding the_kernel_refuses_what_a_turn_makes a_process_that_is_not_a_turn_makes_what_it_always_could` |
| The reproduction file keeps the control, the legitimate write and `execve`, and says where the rest went | `. alo-bounding what_a_bound_turn_can_still_change a_turn_cannot_start_a_program_that_is_outside_its_bound` |
| `Pinned` gains the five where the thirteen are, attached last in a known order, and `every_hook_named()` is the one list with no duplicate | `. alo-bounding lib pinned::tests::the_boundary_is_pinned_where_the_decision_says_it_is` |
| A turn is refused over each of the five new pins with no line of the loop changed | `. alo-bounding a_turn_without_a_boundary_does_not_run a_turn_is_refused_when_the_programme_is_not_held_on_a_hook` |
| The five hooks outside a turn leave no trace, added to the ordinary day | `. alo-bounding the_boundary_decides_and_forgets ordinary_programs_run_under_the_boundary_and_nothing_is_written_down` |
| The two maps stay two | `. alo-bounding the_boundary_decides_and_forgets the_program_has_nowhere_to_write_what_it_sees` |
| The five leave the quirks table, the eighteen are documented where an auditor reads, and nothing listed as unwatched is watched | `. alo-bounding the_unwatched_mutations_are_written_down every_mutation_this_boundary_does_not_watch_is_written_down` |
| An empty table under a present heading is accepted and a heading that has gone is refused | `. alo-bounding the_unwatched_mutations_are_written_down the_check_catches_a_list_that_has_stopped_being_true` |

## Verification

Run on Ubuntu under WSL2, Linux 6.18.33.2, as root, with a BPF filesystem
mounted at `/sys/fs/bpf` and `bpf` among the security modules the kernel
started, from `/mnt/c/dev/alo-os-claude` with this checkout's own build
directory as the supervisor chooses it
(`/root/alo-builds/alo-os-claude-bd192ccccbc3745b`). **WSL is development
evidence and never certified-hardware acceptance**; no *on the machine* box is
affected.

### The hooks' shapes

A throwaway BTF reader in Rust, run once over `/sys/kernel/btf/vmlinux`, printed
the five function prototypes in the table above; `bpf_lsm_inode_unlink`,
`bpf_lsm_inode_setattr` and `bpf_lsm_file_ioctl` were printed beside them and
agree with what `kernel.rs` already says of those three.

### The reproductions, before the hooks

`cargo test -p alo-bounding --test what_a_bound_turn_can_still_change` against
the programme at `16eba10`:

```text
test a_file_is_made_outside_the_bound_and_stays_empty ... ok
test a_symbolic_link_is_made_and_the_turn_cannot_read_through_it ... ok
test a_file_is_made_outside_the_bound_without_being_opened_and_stays_empty ... ok
test a_turn_cannot_start_a_program_that_is_outside_its_bound ... ok
test directories_are_made_and_removed_but_a_folder_with_anything_in_it_is_not ... ok
test result: ok. 5 passed; 0 failed; 1 ignored
```

Passing there means each name landed outside the grant, which was the gap.

### After the hooks

- `cargo test -p alo-bounding` — every target passing: 36 unit tests and
  the nineteen integration files, every kernel test against the real loaded
  programme with eighteen hooks attached;
  `the_kernel_refuses_what_a_turn_makes` 6 passed, 1 ignored (the child);
  `what_a_bound_turn_can_still_change` 1 passed, 1 ignored;
  `the_boundary_decides_and_forgets` 4 passed, 1 ignored;
  `the_unwatched_mutations_are_written_down` 4 passed;
  `a_turn_without_a_boundary_does_not_run` 8 passed.
- `cargo test -p alo-boundaryd` — passing.
- `cargo fmt --all --check` — clean, in both workspaces.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings`
  in `crates/alo-bounding-kernel` — clean.
- `cargo doc --no-deps -p alo-bounding -p alo-boundaryd` with
  `RUSTDOCFLAGS=-D warnings` — clean.

### Not run here

- **The whole workspace's suite**, by instruction; the supervisor runs it.
  The crates that write inside a boundary on the production path
  (`alo-agentd`, `alo-turn`, `alo-files`) create files only in folders
  `Reaching` names, which is what the create hook decides by; their kernel
  tests are the supervisor's to run.
- **Certified hardware.** Every measurement above is WSL2.

## Limitations

- **Cost.** Five hooks more, on every file, directory and link made or
  directory removed on the machine; for a process that is not a turn each is
  one hash lookup and a miss. Not measured on a certified machine;
  `docs/hardware.md` is where that belongs.
- **A device node inside the grant is allowed by this boundary** and refused
  by the kernel to a person without `CAP_MKNOD`, which `alo-agentd` does not
  hold; the boundary is a floor under the ordinary bits and not a
  replacement for them.
- **The mapping remains**, as before, for the reason task 12 gave.

## Proposed shared-document updates

For the integration owner. **This contributor has not edited any of them.**

### `CHANGELOG.md`

> **What a turn makes is inside the grant.** A turn could leave an empty
> file, a file made without opening it, a folder or a symbolic link in a
> folder nobody granted it, and remove any empty folder — no byte of
> anybody's document in any of it, and all of it somebody's filesystem. The
> boundary now decides all five by where the name is: outside the grant each
> is refused before anything exists or with the folder still there, and
> inside it each lands, including the open that writing an archive is. The
> list of filesystem mutations the boundary does not watch is empty.

### `docs/autonomy/QUEUE.md`

Task 18 of the kernel-enforcement plan is complete. Task 19, *What a turn
reads about a file is inside the grant*, is written and ready.

### `docs/autonomy/STATE.md`

Reference `docs/autonomy/updates/what-a-turn-makes-is-inside-the-grant.md`.
The hardening table's filesystem line is the mapping alone; the honest
sentence is still *implementation complete for the in-scope v0.01
requirement; hardware acceptance pending*.

### `ROADMAP.md`

No change proposed.
