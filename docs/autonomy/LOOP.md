# The alo OS build loop

One iteration builds **one queue item**, completely, and stops. A supervisor
runs iterations until the journal says the queue is done or that something is
wrong.

The loop exists because the work in `ROADMAP.md` is long and mostly independent,
not because it is unsupervised. Every iteration ends in a commit that met the
gate in `CLAUDE.md`, or in a halt that says why it could not.

## There are no tracks here

The supervisor's prompt names a "track", because the repository it came from has
several. **This repository has one queue and one loop.** Whatever track the
prompt names, the work is `docs/autonomy/QUEUE.md` — treat the word as noise
rather than as scope you have been asked to invent.

Halting on it once was right: an unknown track could have meant real work
somebody expected. It is written down now, so it is answered.

## What one iteration does

1. **Read `docs/autonomy/QUEUE.md`.** Take the first item that is not done and
   is not blocked. If every remaining item is blocked, write `LOOP COMPLETE`
   into the journal with the blocked list, and stop — a loop that keeps
   re-reading impossible work is a loop burning somebody's money.
2. **Read what the item names.** The ADR it implements, the contract it must
   satisfy, the section of `docs/features.md` that promised it. An item that
   cannot name those is not ready to build; mark it `needs design` and take the
   next one.
3. **Build it whole** — input, validation, policy, execution, record, error
   paths. Law 3: no stubs, no `todo!()`, no `unwrap()` outside tests. If it is
   turning out larger than one iteration, **cut its scope, never its depth**,
   and write the cut into the queue as a new item rather than leaving a
   half-built one.
4. **Pass the gate**, all of it. `cargo fmt`, `cargo clippy` with zero warnings
   *and* zero errors, tests including the refusal paths, documentation in the
   same change, a `CHANGELOG.md` line.
5. **Commit and push.** One item, one commit, a message that says what changed
   and why somebody would care.
6. **Update the queue, the roadmap and the journal.** Tick the queue item.
   Append to `docs/autonomy/STATE.md`: what was built, what the gate said, and
   anything the next iteration should know.

   **Then open `ROADMAP.md` and move the line this item served** — in the same
   commit. Almost never a tick: a queue item builds a crate, a roadmap line is a
   whole capability, and the screen or daemon that finishes it is usually still
   missing. What you write is the `· Built: … · Owed: …` clause described at the
   top of that file, naming the crate that now exists and the thing still owed.

   If the item served no roadmap line, **say so in `STATE.md` and say why** —
   the way iteration 10 did when it found *test a provider* lived only inside
   "Settings, as one place". That is a real answer. Silence is not, and silence
   is what happened: eight consecutive iterations left `ROADMAP.md` untouched
   while eight crates landed, so the file reported that nothing had been built.

   **Never resolve this by ticking.** A roadmap tick means law 3 on real
   hardware, this loop has no hardware, and a loop that learns to tick to
   discharge an obligation is worse than one that never updated the file.
7. **Stop.** One item per iteration. Two is how a bad decision gets made twice
   before anybody reads the first one.

## What stops the loop

Write one of these as a line of its own in `STATE.md`:

- **`LOOP COMPLETE`** — every item is done or blocked, with the blocked ones
  listed and why.
- **`LOOP HALT`** — something is wrong that the loop must not work around: the
  gate fails for a reason the iteration did not cause, a decision is needed that
  is not ours, a test that used to pass has started failing, or the same item
  has failed twice.

**Halting is not failure.** An iteration that halts with a clear reason is worth
more than one that invents a way past a problem nobody has looked at.

## What the loop may never do

- **Never weaken the gate to pass it.** Not a lowered lint, not an ignored test,
  not a `#[expect]` added to silence something real. If the gate is wrong, halt
  and say so.
- **Never tick an item it did not finish.** `ROADMAP.md` says a tick means done,
  not written; the same rule applies here, and the loop is exactly where that
  rule would erode first.
- **Never add a verb that runs an arbitrary command** (law 2), or an escape
  hatch that amounts to one.
- **Never claim hardware verification it did not do.** Most of v0.01 ends on a
  certified machine that this loop does not have. Code that is built and unit
  tested is *built and unit tested*, and the item says so.
- **Never touch another repository.** Items that belong to `alo-workplace` are
  marked as such and are not this loop's to do.

## Where things are

| | |
|---|---|
| `docs/autonomy/QUEUE.md` | The work, in order, with what each is blocked on |
| `docs/autonomy/STATE.md` | The journal: one entry per iteration, newest last |
| `ROADMAP.md` | What a person outside the loop reads to know where the product is. Moved every iteration, per step 6 |
| `CLAUDE.md` | The four laws and the gate |
| `docs/decisions/` | Why things are the way they are. Read before proposing otherwise |

## Running it

The supervisor lives in `alo-workplace` and takes a repository path, so this
repository stays Rust and Containerfiles:

```
powershell -ExecutionPolicy Bypass -File C:\dev\Ficina-orders\scripts\run-loop.ps1 -RepoPath "C:\dev\alo-os"
```

Stop it any time. Every finished item was committed and pushed by the iteration
that built it, so nothing is lost by interrupting one.
