# A pairing is made by two people, and by nothing else

- Date: 2026-09-13
- Workstream: v0.5 the local network, task 2 (`docs/autonomy/v0-5-the-local-network-plan.md`)
- Contributor: Claude Code, by hand — no worker could run
- Status: **the task the rest of the local network rests on**

`docs/features.md` promises, at v0.5: *pairing: mutual, deliberate, enumerated,
revocable in one action, and expiring — grants, across a machine boundary.*
ADR 0003 is what those words mean, and this builds them.

It was written second on purpose. **A pairing nobody can refuse is worse than
no pairing at all**, so this change carries more tests about *not* pairing than
about pairing, and each thing that confers nothing is refused under its own
name — a failure log that says `being_on_the_same_network_confers_nothing` tells
whoever reads it what was promised.

## Mutual, in the type rather than by convention

`Pairing` has no public constructor. The only thing in this workspace that
returns one is `Deliberating::agreed`, and it refuses unless both machines'
people have said yes on their own machine. A `Pairing` in hand is therefore
evidence that they did, rather than a struct somebody could have built.

The confirmation takes a `Side` — `TheOneAsking` or `TheOneAsked` — and not a
`bool`. `confirm(true)` reads as *yes*; what the machine actually has to know is
*which machine the person was standing at*. The difference is the bug being
guarded against, and it has a test of its own:
`one_machine_agreeing_twice_is_not_two_machines_agreeing`. That is the shape a
well-meaning convenience takes — a machine asks, hears nothing, asks again, and
counts its own two attempts as agreement.

## Enumerated means enumerated in a language somebody reads

What a pairing may permit is a closed enum of two arms — *ask questions of the
models on this machine*, *reach the workspace this machine serves* — each with a
sentence in `crates/alo-nearby/src/words.rs`, collected into the machine's one
vocabulary by `alo-saying`. Adding an arm means adding a sentence a translator
reads, which is the cost that keeps the list short.

There is deliberately **no arm for running a verb** on the paired machine, and
there never will be one here. ADR 0003: *pairing lets A ask; it never lets A
act.* Whose grants a verb from elsewhere is evaluated against is task 4, and is
not something this list could grant on another machine's behalf.

Two arms rather than one, with a test saying so. One would be a flag wearing a
list's clothes, and a later change back to one should be a decision somebody
makes rather than one that happens.

## Expiring, with no way to ask for anything else

A proposal states how long it would last, and the duration is part of what both
people are shown rather than a constant somebody has to go and look up. Zero is
refused, and so is anything past thirty days — an upper bound rather than a
policy, so that *expiring* is a property of the type. A person who wants the
machine down the corridor for a year pairs with it again next month, having been
reminded that they are doing it.

Nothing in the crate reads the clock. The moment is passed in, the same way
`alo_capability::Grant` takes it, so what a pairing does at a moment can be
asked about a moment that is not now.

## The four things that confer nothing

| What somebody might expect to be enough | The test |
|---|---|
| the machine is right there on the network | `being_on_the_same_network_confers_nothing` |
| it knows the office wireless password | `sharing_the_wireless_password_confers_nothing` |
| we paired last month | `having_paired_before_confers_nothing` |
| one of us already said yes | `a_pairing_needs_two_people_and_one_of_them_is_not_enough` |

The wireless one is held by a source scan rather than by a value, because what
is being held is *there is no such thing here*: the crate's shipped code is read
for `ssid`, `wifi`, `wireless`, `subnet`, `trusted` and `certificate`. The
failure being guarded against is not a stranger. It is the reasonable-sounding
change — *machines on the office network are already trusted, so skip the second
confirmation* — that ADR 0003 names as the whole vulnerability.

## What this does not claim

**No cryptography, and no claim of any.** What `deliberating.rs` decides is
**who agreed**. Proving that the machine which agreed is the machine that later
asks is a question for the connection between them; that connection is not
built, and nothing in this crate implies it exists. The file says so where a
reader would otherwise assume.

**Revocation is measured across one list**, which is the whole of what exists
today. When a question can actually travel to another machine (task 3), *takes
effect immediately* will need measuring with one in flight, and that belongs to
that task rather than being claimed here.

## Verified

Windows, this development machine:

| Target | Result |
|---|---|
| `alo-nearby` unit tests | 54 passed, 0 failed |
| `a_pairing_is_made_by_two_people_and_by_nothing_else` | 9 passed, 0 failed |
| `the_local_network_says_no_more_than_a_machine_exists` | 5 passed, 0 failed |
| `alo-saying`, `alo-collected` | green, with this crate's words collected |
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -D warnings` | clean |
| the rest of the workspace | green but for `alo-recounting` |

`alo-recounting`'s failures are the Windows-only platform split in
`docs/quirks.md`; nothing here touches that crate.

The crate's words are registered in **three** places in
`crates/alo-saying/src/collecting.rs` — `EVERY_LIST`, `ONE_STRING_EACH`, and the
per-crate count — which is by design and is the third time this workspace has
caught a list added to two of them.

**CHANGELOG.md** — nothing user-visible: there is no surface that shows a
pairing yet. **ROADMAP.md** — *machines find each other on a local network, with
pairing* is now built on both halves; the box stays unticked until something on
a machine uses it.
