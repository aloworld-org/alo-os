# The sweep also looks at what a crate keeps for tmpfiles

**Branch:** `task/mac/the-sweep-also-looks-at-what-a-crate-keeps-for-tmpfiles`,
off `main`, independent of #743/#744/#745
**Date:** 2026-10-17
**Decisions:** none new. This is
[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md)'s packaging being
made checkable, not decided.

## The judgement, in one sentence

**This makes an existing promise true**: the test's own name promises *every unit
a crate keeps reaches the machine*, and it was answering a smaller question than
that — so nothing is added for a person, and a check is made to cover what it
already claimed.

## What was wrong

`crates/alo-image/tests/every_unit_a_crate_keeps_reaches_the_machine.rs` knew
four extensions — `.service`, `.timer`, `.path`, `.socket`. `alo-letting-go`
keeps a **fifth**, `alo-forgetting.conf`, a `tmpfiles.d` file. It was therefore
checked by **nothing at all** — not reported as unshipped, not reported as
anything, which from outside reads as covered.

Audited across every crate: exactly one such file exists, and it is that one.

```
  .service   2  checked
  .path      1  checked
  .timer     1  checked
  .conf      1  NOT CHECKED   alo-letting-go/alo-forgetting.conf
```

## Why it is not a tidy-up

`alo-forgetting.path` contains:

```
[Path]
Unit=alo-forgetting.service
DirectoryNotEmpty=/run/alo/asked-to-forget
```

And `/run/alo/asked-to-forget` is created by **nothing else in this
repository** — only that `.conf`. So shipping the four units without it gives a
path unit that **loads, activates, and can never fire**. Nothing fails, nothing
is logged, and *forget everything you are keeping for me* does nothing for ever.

The desktop lane put the shape better than I did: the sweep's answer was not
wrong about the four extensions, it was **right about a smaller question than its
name claimed.**

## What changed

- **A file's destination is now a function of its extension** —
  `where_it_goes`, with two directories: `image/usr/lib/systemd/system` and
  `image/usr/lib/tmpfiles.d`. The walk, the shipped-after-all check and the
  comparison all use it, so there is one place that knows where a kind goes.
- **`alo-forgetting.conf` is in `NOT_YET_SHIPPED`** with what it is waiting on —
  the same decision as the four units it serves, and a note that **it must not
  ship later than the path unit and must not be forgotten when that one ships**.
- **`every_kind_of_installable_file_a_crate_keeps_is_known`** fails on a kind
  this file has not been taught. It walks eleven extensions a Linux machine
  installs from a directory — `.rules` for udev, `.network`, `.preset`, `.target`
  and the rest — and fails on one `where_it_goes` cannot place, with the fix
  named in the failure. Each of those would have been this fault again, and each
  would have been silent, because **a file nobody looks for cannot be reported
  missing.**

## What was measured, and it runs forever

All seven tests in the file pass, and the two new ones were **mutation-proved**:

| mutation | caught by |
|---|---|
| `alo-forgetting.conf` removed from the list | `every_unit_a_crate_keeps_is_shipped_or_named_as_not` — **so the walk really does see `.conf` files now**, which before this change it could not |
| a crate keeps a `99-a-probe.rules` nobody placed | `every_kind_of_installable_file_a_crate_keeps_is_known` |

Each mutation failed **exactly one** test, and the right one.

Three controls, because a test over a directory tree is the easy kind to make
vacuous:

- `where_each_kind_goes_is_known_and_the_two_places_differ` — both directions
  (`.service` places, `.toml` does not) and the two destinations are asserted
  **different**, because one function answering the same directory for both
  kinds would put a tmpfiles file among the units and look fine.
- `both_places_the_image_installs_to_exist` — each directory exists *and* holds
  a file that certainly ships, so *the directory moved* and *this file is
  absent* cannot be confused. That is the answer this test gives when something
  is genuinely wrong, so the two must not be confusable.
- `every_kind_of_installable_file_a_crate_keeps_is_known` asserts it walked at
  least fifty crates, so finding nothing cannot mean it looked nowhere.

**Nothing here needs a machine or an image build**, which is the point: both
halves are files read as text, so it runs in the ordinary gates rather than at a
release.

## What this deliberately does not do

**It does not ship the four units.** The list says, in as many words:

> Waiting on: whether to ship it now, when nothing yet takes a bracket, or with
> the bracket-taking it exists for.

That is an open decision the repository flagged for a person, and shipping them
because the packaging is now safe would be answering a question that was asked of
somebody else. It is raised with the owner instead.

**One thing worth telling them, from the desktop lane:** a `btrfs` machine boots
on Sunday and will run `alo-letting-go` over a folder that has something in it
for the first time. If it works, *ship now* becomes defensible; if it does not,
the decision answers itself. So the measurement precedes the question.

## Separately: those four units were started on a machine

Not part of this change, and reported because it is what makes the shipping
decision cheap when it comes. All four installed by hand and started:

```
alo-letting-go.service    LoadState=loaded  Result=success  ExecMainStatus=0
alo-forgetting.service    LoadState=loaded  Result=success  ExecMainStatus=0
alo-letting-go.timer      ActiveState=active
alo-forgetting.path       ActiveState=active
```

So shipping them is **packaging rather than repair** — there is no latent
`226/NAMESPACE` or missing path in any of the four, which is the fault that
reached a green branch earlier today in a unit of mine.

**And the remover has still never run over a folder with anything in it.** With
a person's directory and a kept turn present it answered *"this machine keeps
nothing an undo could put back"* and exited 0 — correct, because the development
machine is `ext4` and ADR 0045 term 6 makes that the honest answer, but it
means the measurement returned at the first question. **Unmeasurable in a way
that reads as a pass**, and that is what Sunday's machine is for.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings`
clean. `doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings`
clean. `cargo test -p alo-image -p alo-citing --no-fail-fast`: **0 failures
summed across every result line.**

**What this did not run.** The rest of the workspace's suite, because this change
touches one test file in `alo-image` and no library code — `git diff --stat` is
two files. CI runs the whole suite and is the required check. This is a head
status: these gates passed on this tree, not on the tree that becomes `main`.
