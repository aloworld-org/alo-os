# The committed Figma snapshot

**What this is.** The node tree of the design file, exported and committed, so that a
design change shows up in this repository as a **diff** instead of as nothing. Before this,
the only record of the file was prose written by hand after reading it — and on 2026-10-04
that prose was 55 top-level frames behind without anything saying so.

**Approved by the owner on 2026-10-04**, with the size explicitly accepted: *an untraceable
design change is the larger problem.*

## The Settings window and the alo window, 2026-10-31

**`544-30910.xml` is the page the owner approved on 2026-10-31**, *Proposed ·
Settings and the alo window · 2026-10-31*: fifteen Settings frames, five alo
window frames and a parts kit, 2,905 lines. Read with `get_metadata` on the
page's own id and written by this file's procedure — entry 0 only, LF, no
trailing whitespace, the refusal list checked, the root closed — by a script
rather than by hand, because the response was 243 thousand characters and came
back as a file.

**Geometry and layer names only**, as every file here. The appearance — fills,
strokes, the dark values, type — is in
**[`../the-settings-window.md`](../the-settings-window.md)**, from
`get_design_context` on each frame the same day; the artwork is SVG in
`crates/alo-shell/icons/settings/`, with the node each came from. **Instances
are not expanded**: a button appears as one `<instance>` with its width and no
label, which is why the button widths in that page are recorded as the frames
set them rather than measured off their labels.

## The sign-in and unlock screen, 2026-10-12

**`176-10686.xml` is authoritative for the sign-in and unlock screen**, and
supersedes `70-28.xml` for it. The page export holds the screen frames with
their authentication instance as a stub; this holds the component expanded,
which is where every measurement an implementation needs lives.

**One file for seven links.** The owner gave seven nodes — the screen, the
component, and five states. The five states are the same frame with the same
four children, differing only inside one instance, so capturing each would
record the same frame five times and hide the one thing that differs. The
component `364:25567` is captured instead, and the states' ids are recorded in
the file so a later capture reads the same nodes.

**What the tree carries**: the authentication group is 400 × 268 at (520, 266)
on a 1440 × 960 screen; the monogram 64 × 64 at (168, 0) with its initial at
(23, 14); the account line at (122, 76); the entry 304 × 56 at (48, 144) with
the submit target 44 × 44 at (253, 6) **inside it**; *Other ways to sign in* at
(80, 224). The essential actions are a 1360 × 48 row at (40, 884) — Accessibility
at its left, Switch person and a 48 × 48 power target at its right — and *Back
to lock* is 48 × 48 at (32, 28).

**Three things are hidden in the shipped component**, which is a measurement
rather than an omission: `176:10999` *Instruction*, `176:11003` *Unlock canvas*
and `360:25700` *Alternative authentication methods*. The second is the
separate full-width button an earlier design had, and the owner's direction of
2026-10-12 is explicit that it is not restored.

**And what the tree does not carry.** The error state's red entry stroke and
its red feedback line are a colour change on the same geometry, and a stroke on
a frame is not a child layer — the blind spot this file already records for the
window edge's focus ring. Both were read from the render instead.

**Manrope, stated rather than inferred.** `Heading/Large`, `Display/Medium`,
`Body/Medium` and `Label/Medium` all resolve to Manrope on this node, and the
owner's direction names it. `the-typeface-the-design-specifies-is-not-the-one-that-ships`
is the open question this closes for the design side: the shell compiles Inter,
and that is now a difference somebody must resolve rather than a question
nobody had answered.

## The sign-in wallpaper, as artwork rather than as boxes, 2026-10-12

**`176-10686-wallpaper.svg` is the folded-light wallpaper itself**, exported
from `364:25552` on 2026-10-12. It is 2,874 bytes: a `#FAF7F2` ground, five
paths and five linear gradients.

**It is here because bounding boxes are not curves.** `get_metadata` gives each
vector's box — `364:25553` is 584 × 1240 at (1016, -180) and so on — and a box
says where a curve passes through, never what shape it is. Drawing from the
boxes would be inventing an illustration and calling it the design, which is
the fault this whole folder exists to prevent.

**The ground is `#FAF7F2`**, which the capture's variables do not carry: it is a
fill on the frame rather than a bound token. Worth stating, because the screen
that ships today draws white, and warm off-white against white is the kind of
difference a person feels and cannot name.

**Not yet drawn.** Five cubic paths with gradients are not expressible as the
flat rectangles the shell paints, and the rounded-rectangle road
`crate::dock_surface` uses — a signed distance field, antialiased by one
`0.5 - d` rule — answers corners and not arbitrary curves. Rasterising this
needs either a path rasteriser as a dependency or a render baked at build time,
and that is a decision rather than a line of code. The screen is being built
without it, on the right ground colour, and this file is what it will be drawn
from when that decision is made.

## The Ask alo task window, 2026-10-12

**`417-27657.xml` is the section *Ask alo · Task conversation*** — eight state
frames, an interaction-contract frame, and 118 nodes. Read live from the
canonical file on 2026-10-12, at the owner's instruction, which named the nine
nodes and the words *refresh these nodes in the committed snapshot before
implementation*. That is the refresh rule being overridden by whoever set it,
as on 2026-10-07, rather than a lane reaching for the file key.

**Transcribed, not written by a script.** Like `0-1.xml`, the response was
small enough to return inline and so did not pass through step 3 of *How to
reproduce it*. It is what the tool returned.

**This file is geometry and layer names only, and that is not enough to build
from.** `get_metadata` carries no fill, stroke, typography, token name or
component description, and the task window's entire appearance is in those. The
measured appearance is in **`docs/design/the-ask-alo-task-window.md`**, from a
second call — `get_design_context`, same section, same day — and that page also
names what neither call returned.

**Two further things this tree does not show, recorded here so the gap is not
rediscovered:**

- **The seven state interiors are not in it.** A section read expands only the
  selected instance, so seven of the eight frames appear as bare
  1040 × 800 `<instance/>` boxes. Their interiors came from the component set
  `417:28422`, which is on another root and is **not** part of this file.
- **There is no `Stopping` variant anywhere in the design.** Frame
  `418:28135` is named *Ask alo / 08 · Stopping* and the contract frame
  specifies the state in detail, but the component set has seven variants and
  `Stopping` is not among them. A later capture that adds one is a design
  change, not a correction to this export.


## The compact edge and the window menu, 2026-10-11

**`414-27564.xml` is authoritative for the window edge's sizing, its
breakpoints and its window menu, and supersedes `398-26305.xml` for those.**
`398-26305.xml` remains the record of the normal edge's own anatomy — the
strip, the grip, the title placement and the control artwork — which the new
section does not redraw.

**One file covers both nodes the owner gave**, because
`414:27668` *Compact window edge / Design review* is a child of the section
`414:27564`. Captured in one read rather than two, so they cannot disagree.

Read from the live file node by node on 2026-10-11, not derived from any
export, and freshness is not claimed from node ids — the direction of
2026-10-09 still holds. What evidences it instead: the section contains the
`State=Full controls` master at **196** wide and four compact masters at
**148**, and those two numbers are the breakpoints this capture exists to
record. A capture made before the compact design was drawn could not hold
them, and `70-28.xml` does not contain the section at all.

**The build contract is sixteen layer names**, not prose written about the
design. `get_metadata` returns text content only as layer names, so each
sentence of the contract is recorded exactly as the export gives it —
including the places where a number runs into the preceding word
(`width148`, `Menu264px wide`). Those are the export, not transcription
errors, and they are left alone so a diff against a later capture is
meaningful.

**Three subtrees were expanded by separate calls** and are inlined in place of
the `<symbol/>` stubs the section-level call returned: `414:27580`
(*State=Reveal*), `414:27623` (*State=Full controls*) and `414:27639`
(*Window=Normal*, the menu). The section-level response gives a symbol's box
and not its contents, so a capture without these would record that the
component exists and nothing about where its targets are.

**The colours are the five this project already has**, which is the finding
worth stating: `bg/surface`, `border/default`, `text/primary`,
`text/muted` and `bg/cool`, resolving light from `414:27638` and dark from
`414:27787`. The design's own note says so — *existing semantic colours; same
geometry* — so the compact edge and the menu need no new role in
`alo-appearance`. Read per node because the connector is mode-incoherent
within one response; both sets are in `variables.toml`.

**Two of the four were read on 2026-10-12 and are expanded above**:
`414:27594` (*State=Menu open*) and `414:27609` (*State=Keyboard focus*).
`414:27566` (*State=Rest*) and `414:27653` (*Window=Maximised*) are still
boxes with no contents; rest and reveal are stated by the contract to share
geometry.

**Menu open adds exactly one layer and exactly one colour.** A
`rounded-rectangle` 32 × 28 at `(6, 14)` inside the *Window menu / 44×44*
frame, and `bg/cool` over the keyboard-focus set. That is the same 32 × 28
`A_HIGHLIGHT_IS` the painter already fills on hover, so **an open menu draws
its trigger as hovered** and the painter needs no new shape and no new colour
for it.

**The focus ring is a stroke on a frame, and `get_metadata` cannot see one.**
That is why this section used to say the ring's measurements *are not in this
capture*: the statement was true and it hid the reason. `414:27609`'s metadata
comes back **byte-for-byte identical to `State=Reveal`** — no extra child, no
extra variable — because a frame's stroke is not a child layer. Anything else
in this file read for a focus state has the same blind spot.

Measured off the rendered PNG instead, by differencing the two states' renders
and keeping what is dark in one and not the other (309 pixels):

| | |
|---|---|
| the ring | `x 52..95, y 0..43` — **44 × 44**, the *Window menu / 44×44* frame's own bounds |
| thickness | **2 units** |
| colour | `text/primary`, the only dark ink bound on the node |
| corner | reaches full width by the 5th row; `radius/sm` = 8 is the only radius bound — **inferred, not measured** |

The thickness was checked against antialiasing, which would have faked it: at
row 21 the pixels read `#ffffff #102a43 #102a43 #ffffff`, the ink exactly with
no partial values either side.

**The ring is the 44 × 44 interaction envelope and not the 32 × 28 visible
strip**, so it stands 12 units above the strip, which begins at `y = 12`. The
obvious implementation — ring the pill a person can see — is wrong, and nothing
in the file says so except the geometry. `window_edge_paint` had done exactly
that, and the change that carries this capture fixes it.

**Dark needs no separate read for either state.** The ring is `text/primary`
and the highlight `bg/cool`, and `variables.toml` already holds both in dark
(`#FFFCF8` and `#183B56`).
## Stale for a second component, 2026-10-11

**`70-28.xml` is stale for `More apps / Popover` and `406-26881.xml` is
authoritative for it.** Same shape as the 2026-10-09 entry below, and found the
same way: searched by name against the 7,344 frames the page export holds,
`More apps / Popover` returns nothing. The only match for *More apps* is
`More apps / opens inward`, which is the **list** design this component
replaces.

That replacement is the part a reader must not miss. They are not two versions
of one design. The owner's ruling of 2026-10-11 —
[ADR 0103](../decisions/0103-the-dock-more-apps-and-the-minimised-panel-are-three-connected-surfaces.md)
— changed what the surface *is*, from the applications that overflowed to every
pinned and open application, and `406:26881` is the design for the new one. So
`AN_OVERFLOW_PANEL_IS_WIDE = 248`, measured off the old frame, is superseded by
the 448 in the new one.

**Freshness is not claimed from node ids**, by the owner's standing direction.
This capture evidences itself two ways instead. It was read from the live file
on 2026-10-11, node by node, after the owner gave the two links. And it holds
six states — `Default`, `Search`, `No results`, `Keyboard`, `Scroll` and
`Windows` — of which `Windows` is the window chooser that ADR 0103 created on
that day. A capture made before the ruling could not hold it.

The heights in it are **caps and not measurements**: the owner stated 536 for
the whole popover and 336 for the list viewport separately from the file, and
every state is drawn at the cap. The file itself cannot show that distinction,
which is why it is written at the top of `406-26881.xml`.

## Stale for one component, 2026-10-09

**`70-28.xml` is stale for `Canvas / External window edge` and
`398-26305.xml` is authoritative for it.** The page export is still the record
for everything else in the file.

The component is not in the page export at all - searched for by name, nothing,
against the 7,344 frames that export does hold. It is being implemented
against, so it is captured in full as its own file rather than left unrecorded.

**Freshness is not claimed from node ids, and must not be.** The owner's
direction of 2026-10-09: *node IDs alone do not establish freshness: existing
398:\* nodes were edited, and new visual elements were added under 402:\*.*
So the capture evidences itself two other ways - it was read from the live file
node by node on 2026-10-09 rather than derived from any export, and it contains
`402:26312`, `402:26313` and `402:26318`, each named *32px visible edge / 44px
interaction*, which are the compact-size update itself. A capture made before
that update could not hold them.

The tokens its variants resolve are in `variables.toml`. Two of them -
`bg/cool` and `status/danger` - carried *the owner stated this value* because
no light node here had shown them; `398:26260` resolved both directly, so both
are now measured and name that node in `also_seen_in`.

A full-page re-export remains owed and does not block this component, by the
owner's direction of the same day.

## Refreshed 2026-10-07, and the staleness is cleared

**The re-export this file said was owed has landed.** `70-28.xml` now carries the design as
of 2026-10-07, including the four first-start section roots and the corrected AI-choice
flow. The ids that were missing — `370:30053`, `370:30057`, `371:30239`, `371:30245`,
`371:30247`, `371:30252` — are in it, and `183:10679` is *12 · Choose how alo helps* here as
well as in the file.

**What the refresh moved:**

| | before, 2026-10-04 | after, 2026-10-07 |
|---|---|---|
| bytes | 2,247,251 | **2,350,573** |
| lines | 25,217 | **26,421** |
| nodes | 20,100 | **21,072** |
| `<text>` nodes | 5,897 | **5,965** |

**Measured off the refreshed export, not read off the screens:** 77 frames under the four
section roots, **69 carrying the identity label** and 8 without it, the 8 being the
canvas states in section `372:30292`. The scripts that computed those figures are the
record of how, and `../the-first-start.md` carries what they mean.

**Written by a script from the tool's own bytes.** The response was 2.6 MB — far past what
any reader could retype — so the harness persisted it and the script took the markup and
dropped the 234-character trailer, which is step 3 of the procedure below. **This file was
not transcribed by hand**, and that matters here more than anywhere: a snapshot exists to
be diffed, and a hand-copied one would be the unverifiable prose it replaced.

**It still holds no copy, which is how it is made rather than a fault.** `get_metadata`
returns ids, names, positions and sizes and nothing else: 5,965 `<text>` nodes, every one
self-closing, and **no `characters`, `content` or `value` attribute anywhere** — asserted
by the verification run, not assumed. An onboarding screen is mostly words, so words come
from `get_design_context` and are written down separately, which is what
`../the-first-start.md` is for.

## What is in it

| file | root | what it is |
|---|---|---|
| `70-28.xml` | `70:28` | page *03 — alo OS · Living canvas* — 2,350,573 bytes, 26,421 lines, 21,072 nodes |
| `0-1.xml` | `0:1` | page *00 — Cover* |
| `417-27657.xml` | `417:27657` | section *Ask alo · Task conversation* — 118 nodes; appearance is **not** in it, see the 2026-10-12 section |
| `ROOTS.md` | — | every root id known to exist, exported or not, because the interface will not list them |

```text
file key        nDxyF5Ho9oC4RObjVzwBNJ   canonical, by the owner 2026-10-04
page            70:28                    03 — alo OS · Living canvas
exported        2026-10-07   70-28.xml refreshed; 0-1.xml unchanged from 2026-10-04
exported by     the development PC, at the owner's instruction — see Refreshing it
figma revision  NOT EXPOSED by this interface — see Limitations
inspected as    sha256 of each export, below
```

## The canonical file, and the inspected revision

**`nDxyF5Ho9oC4RObjVzwBNJ` is the canonical design file**, named by the owner on
2026-10-04, with the living canvas at
<https://www.figma.com/design/nDxyF5Ho9oC4RObjVzwBNJ?node-id=70-28>.
**`8q0JVtnLroZYNdDkIQeJni` is historical reference and not implementation
authority** — it is `docs/design/figma-brief.md`'s own first output, and the
brief and `README.md` both pointed at it until this change.

**Within the canonical file, the approved visible states and components are the
reference, and hidden legacy layers are not.** Frame `337:23323` is the worked
example: it draws the minimized panel's empty handle while also containing a
hidden `200×600` shelf and a hidden `125.48`-tall first preview, both superseded.
`get_metadata` reports `hidden="true"`, **and a hidden parent hides children that
carry no flag of their own**, so visibility has to be tracked down the tree
rather than read off one node.

**The inspected revision is recorded as a content hash, because Figma does not
offer a revision here.** This was measured rather than assumed: no tool in this
interface returns a file version, a history id or a last-modified time, and the
REST endpoint that would needs a personal access token this machine does not
hold. A hash is what a reader can actually verify against the file they hold:

```text
70-28.xml  sha256  dc185c2876b97ed3ca0530d38912ca2371174119109473c3464cb8cb11c73f5a
0-1.xml    sha256  e0632cdbf993d95831a4eff6792b93698011e4f6cb51125ee6243934661d7a16
```

*`70-28.xml`'s hash changed with the 2026-10-07 refresh; the previous export hashed
`ccce1673…a2c97b6` and is in this file's own history.*

**It identifies the export, not the Figma document**, and the difference matters:
two exports of an unchanged file are byte-identical and hash the same, but a hash
cannot tell you *when* the file last changed, only whether this export still
matches what was committed. **A later change that obtains a token should record
Figma's own revision beside these rather than in place of them**, because the
hash remains the only figure a machine without Figma access can check — which is
two of the three machines working on this repository.

**Those three figures were stale until 2026-10-04 and nothing noticed.** The `#489`
refresh carried the owner's edits into `70-28.xml` — five fewer nodes, four fewer
lines, 1,206 more bytes — and left this table describing the file as it had been.
A description of a file that the file cannot contradict is the recurring fault in
`docs/quirks/`, so it is now asserted: `a_palette_with_one_source.rs` reads these
numbers back off `70-28.xml` and fails if they drift again.

**That assertion earned itself on 2026-10-07.** The refresh above changed all three, and
the test is what made updating them unavoidable rather than optional. **Its figures are not
the obvious ones** — it counts bytes with `len()`, lines with Rust's `lines()`, and nodes
as occurrences of `id="` rather than as XML elements — so the figures in this file were
recomputed by that definition before being written here. An element count taken from an XML
parser is a different measurement that happens to agree on this file, and agreeing by luck
is how a check gets quietly weakened.

## How to reproduce it

There is no shell command. The export comes from the Figma MCP server, which is reached
per session, so the procedure is the reproducible part:

1. `get_metadata(fileKey)` with **no** node id — lists the pages this interface can see.
2. `get_metadata(fileKey, nodeId)` for each root — returns the subtree as XML. A response
   over the tool's size limit is written to a file by the harness rather than returned.
3. From the tool result (a JSON array of `{type, text}`), take **entry 0** only. Entry 1 is
   the tool's own trailing instruction to call `get_design_context`, and is not part of the
   export.
4. Normalise CRLF to LF, strip trailing whitespace, end with exactly one newline. **Nothing
   is reordered** — tree order carries meaning.
5. Refuse to write if the text matches base64 image data, a `figma.com` or `amazonaws` URL,
   `X-Figma-Token`, `Authorization:`, a bearer token, or a `figd_` token. None of these were
   present in this export; the check is part of the procedure, not a claim about one run.
6. Verify each root: the first line carries the node id asked for, and the document closes
   the element it opened. **A response that does not close its root is partial and must not
   be written.**

## Coverage, stated as coverage and not as completeness

**This is not the whole file, and nothing here should be read as saying it is.**

```text
pages this interface lists        1      00 — Cover
pages actually exported           2      00 — Cover, 03 — alo OS · Living canvas
pages that exist                  UNKNOWN, and at least 2
```

`70:28` was exported because its id was known from a design link, **not because the
interface offered it.** Any page whose id nobody has written down is absent from this
snapshot and there is no way, through this interface, to discover that it is missing.

## Limitations, each one measured

- **Page discovery is broken.** `get_metadata` with no node id reports exactly one page,
  `0:1 00 — Cover`, while `70:28` and `351:25693` both demonstrably exist. **A listing
  containing only the cover is not evidence that the designs do not exist** — it is evidence
  that the listing is wrong. A lane reading it almost reported that this file holds nothing
  but a cover.
- **No file revision.** Nothing in this interface returns a document version or revision
  id, so a snapshot cannot be tied to one. The export date is the only anchor, and two
  snapshots taken the same day cannot be ordered.
- **Components cannot be listed.** `list_file_components_for_code_connect` answers *you
  need a Dev or Full seat on an Organization or Enterprise plan*; this account is Pro. So
  the component inventory in this snapshot is whatever appears inline as `<symbol>`, not the
  file's published library.
- **No interaction definitions.** `get_metadata` returns ids, names, positions, sizes and a
  `hidden` flag. Prototype links, triggers and transitions are **not** in it, so the
  owner's *interaction definitions where the export provides them* is satisfied by: it does
  not provide them.
- **No variables.** `get_variable_defs` is per node and was not run; variables are absent
  from this snapshot rather than empty in it.
- **Text content is not exported.** A `<text>` node carries its layer *name*. Where a
  designer named the layer after its content the words appear; where they did not, the words
  are not in this file.
- **`0-1.xml` was transcribed, not written by the export.** The cover's response was small
  enough to return inline, so it did not pass through step 3. It is byte-for-byte what the
  tool returned, but it is the one file here that a script did not write.

## Refreshing it

**One lane owns refreshes: the third PC**, which has the Figma access and the file key.
Every other lane **reads the committed snapshot** and does not call Figma — one lane reading
a live file is a single point of failure, and two lanes reading it get two answers with no
way to tell which is older.

**The 2026-10-07 refresh was the development PC, by the owner's own instruction**, which
named the file, the four section roots and the words *refresh the design snapshot*. That is
the rule being overridden by whoever set it rather than a lane deciding for itself, and it
is recorded here so the next reader does not treat the export as unauthorised. **The
hazard the rule guards against did not arise**: this was one lane reading the file, and the
figures above say which export is newer. **The rule stands for anything the owner has not
directed**, and a lane that wants a refresh asks rather than reaching for the file key.

**A failed refresh must leave this snapshot alone.** The procedure writes only after every
check in step 5 and 6 passes, so a partial or refused response leaves the last good
snapshot in place. **Report the failure; never commit a shorter tree as an update** — a
smaller file is indistinguishable from a design that lost 55 frames.

## What a new frame is, and is not

A frame appearing here is **design evidence**. It is not authorisation to build anything:
`CLAUDE.md` binds building to `docs/features.md` with a tier, inside the current release.
A design that draws something this repository has not promised is a question for the owner,
not a licence.

## The first-start account sheets, 2026-10-13

**`the-first-start-sheets.xml` is authoritative for *10 · Your name* and
*11a · Add a PIN***, and supersedes `70-28.xml` for them. The page export
holds both frames with every `Choice /` and `Action /` as a stub; this holds
the choice component expanded, which is where the measurements an
implementation needs begin.

**Two sheets of a journey of eleven.** `ROOTS.md` inventories the whole
first start — Welcome, language, region, keyboard, Your name, Protect your
computer, Sign-in options, Add a PIN, fingerprint, recovery, and the AI
choice. These two are captured first because they are what stands between a
machine with no accounts and a person who can use it: `alo-greeting`'s
`makes_an_account` has been able to make the first account since 2026-10-10,
and no screen has ever been able to call it.

**They are one composition, not two.** Both carry the same skeleton — a
stage line, a 600-wide column at (160, 164) of eyebrow, title, introduction,
two 600 × 85 choices, a consequence and two actions, with Accessibility at
the bottom left and a footer at the bottom right. That is a *setup sheet*,
and `ROOTS.md` says the rule is structural rather than a list. It should be
built once and filled many times, the way `crate::sign_in_layout` serves both
the sign-in and the unlock screen.

**The wallpaper is the one already drawn.** Both frames carry the same five
vectors at the same coordinates as `364:25552` in `176-10686.xml`, which
`crate::sign_in_wallpaper` already rasterises. Nothing new to transcribe.

**What this capture does not have**, recorded so nobody reads its silence as
absence: the `Choice /` component's own states — whether a row draws a field,
a border or a selected state is not in metadata and must be read from a
render; the contents of every `Action /` instance; and the nine other frames.

## The language, region and keyboard sheets, 2026-10-13

**`the-language-region-keyboard-sheets.xml`** holds four more of the eleven:
`185:10702` *09 - Language, region and keyboard*, and the three pickers behind
its three rows - `372:30625` *09b - Choose language*, `372:30655` *09c - Choose
region* and `185:10736` *09a - Keyboard choice*.

**They are the same setup sheet again**, which is the fact worth having: the
same 600-wide column at (160, 164), the same eyebrow at 0, title at 32,
introduction at 136, choices at 177 in rows of 85 at 95 apart, the same
Accessibility at (40, 876) and the same footer at (736, 892). Six captured
frames now share one composition, and `crate::setup_sheet_layout` is it.

**Three differences from the two account sheets**, each already answerable:

- **A picker has no introduction.** On *09b* the choices begin at 136, where
  the introduction would have been, with no gap left for it. That is the same
  shape as a sheet with no consequence, which `setup_sheet_laid_out` already
  takes as *a height of zero means there is none*.
- **The selected row is 87 tall where the others are 85**, and that was
  already settled: `../the-first-start.md` measured every card over all 77
  frames on 2026-10-07 and recorded *ordinary 85, 1px border; selected 87,
  2px border, one more pixel on each edge*, with the check that a surviving
  1.5px border would have measured 86 and none does. **This note previously
  said it had to come from a render.** It did not, and saying so sent the
  next reader to Figma for a fact this repository had already written down.
