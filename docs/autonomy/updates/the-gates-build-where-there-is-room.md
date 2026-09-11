# The gates build where there is room, not where there is none

**Date:** 2026-09-11
**Workstream:** the build loop (`tools/kernel-loop`)
**Contributor:** Claude, in `C:\dev\alo-os-claude`
**Task:** 30 of `docs/autonomy/v0-01-delivery-plan.md`
**Status:** ready for integration

## What was wrong

The gates ran in one place and measured another.

Every gate is handed to the distribution, and the supervisor's readiness check
asked `df -B1 --output=avail .` — `.` being the directory the checkout is in,
which on this machine is a Windows drive 98% full. The build itself went to
`$HOME/target-claude` on the distribution's own filesystem, which has 868 GB
free. So the 12 GiB reserve refused runs over a shortage that was nowhere near
the build. On 2026-09-11 that refusal parked a task that was finished, and the
answer people reached for was to delete a lane's build directory by hand — an
hour of recompilation spent to free space that was never scarce where the
compiling happened.

The second half was quieter. `$HOME/target-claude` is named for a *lane*, not
for a *checkout*: two checkouts running this same supervisor would compile the
same workspace into one directory, which is two workers overwriting each other's
artefacts while both watch a build that should not be rebuilding. A shared
`target` is a lock, and a lock is a lane waiting.

## What changed

**`tools/kernel-loop/src/where_it_builds.rs`** (new) owns both halves of the
question, because *which directory* and *how much room is in it* were only ever
one question answered in two places.

- **One build directory per checkout.**
  `$HOME/alo-builds/<the checkout's name>-<fingerprint of its path>` — readable,
  so a person can find it, and unique, so two clones both called `alo-os-claude`
  in different places never meet. On this checkout that is
  `$HOME/alo-builds/alo-os-claude-bd192ccccbc3745b`. It is made once per run and
  handed to every gate after that: a run that chose twice would rebuild the
  workspace in the middle of a task and give two answers about one tree.
- **The reserve is asked of the directory the build uses**, with
  `df -B1 --output=avail,target`, and the refusal names the filesystem it
  measured and how much is free on it. Measured on this machine after the
  change: 935 951 208 448 bytes free on `/`, against 12 GiB required. Before the
  change the same check was reading a Windows drive with about 4 GiB free that
  no gate writes a byte to.
- **An answer that cannot be read is a refusal**, not a pass. A reserve that
  waved a run through whenever it failed to measure anything would not be a
  reserve, and the failure it would wave through is the one that fails as a
  linker rather than as a build.
- **A machine where the directory cannot be made falls back** to Cargo's own
  `target/` beside the checkout — today's behaviour, with the reserve then
  measured on the checkout's filesystem — and says so in a line carrying what
  `mkdir` said. A supervisor that refused to gate because it could not make a
  directory would have turned a convenience into a new way to stop.
- **Nothing removes a build directory.** The ones from before —
  `target`, `$HOME/target-claude`, `$HOME/alo-os-target` — are named when a run
  starts, with `du -sh` and the sentence that whether any of them goes is a
  person's decision. `nothing_here_can_remove_a_build_directory` reads this
  file's own source to keep it that way, in the shape
  `recovering.rs` already uses for the pathspec it may not take.

**`tools/kernel-loop/src/gates.rs`**: the disk entry is gone from `READY`,
because it was never a constant — it depends on where this checkout builds.
`the_machine_is_ready` asks `where_it_builds::there_is_room` first, and
`no_readiness_check_asks_this_drive_how_much_room_a_build_has` keeps a shell
`df` from reappearing in the list. The bridge is split in two: `running` sets
`CARGO_TARGET_DIR` from the chosen directory, and
`without_a_target_directory` is what the two questions asked *before* there is an
answer — can the directory be made, and how much room is on its filesystem — go
through, so that making the choice cannot ask for the choice.

**`tools/kernel-loop/src/main.rs`**: `run`, `publish` and `verify` each say
where they are building before they build anything, through the journal so the
line is in `.kernel-loop/loop.log` a week later as well as on the terminal.

**`docs/autonomy/SHARED_MAIN.md`**: the sentence naming this lane's target
directory said `/root/target-claude`, which is no longer true. Only this lane's
half was touched; the desktop lane's `/root/alo-os-target` and its Windows C:
preflight are as they were.

## Decisions taken, and why

1. **A new file rather than more of `gates.rs`.** `gates.rs` is already the gate
   list, the runner, the readiness checks and the bridge. Where a build goes and
   how much room is there is a second reason to change it, and CLAUDE.md's
   fourth law says that gets split in the change that discovered it.
2. **The name is `<checkout>-<fingerprint>`, not a hash alone.** A directory
   nobody can recognise is one nobody cleans; the readable half is the point of
   saying it out loud at the start of a run.
3. **FNV-1a written out, not a dependency.** Sixteen hex characters that
   distinguish paths. A crate added to shorten a directory name would be a
   dependency inside the program that decides what gets published.
4. **The same rule on both hosts.** The Linux half sets `CARGO_TARGET_DIR` too,
   from `$HOME`. One rule is one thing to reason about, and a checkout on a
   Linux host gains the same *one per checkout* guarantee.
5. **A cold rebuild is paid once, knowingly.** The 44 GB in
   `/root/target-claude` is not thrown away — nothing removes it — but the next
   run does not use it, so the first gate run after this lands a full workspace
   build. That is the price of a directory named for the checkout rather than
   for the lane, and moving the old one into place would have been this program
   mutating a build directory, which the task's constraint forbids for good
   reason.
6. **Unquoted paths, made safe by construction** rather than by quoting. The
   bridge assembles a command line by joining words with spaces; the chosen path
   has no character in it that a shell would have to be protected from, and
   `a_checkout_with_an_awkward_name_still_names_a_plain_directory` is what holds
   that.

What did **not** change: the gates themselves. Same gates, same order, same
refusals, same twice-before-believed rule, same per-crate clean before gating.

## Verification

Windows 11 host, gates bridged into WSL Ubuntu. Run from
`C:\dev\alo-os-claude\tools\kernel-loop`:

```
cargo fmt --all                                  clean
cargo clippy --all-targets -- -D warnings        clean, zero warnings
cargo test                                       77 passed; 0 failed
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps   clean
```

Each acceptance test was then run on its own with `--exact`; all thirteen passed
one at a time. End to end on the real machine, from the checkout root:

```
tools/kernel-loop/target/debug/alo-kernel-loop.exe verify
alo-kernel-loop: the gates build in $HOME/alo-builds/alo-os-claude-bd192ccccbc3745b, …
alo-kernel-loop: build directories from before this one are still here, and nothing
                 in this loop removes them: target, $HOME/target-claude, $HOME/alo-os-target. …
alo-kernel-loop: there is no handoff waiting, …
```

and the reserve's own question, asked as the code asks it:

```
$ df -B1 --output=avail,target $HOME/alo-builds/…
       Avail Mounted on
935951208448 /
```

**Not run here:** the full workspace suite, deliberately — the supervisor runs
it after this worker, and two finished tasks have been lost waiting on it. No
product crate was touched. Nothing about this is certified-hardware acceptance:
WSL is development evidence, as `gates.rs` has said since it was written.

## Limitations

- The fingerprint is of the path *as this host writes it*. A checkout reached by
  two different names — a drive letter on one host, a mount point on another —
  is two build directories. It costs a rebuild and never shares one, which is the
  safe way round.
- `the_old_ones` knows the directories this repository has actually used. A
  directory somebody made by hand under another name is not named by it.
- The first gate run after this is a cold workspace build.

## Proposed updates to the shared documents

Consolidation is the integration owner's; these are proposed, not made.

- **CHANGELOG.md** — *The build loop builds where there is room.* Each checkout
  now compiles into a build directory of its own on the filesystem the gates run
  on, said out loud when a run starts; the 12 GiB reserve is measured on that
  filesystem rather than on the drive the checkout happens to sit on, and says
  which filesystem it measured. Nothing removes any build directory.
- **QUEUE.md** — task 30 of the v0.01 delivery plan done; task 31, *The weights a
  machine arrives with*, ready and depending on nothing.
- **STATE.md** — reference this report.
- **ROADMAP.md** — no change; no product promise moved.
- **docs/autonomy/DELIVERY.md** — its *Runner* section describes the desktop
  supervisor's Windows C: preflight, which is unchanged and untouched here. If
  that lane later adopts the same measurement, the paragraph is where it would be
  written. Left to its owner rather than edited across lanes.

## The next task

The plan named none after 30, so task 31 is written in it: **The weights a
machine arrives with**. The reasoning is short — ADR 0025 was accepted as Option
D, and two of the three things standing in front of *the local model is what the
machine arrives ready to run* are now built (the pinned runtime is on the image,
and `crates/alo-setting-up` is the four choices). The third, weights, is
untouched, and the open question the ADR left — image or setup — is a decision
inside that work rather than a blocker in front of it.
