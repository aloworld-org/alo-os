# A refusal the machine made, told apart from one the change earned

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **twice in one day a finished task was parked under a sentence blaming it.**

The gates already run a failing gate a second time before believing it, and the
sentence they add when it fails twice is *run twice and refused both times, so
this is the work rather than the machine*. That is true of a flake. It is false
of a machine that is out of memory, because **a compiler with no memory fails as
reliably as a broken change does**, and a second run cannot tell them apart.

Both of these happened on 2026-09-11 and both were reported as the work:

    error: failed to write to `…/full.rmeta`: Cannot allocate memory (os error 12)

    A connection attempt failed because the connected party did not properly
    respond… Error code: Wsl/Service/0x8007274c

The first is WSL reaching the 6 GB cap `.wslconfig` gives it, which a
whole-workspace build with two lanes gating at once goes past. The second is the
distribution not answering at all — the gates never ran. Neither says anything
about the change, and a person reading the journal is sent looking for a defect
that is not there.

So a refusal is now checked against the machine's own words before it is
believed, and one that matches is reported the way a missing BPF mount or a full
disk already is: **this machine is not ready to be gated**, with the gate named
and everything it said kept underneath.

**Deliberately narrow.** Four phrases, each the machine's rather than a
compiler's opinion of a change, and a test puts four real breaks — a missing
method, a failed assertion, an unused variable, a red suite — in front of it to
insist they are still the change's. This must never become a road for a broken
change to reach `main`.

**The journal now records what the gates said.** It noted *that* a task was sent
back for repair and never *why*, so the only place the error survived was the
parked commit — which is written later and only if parking works. Diagnosing
today meant reading commit messages to find out what a gate had objected to
twenty minutes earlier.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — a refusal names whose it is, and the journal carries it.
