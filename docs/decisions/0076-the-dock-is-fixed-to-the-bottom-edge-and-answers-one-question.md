# ADR 0076 — the Dock is fixed to the bottom edge, and answers one question

**Status:** accepted, 2026-09-29, by the owner. **Amended 2026-09-30** — the
decision stands; what it cost is now written down, and the reason it gave for
taking it is retired.

## What this cost, which this record did not say

**This record withdrew a choice a person had been promised, and did not say so.**
`docs/features.md` carried *the person decides where it goes — bottom, left,
right or top, chosen in Settings*. That is gone, and a person who would have put
their dock down the side of a wide screen — which both of the systems they are
coming from allow — no longer can.

The fifth law says *a change that takes a choice away from the person is a bug,
whatever the reason given for it*. **It landed on 2026-09-22, seven days before
this record, which cites it zero times.** Nobody noticed until three lanes read
the laws against their own crates on 2026-09-30.

**The reason given here is retired, and it is the part that generalises.** This
record argued that nobody had chosen an edge because nothing has booted the
image, so the cost fell on a contract rather than on a person. But before a
machine boots, *no* choice has been exercised — not one accessibility setting,
not the dock's own hiding setting. That reason does not argue about docks; it
argues that the fifth law does not bind until somebody boots, which is the whole
period in which this product is being decided. `CLAUDE.md` now says so, and the
argument is not available to the next record that reaches for it.

**What is decided, and what is not.** The Dock is fixed to the bottom edge, and
that stands: the reasoning from *what is the Dock for* is sound, the design file
draws it there, and the shell is built on it. **What is not decided is that a
person may never choose an edge again.** This record withdrew the promise
outright where it should have deferred it. That distinction is not pedantic — a
withdrawn promise is gone and a deferred one is owed, and only one of the two
survives the fifth law. Whether the choice returns, and at which tier, is the
owner's and belongs in `docs/features.md`, which is the only place scope is
decided.

**Why this is an amendment and not a reversal.** Relitigating a settled decision
without new facts is forbidden here. A law nobody applied is a new fact, and it
reaches this record's *reasoning* and its *silence* — not its conclusion, which
nothing here disturbs.

## What forced it

The canvas landed, and the Dock was designed before it. A dock that a person
could move to any edge made sense when the screen was a flat arrangement of
windows; it does not survive the question *what is the Dock for* once there is a
plane that pans and zooms underneath it.

The owner answered that question, and the answer decides the rest:

> The Dock answers *where do I go?* The canvas answers *where is my work?* The
> right panel answers *what did I put aside?* Keeping those jobs distinct makes
> the interface learnable even with many windows open.

## The decision

**The Dock is fixed to the bottom edge of the screen, above the canvas.** It does
not move when the canvas moves and it is not a second workspace.

**This record's own restatement, which the rest of it reasons from:** the Dock
holds nothing that is not an application a person can open or focus. That
sentence is not the owner's. It is this record's reading of the three questions
above, and the removals below follow from *it* rather than from them. A reader
who disagrees with the restatement should disagree with those removals, which is
why it is separated out rather than folded into the quotation.

## What is withdrawn, and who may withdraw it

### The **[v0.01]** promise, in `docs/features.md`

> **The dock, and the person decides where it goes** — bottom, left, right or
> top, chosen in Settings. It works in both orientations rather than being a
> horizontal bar someone turned sideways: the status area reflows, and labels
> give way to icons where the short edge demands it.

Withdrawn, with **[v0.5]** *Per display, so the dock can sit along the bottom of
the laptop and down the side of the external screen*.

### The **[v1]** promise, in `ROADMAP.md:758` — **and its code box is ticked**

*The dock on any edge* is **not** in `docs/features.md`; searching there finds
nothing. It is `ROADMAP.md:758`, and at `:760` it carries `- [x] **The code.**`

**A tick is the owner's signature on the release's account of itself, and this
record does not remove it.** What this record does is say that the tick is now a
claim about code that is being deleted, so it has to come off, and that taking it
off is the owner's act and not a lane's.

**The executing lane's action is to ask the owner**, and neither to untick it nor
to leave it. Saying only *not this and not that* leaves a lane instructed to do
nothing and not told to raise it.

**And the order matters, because nothing in the repository disagrees with a false
tick.** `alo-reconciling`'s `orphaned_boxes` counts halves — whether a promise has
a code box and a machine box — and never asks whether a ticked one is true.
`counts_that_drifted` catches one numeric case and not this. `alo-dock` survives
this change, so no test fails either. Checked, not assumed.

So between a code removal and an untick, `ROADMAP.md:760` reads `- [x] **The
code.**` for two orientations, four edge words and a four-edge EN 301 549
threshold that no longer exist — **and it reads green.** That is a claim that stays
true-looking after it stops being true, which is the fault this repository spent a
night removing from its gates, appearing in its roadmap instead.

**Therefore: either the owner unticks first and the code removal follows, or both
go in one change the owner approves.** What must not happen is *remove the code
and flag the tick*, which is a window of unknown length in which the release's
account of itself is wrong and nothing contradicts it.

Its text is load-bearing beyond the Dock: it claims EN 301 549's *200% on the
smallest screen alo OS lays out for, **on all four edges***. Conformance is being
re-read in another lane right now, so that sentence must not silently become a
claim about a threshold on an edge that no longer exists.

## This overrides a standing rule, and says so

`CLAUDE.md` is explicit:

> **Contracts outlive code.** The agent verbs, the application-adapter SDK, D-Bus
> interfaces, **config keys**, the image format and the update channel are public
> surfaces. Third parties build adapters against ours; they change additively,
> and **a break requires versioning and deprecation.**

**Two surfaces are involved and they fail the rule differently. Only one needs
an override.**

`dock.toml`'s `edge` is a **config key, and it is handled gracefully rather than
broken.** The file still reads, the person keeps `hiding`, and the machine says
which key it skipped by name. That is what a config key's deprecation looks like
when the key's meaning is gone: not a version bump for a file with two keys, but a
reader that does not fail and does say what it ignored. **No override is claimed
for it.**

`alo_dock::Edge`, `Along` and `Dock::edge_on` are a **Rust API that vanishes with
no deprecation period.** That is a clean break, the rule forbids it, and **this is
the one place an override is claimed.**

It rests on one fact, which is the repository's own and not an argument:
**nothing has booted the image.** `docs/autonomy/v0-5-evidence.md` says so in
three places. The rule's own sentence is *third parties build adapters against
ours* — there is no installed machine and no third-party adapter, so the interest
the rule protects is not engaged.

**The strongest objection, answered rather than left out:** *a rule followed only
when it costs nothing is not a rule.* Somebody may say the purpose of *contracts
outlive code* is to build the habit before the first customer, precisely so the
first break is not the one that teaches you — and that an override taken the one
time it is free is convenience wearing a rule's clothes.

The answer is that deprecating an enum nobody outside this repository can name
teaches nothing and costs for ever. A deprecation period is a promise to somebody;
with nobody on the other side it is a comment plus the *offered but discouraged*
cost this record rejects two sections down. The habit worth building is not
*always deprecate* but *never break a surface somebody holds* — and the way to keep
that habit auditable is the paragraph below, not a ceremony performed at nobody.

**The fact expires.** The first installed machine makes this override unavailable,
and any later removal on this surface needs the versioning and deprecation the
rule asks for. A future reader should read this as *the rule was beaten, for an
API only, by a fact that was true in September 2026* — not as a precedent.

## What this costs, stated smaller than it first read

An earlier draft of this record said *a person who had the dock on the left finds
it at the bottom*. **No such person exists** and the sentence should not have
implied one. The cost to people is zero; the cost is to a contract, and the
paragraph above is where that is paid.

## The status area is no longer the Dock's

`docs/features.md:99` promises, at **[v0.5]**: *Status area: clock, battery,
network, volume, brightness — at the far end of the dock, wherever the dock is.*

A clock is not something a person opens or brings into focus, so by **this
record's restatement** — not by the owner's words, which never mention a status
area — it is not the Dock's.

**The promise is not withdrawn.** A machine still needs those things. What this
record does is take away their location, and a promise with no location and no
crate is the orphan `alo-reconciling` exists to catch.

**So it is not left floating:** the question *where does the status area go* is
handed to the shell's own plan,
`docs/autonomy/the-smallest-canvas-worth-showing.md`, and the entry for this
promise in `docs/autonomy/v0-5-evidence.md` must say that it is owed a location
before it is owed an implementation.

**Who does that**, since an instruction addressed to nobody is not an
instruction: the change that removes the code carries the evidence entry with it,
because a promise losing its location and its entry keeping the old one is the
same dangling-claim fault as the tick. If the lane executing this does not hold
`docs/autonomy/v0-5-evidence.md`, it says so in the pull request and the owner
assigns it there — **it does not land the removal with the entry stale.**

## The Dock shows no minimised windows

The right panel holds previews of minimised windows. In the owner's words: *we
should not put duplicate minimized-window thumbnails in the Dock.* A minimised
application keeps its open indicator on its icon; the window is represented once,
in the panel.

The Dock may show a small activity indication where alo is working, and **must
not become an agent dashboard**.

## Two other promises want the bottom edge

Naming the collision rather than leaving it to be found:

- `docs/features.md:437`, **[v1]**: *No dock by default — the alo key, **the
  bottom edge** or a swipe reveals the alo Edge… a person may pin a dock.*
- `docs/features.md:453`, **[v1.1]**: *The time ribbon — a ribbon at the **bottom
  edge**.*

Three things now reach for one edge. **This record does not resolve that** — it
fixes the Dock there and says the other two exist, so whoever builds the second
one finds a decision rather than a surprise.

## Why a single edge rather than a default

A default would keep `Edge`, `Along`, the per-display exception and both
orientations in the code, and keep every one of them as a thing the compositor
draws, a settings panel offers and a test covers — for a choice the design says
nobody should make. **The cost of *offered but discouraged* is paid on every
screen, at every scale, forever.** A decision that removes the code is the only
kind that removes the cost.

Anybody restoring a second edge is undoing a decision rather than filling a gap,
and this record is the reason.

## What comes out

- `alo-dock`: `Edge`, `Along`, `StatusArea`, `End`; `Dock::edge`, `set_edge`,
  `edge_on`, `set_edge_on`, `put_display_back`; `Changes::set_edge`, `edge`,
  `set_edge_on`, `edge_on`, `displays`, `forget_display`; `Setting::Edge`.
- The four edge words, and whatever only the vertical orientation said.
- `docs/features.md`'s **[v0.01]** promise and its **[v0.5]** descendant;
  `ROADMAP.md:758`'s **[v1]** promise, whose tick is the owner's to remove.

## What a settings file that names an edge does

**It reads, and the edge is ignored.** Not refused.

This is the opposite of what #253 describes for a wallpaper, and the difference is
structural rather than a matter of policy — **the earlier draft gave a reason that
would have justified either answer, which is the wrong kind of reason.**

In `dock.toml`, `edge` is a **key**. A reader that does not know a key can skip
it, and that is the whole difference.

> **Corrected while carrying this record out.** This paragraph originally ended
> *and `dock.kept.unknown-key` is the sentence for exactly that*, which has it
> backwards. That sentence is a **refusal**: a key `alo-dock` does not recognise
> makes the whole file fail to read. It is the right answer for a typo and the
> wrong one for a key this project itself wrote last release. Read-and-ignore is
> the **absence** of that refusal, not a use of it — so `edge` and `displays`
> stay on `keeping.rs`'s list of keys the file may have, with no field behind
> them, and serde skips what no field claims. The decision in this section is
> unchanged; only the mechanism named in it was wrong.

In `appearance.toml`, `background` is a key whose **value is a serialised type
being removed**. Deserialisation fails on the shape of the file, not on the
presence of a name, so there is nothing to skip and nothing short of a migration
will read it — which is why that one is `expand → migrate → contract` and this one
is a line.

The consequence for a person is the same in spirit and worth saying: a file
holding a setting alo OS no longer has should not cost them the settings it still
does. `dock.toml`'s other key is `hiding`, and refusing the file whole over `edge`
would throw that away.

## What stays

`Hiding` — whether the Dock gives way when a window needs the room — is untouched
and becomes the only thing `dock.toml` holds. Favourites, open applications, a
stable icon size and an overflow area are the Dock's own.

## Rejected

**Keeping the edges and defaulting to the bottom.** Costed above.

**Deprecating `Edge` in place**, which is what `CLAUDE.md` asks for. Rejected for
the Rust API on the *nothing has booted* fact, with the *a rule followed only when
it costs nothing* objection answered in the override section rather than ignored.
**Not rejected for the config key**, which is handled gracefully instead — see
that section: those two are not the same decision and an earlier draft of this
record treated them as one.

**Refusing a settings file that names an edge.** It makes the break loudest at the
moment a person can do least about it, and costs them a setting that is still
valid.
