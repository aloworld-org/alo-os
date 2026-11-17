# A sorted list resolved by keeping both sides, in a crate no check runs

**Concluded:** *#1353's rebase over #1354 resolved cleanly, so the gate's green
said the daemon still offers the verbs its own test says it does.* #1354 added
`accessible.set_value` to the daemon's list of verbs and #1353 added
`accessible.select_item` beside it. Both touched the same lines of the sorted
list in `crates/alo-agentd/src/starting.rs`, and the conflict was resolved by
keeping both lines, in the order the two sides happened to arrive.

**What was true.** The test that holds that list,
`this_daemon_offers_the_six_the_two_software_verbs_and_the_three_application_acts`,
sorts the daemon's verbs and compares them with a list written out by hand. On
`main` after #1353 (`026c0ac2`), that list reads:

```rust
"accessible.set_value",
"accessible.select_item",
```

`select_item` sorts before `set_value` (`l` comes before `t`), so the test fails
on `main`. It failed when it was run locally. **The gate on #1353 was green, and
so was every gate after it.** For a person using the machine, nothing is
different: the daemon offers the right verbs, and only the test's copy of the
list is out of order. What changed is that the one check that the daemon offers
exactly the verbs it should was red, and nothing anybody looks at could show it.

**The mechanism has two halves, and the second is the one worth recording.**

*First, the resolution.* Keeping both sides is the right answer to most
conflicts in a list. It is the wrong answer to a list whose order is part of
what is checked. Git compares text and does not know a list is sorted, so the
result merges without complaint and reads correctly at a glance: both verbs are
there.

*Second, nothing runs that test.* Checked on 2026-11-17 by reading every file in
`.github/workflows/`:

- the gate runs the nine gates in `tools/kernel-loop/src/gates.rs`, whose test
  filter begins `not (package(alo-bounding) + package(alo-agentd) +
  package(alo-boundaryd))`. That excludes the **whole** `alo-agentd` package,
  because some of its tests attach BPF programmes to the kernel;
- `mutants.yml` excludes `crates/alo-agentd` from the changes it mutates and
  uses the same filter for the tests it runs;
- the other `cargo test` steps (`every-try.yml`, `installer-candidate.yml`,
  `release.yml`, and `gate.yml`'s run of `tools/kernel-loop`) name other
  packages;
- `.config/nextest.toml` says the excluded suite "runs on `main` after each
  merge now, under `--profile boundary`", and `gates.rs` and `mutants.yml` say
  the same. **No workflow runs `--profile boundary`.** A search of
  `.github/workflows/` for `profile boundary` and for `--profile` finds only
  `mutants.yml`'s `--profile gates`.

So no job in CI runs any test in `alo-agentd`. The sentences saying a suite runs
after a merge describe a step that does not exist: true of an intention, read
as true of a machine.

**This is the second time, and the first was written down.** `gates.rs` already
records it, beside the filter: on 2026-10-19 `main` was red for a day because
the same test asserted eleven verbs while the daemon offered twelve, and "it
needs no root, no network and no kernel, and it ran nowhere". The same comment
measured `cargo test -p alo-agentd --lib` at **554 passed, 0 failed** as root on
one VM and said "not one of this crate's 554 library tests needs a BPF programme
attached"; the BPF is in five integration-test binaries, which it names. It then
decided, deliberately, not to change the filter, because one VM is not a hosted
runner and fourteen of the library tests want root. That was a reasoned choice,
and the gap it left open is the one this entry walked into a month later. **The
gap was named and not closed, and a named gap reads as a handled one.**

**The cure.**

- After resolving a conflict in a list that a test holds sorted, sort it again
  and read the order, not only whether both lines are present.
- After touching `alo-agentd`, run `cargo test -p alo-agentd --lib` before
  pushing. Today nothing else will, and the gate's green does not cover it. The
  gate prints its exclusions under *what these gates do not cover*; that line is
  the one to read.
- The lasting fix is a CI step that runs `alo-agentd`'s library tests, naming
  the five BPF binaries the way `the_kernel_refuses_what_a_run_was_not_given` is
  already named, and naming the fourteen tests that want root. Nothing has been
  built here: a step on the landing path is the owner's call, and this entry
  asks for it rather than adding it.
- The order on `main` is corrected in the file chooser's change (#1374), not
  here, so that one change owns that file.

Found by the lane that made the resolution, on running the crate's tests
locally for an unrelated change. Nothing caught it.
