# A unit with no `[Install]` section is started by something

**Branch:** `task/mac/a-unit-with-no-install-section-is-named-by-something`,
stacked on #745 (which is stacked on #744)
**Date:** 2026-10-17
**Decisions:** none new. This is
`the_image_starts_what_it_ships.rs` covering the direction its own name claims.

## The judgement, in one sentence

**This makes an existing promise true**: that file's name is *the image starts
what it ships*, and a whole shape of unit was outside what it asked — so nothing
is added for a person, and a check is made to cover what it already claimed.

## What it missed, and why that is the interesting part

`alo-giving-a-folder@.service` shipped earlier today and **nothing started it**.
The unit was right, the binary was on the image, the capability was right, and
the folder it exists to make was absent on every machine.

It got past both existing tests, and the reason is worth reading twice:

| test | why it passed |
|---|---|
| `every_unit_that_asks_to_be_wanted_is_enabled` | the unit has **no `[Install]` section at all**, so it is not asking to be wanted — skipped |
| `every_unit_starts_a_file_this_image_puts_there` | its `ExecStart` program **is** put there by the recipe — passes |

**A unit asking for nothing and starting a file that exists is a unit both of
these call correct.** It loads on a machine, `systemctl status` shows an entirely
ordinary inactive unit, and nothing will ever start it.

I found it by thinking about it, which is not a mechanism. This is the mechanism.

## The three ways such a unit can be reachable

Measured against the four units in this image that have no `[Install]`, because a
check that knew only one way would fail on the ones that are already correct:

| | unit | how |
|---|---|---|
| another unit names it | `alo-applying-an-update.service` | `alo-going-back.service` references it |
| an activator of the same stem | `alo-asking@.service` | `alo-asking.socket` starts it **by systemd's convention** — the socket's name plus `@` — and names it nowhere |
| a program builds the name | `alo-going-back.service`, `alo-applying-an-update.service`, `alo-giving-a-folder@.service` | `alo-brokerd/src/units.rs`, `alo-sessiond/src/making_a_folder.rs` |

The second row is why this is three roads and not one. A check demanding the name
be written down somewhere would fail on the one unit in this image that is
correct by convention — and then somebody would weaken the check rather than fix
nothing.

## Why the third road is a list and not a search

Because the search is too weak to be a check. Grepping Rust for a unit's stem
finds doc comments, crate names and task reports: `alo-giving-a-folder` appears
as a **crate** name in a test that has nothing to do with starting it. A list
that passes on that is worse than no list.

So `STARTED_BY_A_PROGRAM` names the unit *and the file*, and
`every_unit_said_to_be_started_by_a_program_is_named_in_that_program` reads that
file for the name. The claim is checked rather than taken — the same rule the
not-yet-shipped lists elsewhere are held to, which is what keeps them from
rotting.

A template is matched on its stem, because the program builds the instance —
`format!("{THE_UNIT}@{person}.service")` — and never writes `@.service`.

## What runs forever

Three tests, added to the file whose responsibility this already is rather than a
new one:

- `every_unit_with_no_install_section_is_started_by_something`;
- `every_unit_said_to_be_started_by_a_program_is_named_in_that_program`;
- `nothing_started_by_a_program_has_grown_an_install_section` — the other half,
  because an entry that has stopped being true is one somebody reads as current.
  A unit that gained `WantedBy=` is the recipe's business and the first test in
  this file already checks it, so its line here would then be about nothing.

**Mutation-proved three ways, each failing exactly one test and the right one:**

| mutation | caught by |
|---|---|
| the giving unit dropped from the list | `every_unit_with_no_install_section_is_started_by_something` |
| the list repointed at a file that does not name it | `every_unit_said_to_be_started_by_a_program_is_named_in_that_program` |
| a new unit shipped with no `[Install]` and nothing naming it | `every_unit_with_no_install_section_is_started_by_something` |

**The control** is that the walk asserts it saw at least three such units, so
finding nothing unreachable cannot mean it looked at nothing — and the message
says the image had four on the day, so a later drop to one is visible rather than
quietly satisfying the bound.

## Why it is stacked on #745 rather than off main

Because off `main` it would be a test that **fails #744**. #744 ships the unit
and #745 is what starts it; between the two there is a tree where the unit is
genuinely unreachable, and that is correct rather than something to list as an
exception. Stacking puts this after both, so the check lands on a tree where the
answer is *yes* for a real reason.

That ordering is the whole reason it is not off `main`, and it is written down
here because a reviewer would otherwise reasonably ask.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings` clean.
`doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings` clean.
`cargo test --no-fail-fast -p alo-image -p alo-citing`: **0 failures summed
across every result line.**

**What this did not run**: the rest of the workspace, because the change is one
test file and no library code. CI runs the whole suite and is the required check.
This is a head status: these gates passed on this tree, not on the tree that
becomes `main`.
