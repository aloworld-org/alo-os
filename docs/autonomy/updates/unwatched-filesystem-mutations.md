# The filesystem mutations that remain unwatched

- Date: 2026-09-08
- Workstream: kernel enforcement (`alo-bounding`, `alo-bounding-kernel`)
- Contributor: Claude Code, kernel-enforcement workstream
- Task: Documenting the filesystem mutations that remain unwatched
  (`docs/autonomy/kernel-enforcement-plan.md`, task 5)
- Status: done — the list, in four places, with every row reproduced against the
  real loaded programme and a test that fails the day the list stops being true
- Follows: `publication-hardening-and-egress-coverage.md`, which did the same for
  the three network gaps

## What this task was, and what it turned out to be

`crates/alo-bounding-kernel/src/deciding.rs` already listed the mutations the
boundary does not watch. Nothing else did — not `docs/quirks.md`, not a contract,
and not the crate an auditor reads — and nothing anywhere reproduced them. So the
task was documentation of a known limit, and it stayed documentation; no hook was
added and no behaviour changed.

What it turned out to be was **documentation that runs**. A written-down gap
stops being a gap when somebody closes it, and nothing about closing it makes the
paragraph change. Security documentation rots in the direction of *understating*
the protection, which is the direction nobody re-reads, so a paragraph saying *a
turn can still do X* is read by the next person as the state of the boundary
whether or not it still is. Two tests now stand behind the prose: one that
reproduces every row against a running kernel, and one that parses the list and
refuses it when it has stopped being true.

## The list

`docs/quirks.md`, under *Four hooks are not a filesystem: what a bound turn can
still change*. Seven rows, each naming the hook, what a bound turn can still do
with it, why no contents leave a grant, and **v0.5** as the release that owns
closing it.

| Hook | What a bound turn can still do |
|---|---|
| `inode_symlink` | make a link in a granted folder that leads to a file nobody granted |
| `inode_create` | make a file in a folder nobody granted, by opening with `O_CREAT` |
| `inode_mknod` | make the same file without opening it at all |
| `inode_mkdir` | make a directory in a place nobody granted |
| `inode_rmdir` | remove an **empty** directory nobody granted |
| `inode_setattr` | change the mode, owner, times — **and size** — of a file nobody granted |
| `inode_setxattr` | set an extended attribute on a file nobody granted |

The property the four existing filesystem hooks were chosen for is what the list
holds up: **none of these moves a byte of somebody's file past a grant.** A
symbolic link is a name and reading through it is an open; a created file is
empty and filling it is an open; a directory holds no bytes and emptying one
needs `inode_unlink`, which is watched; an attribute is somewhere to put bytes
the turn cannot read.

That promise is narrower than *a turn cannot change anything outside its bound*,
and the whole point of writing the list down is that the second sentence is the
one an auditor supplies for themselves when nobody writes the first.

## Where it is written, and for whom

| Where | For whom |
|---|---|
| `docs/quirks.md` | the canonical table: hook, what it allows, why contents stay, release |
| `crates/alo-bounding-kernel/src/deciding.rs` | somebody reading the code that decides, beside the four hooks that do |
| `crates/alo-bounding/src/lib.rs` | somebody auditing the crate, in the section beside the network one |
| `docs/contracts/agent-verbs.md` | an adapter author, in plain words, with no hook names and a pointer to the table |

The contract addition is additive and changes nothing a caller sees: it says
there is a floor under the grants, that **inside the bound is never the same as
authorised**, what the floor covers, and that it does not cover every way a
filesystem changes.

## Two things the audit found that the task did not ask for

**A bound turn cannot start a program.** `execve` opens the file it is about to
run, `file_open` is watched, and a program outside the bound is a file outside
the bound — so `/bin/true` from inside a turn is `EACCES` like anything else.
None of the unwatched mutations can be escalated by running something that makes
the calls instead. It is a floor under law 2 rather than the law itself, which is
`alo-capability`'s, and a dynamically linked program copied into a granted folder
would still be refused its loader. It is reproduced
(`a_turn_cannot_start_a_program_that_is_outside_its_bound`) and it was found by
accident, trying to reproduce the next paragraph through a coreutil.

**The size half of `inode_setattr` is worse than the rest of the list.**
`truncate(2)` changes a file's length through that unwatched hook without opening
anything, so **a bound turn can empty a file nobody granted it.** Measured on
2026-09-08 on this kernel: the same turn was refused `open` on the file with
`EACCES` and then truncated it to zero bytes. No contents leave a grant — nothing
is read and nothing is copied, so the list's promise survives as written — and
contents are destroyed where they are, which is a different harm and a real one.

It is the only row **not reproduced in the committed suite**, and the reason is a
rule rather than an oversight. No call this repository can make reaches
`truncate(2)`: `std` has no path truncate, `rustix` has only `ftruncate` on a
descriptor, `unsafe` is forbidden outside `alo-bounding-kernel`'s one file, and
the coreutil that would do it is refused at the `execve` above. The measurement
was made by hand with a Python child joined to the turn's control group, and a
language that is not Rust is a bug in this repository, so it was not committed.
`docs/quirks.md` records the method and the limit rather than a claim, and the
hook itself *is* reproduced — through a mode change, which is what says
`inode_setattr` is unwatched.

**Nothing about either is closed here.** Both are recorded, placed at v0.5, and
left for whoever schedules that release — with a note that `inode_setattr` is the
one to take first, because it is the only one that destroys.

## The tests

### `crates/alo-bounding/tests/what_a_bound_turn_can_still_change.rs`

Seven tests against the **real loaded BPF LSM** on a running kernel. Each drives
a real turn — a control group, a bound naming one folder, a child process inside
it — and reports four things in order:

1. **the control**: an open of a file nobody granted, which must be `EACCES`. A
   turn that changed something proves nothing if the boundary was never applied,
   so every result is thrown away unless this refusal happened;
2. **the subject**: the unwatched mutation;
3. **the after**: the act that would turn that mutation into contents leaving,
   which must be refused;
4. **the legitimate write**: a file written *inside* the granted folder, which
   must be allowed — `archive_folder` does exactly that, and a boundary refusing
   it would be a regression wearing the costume of a closed gap.

| Test | Subject | After |
|---|---|---|
| `a_symbolic_link_is_made_and_the_turn_cannot_read_through_it` | the link is made | reading through it is `EACCES`, and the test reads through it from outside the turn to prove the link really leads to the secret |
| `a_file_is_made_outside_the_bound_and_stays_empty` | `O_CREAT` leaves the inode and the open is refused | writing to it again is refused; the file is asserted to be zero bytes |
| `a_file_is_made_outside_the_bound_without_being_opened_and_stays_empty` | `mknod` succeeds outright | writing to it is refused; zero bytes |
| `directories_are_made_and_removed_but_a_folder_with_anything_in_it_is_not` | an empty directory is made and removed | removing a name inside the private folder is refused, which is why the folder cannot be emptied |
| `a_files_permissions_are_changed_and_it_is_still_not_readable` | the mode becomes `0o777` | the same open is still `EACCES`, because the boundary decides by place |
| `an_extended_attribute_is_set_and_the_file_is_still_not_readable` | the attribute is set and read back | the open is still `EACCES` |
| `a_turn_cannot_start_a_program_that_is_outside_its_bound` | starting `/bin/true` is refused | so is opening it, which is the same hook and the same rule |

Data is asserted undisturbed after every refusal, which is this workstream's
standing rule.

**It runs as root, and that is the point rather than a flaw.** The ordinary
permission bits refuse nothing to root, so every *allowed* is the boundary's own
answer rather than a courtesy of the mode. On a real machine `alo-agentd` runs as
the person and those bits are still there — the boundary is a floor **under**
them, never a replacement — and the quirks entry says so.

### `crates/alo-bounding/tests/the_unwatched_mutations_are_written_down.rs`

Four tests, no kernel needed, because the list has to be checkable on a machine
that cannot run the boundary at all. It reads the hooks out of
`crates/alo-bounding-kernel/src/kernel.rs`, where `#[lsm(hook = "…")]` declares
them, parses the table out of `docs/quirks.md`, and refuses:

- a row whose hook the programme now watches — **the failure this exists for**:
  it names the row and says to come and change it;
- a hook the programme has that `deciding.rs`, `lib.rs` or the reproduction file
  does not mention, so a sixth hook cannot arrive silently;
- a row naming a release `docs/features.md` has never heard of, because *which
  release owns closing it* is what makes a limit work rather than a shrug;
- a row that says nothing about why contents stay inside the grant;
- a row nothing reproduces;
- an empty table, so a heading that moved fails rather than passing in silence.

Each of those refusals is shown happening in
`the_check_catches_a_list_that_has_stopped_being_true`, against doctored input,
beside a sound list that passes — a documentation test never seen to fail is one
nobody should believe. The parser is shown reading the real table's shape and
*not* reading a different table under a different heading in the same document.
And the five hooks are asserted as an exact list, for the reason
`the_boundary_decides_and_forgets` asserts its two maps by name.

## What did not change

No hook was added, no map was added, nothing was widened, no grant covers more,
no capability exists that did not, and `alo-capability` still decides everything
about authority. `the_boundary_decides_and_forgets` and the four filesystem
refusal suites are untouched and still pass. Nothing in this change needed an
approval, and none of the three things that would have required one — a new
capability, a widened grant, a change to what the programme remembers — is here.

## Verification

Ubuntu on WSL2, kernel 6.18.33.2, Rust 1.98.0, own `CARGO_TARGET_DIR`,
`/sys/fs/bpf` mounted, `bpf` in `/sys/kernel/security/lsm`, `dmesg` stall count
zero. All executed, all passing:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace` — 119 result lines, none failed, including the eleven
  new tests
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps` (the task's own
  named evidence)
- `crates/alo-bounding-kernel`: `cargo fmt --all --check` and `cargo clippy
  --release --target bpfel-unknown-none -Z build-std=core -- -D warnings` on the
  pinned `nightly-2026-06-01`

Each of the eleven tests was then run **on its own** with `--exact`, and each
reported exactly one passing test.

**Not run here:** the Windows half. `alo-bounding` compiles to nothing on
Windows, and no Windows run is offered as enforcement evidence.

**WSL is development evidence and never certified-hardware acceptance.** No *On
the machine* box is affected by anything in this report, and nothing here ticks
anything.

**One measurement is by hand and is labelled as such**: the `truncate(2)` result
above. It is not in the committed suite and this report does not present it as
one.

## Remaining limitations

- Every row on the list is still open, and every one is v0.5.
- `inode_setattr`'s size half destroys rather than litters, and it is the one to
  close first.
- A descriptor opened before a turn began is still usable inside it. That is plan
  task 6 and is untouched here; the socket half of it is already reproduced in
  `what_a_bound_turn_can_still_reach.rs`.
- The list is of hooks that bear on **files**. Signals, memory and everything
  else that is not a filesystem or a socket remain outside this boundary
  entirely, as `deciding.rs` says.

## Proposed shared-document updates

Not made here — the integration worker consolidates them.

**CHANGELOG.md** — nothing user-visible. This is documentation of a limit and
tests that hold it in place.

**ROADMAP.md** — no change. Nothing here is a v0.01 delivery commitment; every
row belongs to *the grant is a boundary the kernel imposes* (ADR 0013) and *the
kernel is taught what a turn is* (ADR 0015), both v0.5.

**docs/autonomy/QUEUE.md** — no new item; this workstream's plan is its list. If
one is wanted, the honest single line is *close `inode_setattr` first: a bound
turn can empty a file nobody granted it.*

**docs/autonomy/STATE.md** — two facts worth carrying. The filesystem coverage of
this boundary is now documented in four places and reproduced rather than argued,
with a test that fails when the list stops being true. And the audit found that a
bound turn can truncate a file nobody granted it, which no report before this one
said.
