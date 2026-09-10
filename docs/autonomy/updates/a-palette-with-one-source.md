# A palette with one source

- Date: 2026-09-10
- Workstream: model selection and configuration (`alo-appearance`)
- Contributor: Claude Code
- Task: The colours come from a source this repository can read
- Status: **the source exists and everything here is held to it.** The web
  client's stylesheet is not, and that is stated rather than implied.

## What this is, and why it was the piece to take

v0.01 was audited before starting. **Every `code` box that exists in the
roadmap's v0.01 band is already ticked** — twelve capabilities, finished on an
ordinary laptop. What remains is eight capabilities with no code half at all,
and seven of them are the compositor: Wayland via Smithay, sign-in, the agent
overlay, launcher and window management, copy and paste, the workspace client on
the shell, and the GPU on first boot.

That band is another worker's live workstream — `crates/alo-shell`, 232 files,
committing six hours before this was written, with a stated next-step chain
running through libinput, DRM and GLES. Two agents in it would not merely
conflict; they would make contradictory design decisions in a subsystem being
built increment by increment.

**This item is the eighth, and it is the one that is not compositor work.** It
was found missing by an audit of the ADRs — a consequence of ADR 0002 with no
line in `docs/features.md` — and nothing had been built for it.

## The problem, stated plainly

alo's palette lived in `alo-workplace/web/src/ds/tokens.css`, 327 lines calling
themselves *the single source of visual truth*, and **a Rust compositor cannot
read CSS**. ADR 0002 makes this shell native; there is no CSS in alo OS and
there will not be.

So the six colours existed as **three hand-maintained copies** — that stylesheet,
`crates/alo-appearance/src/token.rs` as Rust constants, and a table in
`docs/design/figma-brief.md` — with nothing whatsoever holding them together.
They happen to agree today. Nothing would have said so on the day they stopped.

That is how a palette drifts: somebody corrects one hex, and a year later a
designer draws in a colour the machine does not have.

## What was built

**`docs/design/palette.toml` is the source.** TOML because it is already the
format this repository types configuration in, a person can edit it, and both a
Rust crate and a web build can read it without either one owning it. It carries
a `format` number, answered before anything else, on the same rule the record
file and the machine description both state.

**The copies stay; the drift goes.** A compositor cannot parse a file at the
moment it draws a frame, so `Token` keeps its constants — and
`a_palette_with_one_source.rs` reads the source and refuses to let them
disagree. Four tests:

- every colour the shell draws with is the colour the source states;
- the source names **exactly** the six the shell has — so a colour cannot be
  added for the web client and quietly not exist in the operating system, which
  is the same drift running the other way;
- the design brief prints the source's values, because it is the copy a person
  reads *before* drawing anything;
- no two of the six are the same colour, which at this size is far more likely to
  be a copy-paste than a decision.

**Verified by mutation, not by being green.** One hex digit changed in the source
— `#102A43` to `#102A44` — and two tests failed naming both places and both
values: *`navy` is #102A43 in the shell and #102A44 in the palette source*, and
*the design brief does not carry #102A44*. That is the failure a person would
actually get.

## What is deliberately not in the file

**Anything that is not a colour.** Spacing, type scale, radii and motion are not
palette, and a file that grew to hold them would become the same
undifferentiated bucket `tokens.css` had become.

**Light and dark variants.** One palette against two grounds, not two palettes:
`scheme.rs` picks the ground and `contrast.rs` holds a pairing to being readable.

**The five accents a person may choose** (ADR 0010). None of them is in this list
and that is the point — terracotta is the agent's and may never be adopted as
somebody's accent, so the two sets are kept apart rather than filtered apart.

## What this does **not** do

**It does not reach `alo-workplace/web/src/ds/tokens.css`.** That repository is
not beside this checkout, so its stylesheet remains a fourth copy that nothing
checks. Ending CSS's authority over the palette is established *here* — the
source is now in this repository and everything in this repository is held to it
— but the roadmap line also asks for the custom properties to be **generated**
for the web client, and that generation, plus that client adopting it, is not
built.

**So the roadmap line is not finished and no box is ticked.** What exists is the
half that had to come first: there is now one place a colour is changed. The
generator and the workplace-side adoption are the next increment, and the second
of them needs the other repository.

## What I did not do, and why it is worth saying

I did not start on the compositor. Seven of the eight remaining v0.01
capabilities are there, so **v0.01 is now essentially compositor-bound**, and no
amount of work in this lane changes that. That is a scheduling fact the owner
should have rather than one buried in a report: the release's critical path runs
through `alo-shell` and through hardware acceptance, not through here.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible yet; the palette moves but no colour
changes.

**ROADMAP.md** — *The colours come from a source this repository can read* gains
a code half in progress, not a tick: the source exists and this repository is
held to it; generation for the web client remains.

**docs/features.md** — this capability has no line, which is how it went missing.
One is owed.

**docs/autonomy/QUEUE.md** — the palette's source is `docs/design/palette.toml`;
change a colour there and the tests name every copy that has not caught up.

**docs/autonomy/STATE.md** — `alo-appearance` is held to a language-neutral
palette source; `tokens.css` in `alo-workplace` is not yet, and is the remaining
copy.
