# The Dock fixed to the bottom edge

What [ADR
0076](../../decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)
decided, carried into the code: 34 files, four types and a drawing surface
removed, and one setting that turned out to have been half-built for two days.

The record landed on its own first (#263). This is the change it authorises, and
it carries the `ROADMAP.md` untick with it, which is the second of the two orders
that record permits — *either the owner unticks first and the removal follows, or
both go in one change the owner approves*. The owner approved this one. The order
that was not allowed is the code going while the tick stays, because then the
roadmap claims something that is not there.

## What came out

`alo-dock` lost `Edge`, `Along`, `StatusArea` and `End`, the four edge words and
`dock.labels.beside`, `Labels::Beside`, `Room::a_dock_with_names_beside`,
`Room::a_name_beside_an_icon_at`, `measures::LABEL_EMS`, `Screen`'s two
orientation methods, and every `Dock` and `Changes` method about an edge or a
display singled out. `alo-shell` lost `status_items_raster.rs` and the status
area it drew. `alo-displays::Wearing` lost the edge it reported per screen, and
with it the `Dock` it was handed.

Three signatures lost an argument that was only ever there to place the status
area: `Layout::of`, `Dock::layout_on` and `Wearing::of` no longer take the
reading direction or the dock. ADR 0076 did not enumerate those; they follow from
the status area leaving, and they are named here so the next person reads a
consequence rather than finds one.

## Three things the removal found

### A setting that was written down and could not be read back

`Hiding` — *whether the dock gives way when a window needs the room* — arrived on
2026-09-27. The choice existed, `Changes` wrote it to `dock.toml`, and the two
strings for the rows a person picks between were declared in `words.rs`.

**Nothing resolved it.** There was no `Dock::hiding` to ask a person's choice
against what the release ships, so a value written to the file could not be read
back into an answer; `Hiding` had no `said`, so neither string could reach a
screen in any language, including English; and the settings panel offered four
edges and no rows for this. It was a setting that existed everywhere except where
somebody would use it.

It was found because taking the edge out left `Shipped` holding nothing. A
release default has to be a default *of* something, and the only candidate was
the one setting left — which turned out not to be wired. The middle is now there
and the panel draws the two rows.

### A test that was measuring one list against itself

`keeping.rs` held `KEYS`, the keys `dock.toml` may have, and a test asserting it
equalled the keys a change writes. Those two had to stop being the same list: a
`dock.toml` that names an edge has to **read**, because every release before this
one wrote that key for anybody who moved their dock, and a person's file is not
rewritten behind them. An unrecognised key is refused whole, so `edge` and
`displays` stay recognised with nothing behind them, and the edge is ignored.

The equality is now *the keys a change writes are the live half of the list*,
with a second test that the two halves are disjoint — because a key on both lists
would make the first test compare a list against itself minus itself, which is
the shape of the self-agreeing check #260 was about.

ADR 0076 says this is done *via the existing `dock.kept.unknown-key` sentence*.
**That parenthetical has it backwards** and is the one thing in the record this
change contradicts: the unknown-key sentence is what must *not* fire. Read-and-
ignore is the absence of a refusal, not a use of one. The record's decision is
unaffected; the mechanism named in it is wrong, and this paragraph is the
correction rather than a quiet fix.

### A share that might have come loose, and did not

`A_DOCK_MAY_TAKE_ONE_PART_IN` is one part in six, and it was held tight by two
cases: a dock with names *under* an icon and one with names *beside* it. Removing
the vertical orientation removes half of that justification, and the honest worry
was that the surviving case would leave the number loose — that one part in seven
would now also fit, making six a number nobody had a reason for any more.

It does not. On the smallest screen's height, one part in seven loses the names at
the 200% EN 301 549 requires them to survive. The test asserting that is kept and
now says which way it is fixed with one case instead of two.

## What this costs, said plainly

**Four things that were drawn are not drawn now.** The clock, the battery, the
network and the volume were laid out inside the dock's status area by
`status_items_raster.rs`, and that file is deleted.

This is what ADR 0076 directs, not a side-effect: the promise keeps, and the
record takes away its *location* and hands *where does it go* to the shell's own
plan. Its instruction for this change was that the entry in
`evidence-a-person-can-work-on-it-all-day.md` must say the promise is **owed a location before an
implementation**, and it does. Drawing them somewhere else would have been this
lane picking the location the record reserves.

`status_items.rs` is kept. What a clock *says* was never the half that was about
where it goes, and `alo-desktop` still reads it.

**The egress indicator did not go with them**, and this is the part worth being
exact about, because it is a law-1 surface. It sits where it sat: the far end of
the dock, above the band, growing upwards. What changed is where the answer comes
from — the far end of a row was always a question about the way the row is read,
so `egress_status_place.rs` asks the reading direction rather than a type the
Dock no longer has. The test that asserts its corner keeps the exact pixel numbers
it had before the record, deliberately: a test rewritten alongside the code cannot
say the indicator did not move. The same holds for what tells a person their
camera or microphone is in use.

## Two near-misses, and what caught them

**`alo-access::Surface::StatusArea` survives.** A grep for `StatusArea` finds it,
and deleting it would have taken the accessibility surface that announces
`EgressStatusFrame` and *something is leaving* — a law-1 announcement, removed
because it shared a name with the thing being removed. It is a different type
about a different subject. Reading the first line of every file before deleting
it is what separated them, which is the mechanical check #252 concluded was the
only thing that would have caught the `lock_texture.rs` over-removal.

**`Stacked::Downwards` was deleted rather than left.** With one edge, rows only
ever grow upwards. Three rasters matched on both arms, and the compiler named all
three once the variant was gone. Had the variant been kept *in case*, those arms
would have stayed as branches nothing reaches — which is exactly the *offered but
discouraged* cost ADR 0076 refuses.

## Evidence

- `crates/alo-dock/tests/dock_kept_in_its_own_file.rs` — a `dock.toml` written
  by an earlier release reads, the edge is ignored, the dock is at the bottom,
  the live key beside it survives, the dead key leaves on the next write, and a
  misspelled key is still refused whole.
- `crates/alo-dock/src/layout.rs` — the share is still the tightest the standard
  allows with one orientation.
- `crates/alo-dock/src/hiding.rs`, `crates/alo-dock/src/dock.rs` — the choice
  reaches an answer, and both rows say something of their own.
- `crates/alo-shell/src/egress_status_place.rs` — the indicator's corner, in the
  numbers it had before the record.
- `crates/alo-dock/tests/the_contract_describes_this_file.rs` — the contract
  lists every key a change writes and no other, and every example in it reads or
  is refused as it says.

## What is not shown

**Nothing has been looked at.** There is no screen in any of this. The dock is
arithmetic and the drawing is rasters compared against numbers; whether a person
looking at a machine sees a dock along the bottom is owed to hardware, along with
everything else in `evidence-a-person-can-work-on-it-all-day.md` that says the same.

The spec this record serves describes a great deal that is not built: what a
click does, favourites and open applications, an overflow area, indicators
distinguishable without colour, the right panel's minimised windows. This change
removes what should not be there. Building what should is the next piece of work,
and it is not claimed here.
