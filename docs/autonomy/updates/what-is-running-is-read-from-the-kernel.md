# What is running, and what it is using — read from the kernel, not estimated

- Date: 2026-09-13
- Workstream: v0.5 the machine, measured, task 1 (`docs/autonomy/v0-5-the-machine-measured-plan.md`)
- Contributor: Claude Code, as a development worker under the kernel-loop supervisor
- Status: **ready for integration**

`docs/features.md` promises, at v0.5: *what is running, and what it is using —
processes, memory, disk and network in a window. The plain answer to "why is
it slow?", for the person who cannot or will not ask.* The window is the
desktop lane's. This task is what it shows, and the plan puts the whole of the
promise in the word **plain**: a number the person can check against the
machine, not one the machine derived and hopes is close.

## What changed

A new crate, `crates/alo-measuring`, with two dependencies (`alo-strings`,
`thiserror`) and a test that holds the manifest to exactly those.

**A `Reading` is every total the kernel keeps at one moment.** For each
process: its name and start time from `/proc/<pid>/stat`, resident memory from
`VmRSS` in `/proc/<pid>/status`, processor ticks from `utime` and `stime` in
`/proc/<pid>/stat`, bytes read and written through its files from `rchar` and
`wchar` in `/proc/<pid>/io`, bytes received and sent from `/proc/<pid>/net/dev`,
and which network namespace those two belong to from `/proc/<pid>/ns/net`. For
the machine: the sum of the `cpu` line of `/proc/stat`, and `MemTotal` and
`MemAvailable` from `/proc/meminfo`. **Every number carries a `Source`**: the
file and the kernel's own field name. A test opens that file and compares, and
so can a person with `cat`.

**`Reading::since` makes two readings and an interval into `Running`**: bytes
per second for the four counters, a share of the whole machine's processor in
thousandths, memory as a size now, and a separate list of every process that
was running at the earlier reading and is not at the later one. The interval is
an argument. Nothing in the crate reads a clock, and a test shows that the same
two readings give different rates when told a different interval, because the
crate has no way to know better.

**A number that is not there is not a zero.** `Number` has four arms and a
sentence in the reader's language for each of the three that are not a number:
*withheld* when the kernel refused to open the file (another person's process),
*not said* when the file has no such line (a kernel thread has no `VmRSS`), and
*not yet* when the process began after the earlier reading and has nothing to be
a rate over. A process that ended between readings is `Gone`, by name and pid,
in its own list.

**A pid is not a process.** Two readings are matched by pid *and* start time. A
pid the kernel reused between them is one process gone and another not yet,
rather than one process whose counters ran backwards.

**Words.** Nine phrases and one countable sentence, under `measuring.`,
collected into the machine's vocabulary through `alo-saying`'s three list
entries. The Polish test renders the countable sentence at one, three and
twenty-two others, which exercises three plural forms where English has two.

## Decisions

**Per-process network bytes are the namespace's, and the answer says so.**
Linux keeps no per-process count of bytes on the network. `/proc/<pid>/net/dev`
is the count for the network namespace the process is in: exactly the process's
own traffic for one sandboxed into a namespace of its own, and the whole
machine's for one in the default namespace. Reporting the second as though it
were the first would be a number the machine hopes is close — the one thing the
plan forbids. So every `Process` carries a `Network` naming the namespace and
how many *other* processes in the same list share it, with a sentence for the
window: *counted for this process alone* or *counted together with N other
processes*. Loopback is left out of the sum, because bytes that never left the
machine are not what *network* means to the person asking, and they would
count twice. The alternative — reporting per-process network as unmeasurable —
would have left the promise's fourth column empty on every machine; this keeps
it true on every machine and exact on the sandboxed ones.

**The share of the processor needs no clock and no `CLK_TCK`.** It is the
process's tick delta over the machine's tick delta from `/proc/stat`, both read
from the kernel, so a test can compute it from the two readings' files. One
thread busy on one of eight processors is 125 thousandths. The per-second rates
are the only numbers that use the interval, and the interval is the caller's.

**The kernel is a trait with three methods**: list a directory, read a file,
follow a link. `Disk` is the real one. `Reading::of_kernel` takes any
implementation, which is how the sampling code — the same code, not a test
double of it — is exercised on every host against a kernel a test wrote out:
one file unreadable, one process gone between the listing and its reads, one
`status` with no `VmRSS`. On Windows this crate's forty-three unit tests and
the non-Linux refusal run; the five tests that open the real `/proc` are
`cfg(target_os = "linux")` and ran in WSL.

**`Reading::now` on any host but Linux answers `NotMeasured::NotOnThisHost`.**
Not a reading of nothing: the refusal, in the reader's language, the way
`alo-agentd` is absent rather than pretending.

**A process whose `stat` cannot be read is not listed.** Without its start
time it cannot be told from a successor that reuses its pid, and a row that
might be two processes is not a fact. On Linux `stat` is world-readable, so on
a working machine this only ever means the process ended.

**Read-only is a test, not a sentence.** `nothing_here_acts_or_asks_who_is_asking`
reads the shipped source with comments and string literals removed and fails on
any identifier that spawns, signals or renices a process, writes or removes a
file, opens a socket, or reads the environment; holds every absolute path
literal to `/proc`; and assigns `Reading::now` to a function pointer that takes
nothing, which is the whole of *the list is the same whether an agent or a
person asked*.

## Acceptance criteria and the tests that hold them

| Acceptance | Test |
|---|---|
| every number read from `/proc`, named for its file, checked against it | `what_is_running_is_read_from_the_kernel::every_number_is_named_for_the_file_it_came_from_and_agrees_with_it` |
| asking twice gives a rate, with the interval passed in | `what_is_running_is_read_from_the_kernel::asking_twice_gives_a_rate_over_the_interval_passed_in` |
| a process that exited between readings is gone rather than zero | `what_is_running_is_read_from_the_kernel::a_process_that_ended_between_readings_is_gone_rather_than_zero` |
| a process that began between readings has no rate yet | `what_is_running_is_read_from_the_kernel::a_process_that_began_between_readings_has_no_rate_yet` |
| a line the kernel does not keep is not said, not zero | `what_is_running_is_read_from_the_kernel::a_number_the_kernel_does_not_keep_is_not_said_and_agrees_with_the_file` |
| a file the kernel withholds is withheld, not zero | `sampling::tests::a_file_the_kernel_withholds_is_withheld_and_not_a_zero` |
| without `/proc/stat` there is no reading, and the refusal names the file | `sampling::tests::a_machine_whose_stat_cannot_be_read_gives_no_reading_and_names_the_file` |
| nothing is read outside `/proc` | `sampling::tests::every_file_opened_is_under_proc` |
| a reused pid is two processes | `reading::tests::a_reused_pid_is_one_process_gone_and_another_not_yet` |
| no interval, and the same moment twice, are refused | `reading::tests::no_interval_and_the_same_moment_are_both_refused` |
| whose network a count is, is said beside it | `reading::tests::a_shared_network_count_says_how_many_share_it` |
| on Linux the kernel is read | `reading::tests::on_linux_the_kernel_is_read_and_this_process_is_in_the_list` |
| the list takes no account of who asked | `nothing_here_acts_or_asks_who_is_asking::the_list_takes_no_account_of_who_asked` |
| nothing signals, stops, renices, writes or opens a socket | `nothing_here_acts_or_asks_who_is_asking::nothing_in_the_shipped_source_signals_stops_renices_or_writes` |
| the numbers come from `/proc` through no rented crate | `nothing_here_acts_or_asks_who_is_asking::the_numbers_come_from_proc_and_from_no_rented_crate` |
| every sentence is read in the person's language | `what_this_crate_says::every_sentence_is_read_in_the_language_the_person_reads` |
| the words are in the machine's vocabulary | `alo-saying` `collecting::tests::every_crate_that_says_something_is_in_it` |

The non-Linux refusal, `reading::tests::on_any_other_host_nothing_is_measured`,
is `cfg(not(target_os = "linux"))` and ran on Windows; it is not in the
handoff's evidence because the supervisor's evidence run is on Linux, where it
does not exist.

## Verified

WSL Ubuntu on this development machine, `CARGO_TARGET_DIR` the supervisor's own
(`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`), foreground, exit codes read:

| Gate | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --all-targets -- -D warnings` (workspace) | clean, 0 warnings |
| `cargo test -p alo-measuring` | 43 unit, 3 + 5 + 2 integration, 1 doctest: all passed |
| `cargo test -p alo-saying` | 63 + 4 + 1 passed |
| `cargo test -p alo-collected` | 8 + 11 passed |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-measuring --no-deps` | clean |
| every evidence line, alone, `--exact` | 1 passed each, 17 lines |

Windows, this development machine: `cargo fmt --all`, `cargo clippy -p
alo-measuring --all-targets -- -D warnings`, `cargo clippy -p alo-saying
--all-targets -- -D warnings` clean; `cargo test -p alo-measuring` 43 + 3 + 2 +
1 passed, including the non-Linux refusal.

**Not run by this worker:** the full workspace suite, per the task's
instruction; the supervisor runs it. **Not claimed:** anything about a
certified machine. The five real-`/proc` tests ran on WSL2's kernel, which has
`/proc/<pid>/io`, `net/dev`, `ns/net` and a `kthreadd` with no `VmRSS`; a
physical alo OS machine has the same files, and the tests are the same tests.

### Second pass: the rustdoc gate

The supervisor refused the first handoff on the `rustdoc, warnings denied`
gate: `alo-nearby` linked `alo_capability::GrantError` and
`alo_capability::Grant` with intra-doc brackets while not depending on
`alo-capability`, so the links could not resolve. The links are on `main`
from the pairing commit, not from this task, but the gate runs on the tree as
a whole. Fixed the way every other crate here refers to an item in a crate
it does not depend on — plain backticks, no link — in `pairing.rs` and
`deliberating.rs`; adding a dependency for the sake of a link would have been
the larger change. Re-run in the same WSL target, foreground, exit codes
read: `cargo fmt --all` clean; `RUSTDOCFLAGS="-D warnings" cargo doc
--workspace --no-deps` clean; `cargo clippy --all-targets -- -D warnings`
clean; `cargo test -p alo-measuring` 43 + 3 + 5 + 2 + 1 passed,
`cargo test -p alo-saying` 63 + 4 + 1 passed, `cargo test -p alo-nearby`
54 + 9 + 5 + 1 passed.

## Remaining limitations

- **Per-process network traffic is a namespace's**, said beside the number.
  Attributing bytes to one process in a shared namespace would need eBPF or
  `nfacct`, which is a different task and possibly an ADR: it is a kernel hook
  rather than a file read.
- **The share of the processor is of the whole machine**, in thousandths. A
  window that wants *one processor's worth* multiplies by the processor count,
  which is the number of `cpuN` lines in `/proc/stat` and is not read here
  because nothing here needed it.
- **Threads are not listed separately.** `/proc/<pid>/task/` is not read; a
  process's ticks and memory are the whole process's, which is what a person
  asking *why is it slow?* is asking about.
- **Nothing here is a verb yet.** Task 5 of the plan declares the read verbs.

## Proposed changelog entry

*What is running, and what it is using* (v0.5): `alo-measuring` reads every
process's memory, processor time, bytes through its files and bytes on its
network from `/proc`, names each number for the file it came from, turns two
readings and an interval into rates, reports a process that ended as gone
rather than as zero, and says whose network traffic a count is rather than
attributing the machine's to a row. Read-only, no rented reader, Linux-only
with a refusal elsewhere.

## Proposed queue and roadmap updates

Task 1 of `v0-5-the-machine-measured-plan.md` is marked done in the plan. Tasks
2 and 3 are ready and depend on nothing; task 5 depends on 1, 2 and 3.
