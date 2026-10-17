# The undo layout moves where a turn can reach it

**Branch:** `task/mac/the-undo-layout-moves-where-a-turn-can-reach-it`, off `main`
**Date:** 2026-10-17
**Decisions:** none new.
[ADR 0045](../../decisions/0045-what-undoing-rewinds-to.md) already puts the
taking of a bracket in `alo-turn`; this removes the reason it could not be
written.

## The judgement, in one sentence

**This makes an existing promise reachable and removes a present-day
duplication** — `docs/features.md` promises *undo a command*, ADR 0045 says
`alo-turn` takes the bracket, and the layout a bracket must be written in lived
in the one crate `alo-turn` may never link. No promise is added to anybody.

## The present-day fault, which stands on its own

Two crates declared `THE_FOLDER = "/var/lib/alo-undo"`:

- `alo-letting-go`, which owns the layout and **removes** from the folder as root;
- `alo-giving-a-folder`, which **makes** the folder, and declared it a second
  time rather than depend on the remover.

That second copy was not carelessness. `alo-letting-go`'s own
`tests/a_turn_cannot_arrive_at_this_road.rs` forbids anything a turn runs inside
from linking it — *"a second crate reaching for a type, a constant or a function
of this one fails here"* — and ADR 0119 chose the giver for being small, so a
giver that could reach a remover would have been two powers in the place chosen
to hold one. The copy was the price, and the two were kept in step only by a
test reading the published contract.

## And the blocker it was really hiding

`WHERE_A_TURN_RUNS` is `["alo-agentd", "alo-turn"]`. So **`alo-turn` may never
link `alo-letting-go` at all** — which means the crate holding the layout was the
one crate the bracket-taker could not reach. Nothing takes a bracket today, and
this was the first thing standing in the way of anybody writing one.

## What moved

`alo-letting-go`'s `the_folder` module → `crates/alo-kept-undo/src/lib.rs`,
whole, by `git mv`. *Its old path is deliberately not written here: it no longer
exists, and `alo-citing`'s `every_source_this_repository_points_at_exists` is
right to refuse a document that names a file nothing can open.* It was already a leaf: its only imports were `std`, `serde`,
`serde_json` and `thiserror`, and nothing in it touched another module of its old
crate. The names, the two file shapes, the format number, the refusal, and its
six unit tests.

**`alo-letting-go` keeps the path it promised.** `the_folder` is now a module of
re-exports, so every caller and every test that already said
`alo_letting_go::the_folder::…` still does. A re-export is not a second copy.

**`alo-giving-a-folder`'s duplicate is gone** — `pub use alo_kept_undo::THE_FOLDER`.
Its contract-reading test is untouched, because reading the published contract is
the stronger check and is not what changed.

## What was measured, and it is the point of the change

**A turn may now reach the layout, and still may not reach the remover.** Both
directions, by temporarily pointing `alo-turn` at each:

| `alo-turn` depends on | the guard says |
|---|---|
| `alo-kept-undo` | **passes** — both tests ok |
| `alo-letting-go` | **fails** — *"something a turn runs inside ships alo-letting-go for something other than its words"* |

The temporary dependency was then removed: nothing uses it yet, and an unused
dependency is a claim.

**The four btrfs tests still pass** from this tree, as root on a real loop
device — so moving the layout did not change what the remover does:

```
test on_a_real_btrfs_machine_nobody_elses_undo_is_forgotten ... ok
test on_a_real_btrfs_machine_one_act_forgets_everything_that_was_kept ... ok
test on_a_real_btrfs_machine_the_expired_snapshots_go_and_the_rest_stay ... ok
test on_a_real_btrfs_machine_under_the_floor_the_oldest_go_first ... ok
```

## What runs forever: the new crate can do nothing

`crates/alo-kept-undo/tests/this_crate_can_do_nothing.rs`. **This is the guard
the extraction owes**, because the whole of this crate's safety is that it has no
powers — and a power added here travels into a turn's process while
`alo-letting-go`'s own guard still passes, since that guard is about *that*
crate.

- **No door in the source**: ten of them — `fs::`, `File::`, `Command`,
  `std::process`, `TcpStream`, `UnixStream`, `UdpSocket`, `unsafe`, `rustix`,
  `libc` — and none appears.
- **Three dependencies, all text parsers**: `serde`, `serde_json`, `thiserror`,
  asserted as an exact list, with four names that must not appear.
- **The control**: the filter is run over a line containing each forbidden door
  and must find it; the source is asserted over 1,000 bytes and to declare
  `THE_FOLDER`, so a reader returning nothing cannot satisfy the negatives.

**Mutation-proved**, each failing exactly one test and the right one: a
`std::fs::write` added to the crate fails the source guard; `rustix` added to the
manifest fails the dependency guard.

## Two faults found on the way, both in my own earlier work

**The diagram said `0700` and the contract says `0711`.** The header of the moved
file drew the folder as *"the machine's, 0700 root"*. The contract was amended to
`0711 root:root` on 2026-10-16, before any of my work — and **#744 edited that
very line** to move the path and left the mode wrong. Corrected here.

**And the guard caught my own doc comment.** Fixing an intra-doc link broken by
the move, I wrote `alo_letting_go::found` into the new crate — which a turn now
links — and `nothing_an_agents_turn_runs_inside_can_name_this_road` failed with
exactly that sentence. The crate is now named with its hyphen, because the Rust
path form is itself what is forbidden. *This is the second time today a doc
comment of mine tripped that test the same way*, which is why the replacement
text says so in place.

## What this does not do

**It does not take a bracket.** Nothing does, and ADR 0045 still puts it in
`alo-turn`. This removes the reason it could not be written there; writing it is
the next task and needs a snapshot, which `alo-letting-go/src/removing.rs` has
the deleting half of and nothing has the taking half of.

## The gate

`fmt` clean. `clippy --workspace --all-targets --all-features -D warnings` clean.
`doc --workspace --no-deps --all-features` with `RUSTDOCFLAGS=-D warnings` clean.
`cargo test --no-fail-fast` over `alo-kept-undo`, `alo-letting-go`,
`alo-giving-a-folder`, `alo-changing-undo`, `alo-citing`, `alo-image`,
`alo-saying` and `alo-turn`: **0 failures summed across every result line.**
Plus the four `#[ignore]`d btrfs tests as root, above.

**What this did not run**: the rest of the workspace. CI runs the whole suite and
is the required check. This is a head status: these gates passed on this tree,
not on the tree that becomes `main`.

## A process fault of mine, which this change caught

**I gated, then wrote this report, then committed** — so the gate never read the
document it was gating. CI caught it: `alo-citing` reported that line 42 of this
very file named the module's **old** path under `alo-letting-go`, and that no
such file exists.

**And writing that down reintroduced it.** The first version of this section
quoted the failure verbatim, the quote contained the dead path, and the check
failed again — on the paragraph explaining the failure. The check cannot tell a
path **pointed at** from a path **reported as missing**, and in a repository where
every claim is explained in prose beside the thing it is about, those two live in
the same file. The same shape caught two lanes today counting `#[ignore]`
attributes and finding doc comments that mention them.

So the failure is paraphrased here rather than quoted. The check's own words are
the lesson: *"A file renamed leaves every sentence
naming it reading exactly as it did before, so this is either a rename that was
not followed through or a path nobody checked."* It was the second, and the
sentence was in the report **about** the rename.

My own note on this says **measure a document last**, and I had it the wrong way
round: the cheap checks ran against a tree without the report in it, so the local
run was green about a tree nobody was going to land. A gate is a statement about
a tree, and I handed it a different tree from the one I pushed.

