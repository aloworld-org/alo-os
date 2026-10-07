# Known roots in the design file

**Why this file exists.** `get_metadata` with no node id reports **one** page —
`0:1 00 — Cover` — and the file demonstrably has more. So discovery cannot come from the
interface, and a node id that nobody writes down is a node id nobody can find again.

**This is that written-down list.** Every root here was reached by id and confirmed to
exist on 2026-10-04. The list is not closed: a root absent from it may still exist.

```text
file key   nDxyF5Ho9oC4RObjVzwBNJ
confirmed  2026-10-04, by the third PC
refreshed  2026-10-07, by the development PC at the owner's instruction
```

| id | kind | size | name | in this snapshot |
|---|---|---|---|---|
| `0:1` | canvas | — | 00 — Cover | yes, `0-1.xml` |
| `70:28` | canvas | — | 03 — alo OS · Living canvas | yes, `70-28.xml` |
| `70:29` | frame | 1440×960 | 01 · Your canvas | inside `70-28.xml` |
| `351:25693` | section | 4680×7400 | Dock edges · v0.01 specification | inside `70-28.xml` |
| `8:2` | frame | 1440×1266 | Foundations — alo OS | **no** |
| `9:2` | frame | 1440×2160 | alo OS UI primitives | **no** |
| `370:30058` | section | 11400×3580 | 01 — Install alo OS | yes, `70-28.xml` |
| `371:30134` | section | 11400×5960 | 02 — Set up your PC | yes, `70-28.xml` |
| `372:30292` | section | 11400×3580 | 03 — Start working | yes, `70-28.xml` |
| `383:26066` | section | 8500×8700 | 04 — AI choice | yes, `70-28.xml` |

**All four section roots are in the snapshot as of the 2026-10-07 refresh.** They are
children of page `70:28`, which is why they arrived with it rather than needing four
exports — and why the earlier plan to re-export section `383:26066` on its own was never
necessary.

## The whole first-start journey, 2026-10-06 — the designer's own inventory

**This supersedes every partial list in this file.** Given by the designer
through the owner, and it closes the problem this file exists for:
`get_metadata` reports one page for this document however many it has, so the
list could never be derived — only asked for.

**Four section roots, which is what a snapshot refresh should export:**

| section | id |
|---|---|
| Install alo OS | `370:30058` |
| Set up your PC | `371:30134` |
| AI choice, details and access | `383:26066` |
| Start working | `372:30292` |

**77 frames in all**, counting the shared accessibility states once: **60
onboarding** frames and states — including optional branches, accessibility,
responsive examples and the first-canvas tour — and **17 installer** frames
before onboarding. **It is an inventory of states, not 60 steps a person
walks**; alternatives sit beside the step they belong to.

### Confirmed against the refreshed export, 2026-10-07

**The designer's count is right, and it is now checked rather than taken on trust.** A
script walked `70-28.xml` and counted the direct frame children of each section root:

| section | id | frames | with the identity label |
|---|---|---|---|
| Install alo OS | `370:30058` | 18 | 18 |
| Set up your PC | `371:30134` | 21 | 21 |
| Start working | `372:30292` | 15 | **7** |
| AI choice, details and access | `383:26066` | 23 | 23 |
| | | **77** | **69** |

**Section 1 holds 18 frames, not 17**, and the eighteenth is `375:30435`
*Accessibility · Setup options* — listed under accessibility in the inventory below and
physically a child of the installer section. **Both descriptions are true**; a reader
counting installer screens gets 17 and a script counting that section's children gets 18.

**The eight frames without the identity label are all in Start working**, and the rule is
structural rather than a list: **a frame carries the strip when it is a setup sheet with a
`Main / …` column, and carries none when it shows the running canvas** with
`Dock + alo Bar / fixed viewport` and `Place name`. Those eight are `183:10715`, `83:890`,
`372:30350`, `372:30442`, `372:30498`, `377:26182`, `377:26232`, `377:26280`.

### Welcome, language and account — 10

`183:10673` 08 · Welcome to alo OS · `185:10702` 09 · Language, region and
keyboard · `372:30625` 09b · Choose language · `372:30655` 09c · Choose region ·
`185:10736` 09a · Keyboard choice · `183:10675` 10 · Your name · `186:10705`
11 · Protect your computer · `185:10770` 11 · Sign-in options · `186:10737`
11a · Add a PIN · `371:30692` 11c · Add fingerprint

### Account recovery — 1

`371:30665` 11b · Account recovery

### AI choice and connection branches — 13

**The entry point is Arrival, `383:26067`, not the local-selected frame.**

`383:26067` 12d · AI choice / Arrival · `183:10679` 12 · Choose how alo helps ·
`194:10709` 12 · Detail · On this computer · `383:26127` 12e · A machine on your
network selected · `383:26261` 12f · Detail · A machine on your network ·
`384:26276` 12h · Connect a network machine · `384:26324` 12i · Confirm network
pairing · `183:10683` 12a · My provider selected · `194:10735` 12 · Detail · My
provider · `183:10687` 12g · Provider list / Arrival · `385:26346` 12o · Connect
a provider · `183:10691` 12c · No AI selected · `194:10787` 12 · Detail · No AI

### Optional access choices — 3

`183:10695` Sealed box · `183:10699` Ask each time · `183:10703` Full trust

**Each gained a `Not now` action on 2026-10-07**, at 184×48, third in the row:
`394:26225` on `183:10695`, `394:26228` on `183:10699`, `394:26231` on `183:10703`.
It leaves without granting or changing permissions — see `../the-first-start.md`.

### Ready states — 4

`371:30273` 13 · Ready for your canvas · `376:30341` 13b · Ready / No AI ·
`376:30369` 13c · Ready / Provider later · `384:26375` 13d · Ready / Network
machine

### Optional file import — 3

`183:10707` Bring your work · `183:10711` Choose files to copy · `184:10697`
Review file copy

### First canvas and optional tour — 15

`183:10715` 14 · Your first canvas · `372:30442` 14a · No AI · `184:10714`
14b · with copied files · `372:30569` 14c · Open a file · `372:30597`
14d · Create something · `377:26280` 14e · Your new document · `83:890`
15 · Explore a sample canvas · `377:26182` 15 · Show all / alo available ·
`372:30498` 15a · Explore by hand · `377:26232` 15 · Show all / Manual ·
`372:30350` 16 · alo proposes a change · `372:30527` 16a · Proposal rejected ·
`372:30381` 17 · Sample change kept · `372:30548` 17a · Sample action undone ·
`372:30411` 18 · Sample action in History

### Accessibility, keyboard and responsive — 11

**These support the flow and are not extra steps.** `375:30435` · `387:26226` ·
`384:26446` · `385:26281` · `384:26478` · `384:26513` · `384:26548` ·
`384:26583` · `385:26174` · `385:26208` · `385:26244`

### Installer, before onboarding — 17

**Another lane's screens.** `crates/alo-installer` belongs to the desktop PC, so
these are recorded for the boundary rather than claimed.

`370:30059` 01 · Download alo OS · `370:30087` 02 · Check this PC · `370:30256`
02a · Check needs attention · `375:30377` 02b · Installation requirements ·
`370:30118` 03 · Keep Windows or replace it · `370:30282` 03a · Replace Windows
selected · `370:30147` 04 · Review · Keep Windows · `370:30311` 04a · Review ·
Erase disk · `370:30179` 05 · Prepare and restart · `370:30371` 05a ·
Preparation could not finish · `375:30307` 05b · Replace · Ready to restart ·
`370:30206` 06 · Install after restart · `370:30343` 06a · Download interrupted ·
`375:30331` 06b · Replace · Installing · `375:30406` 06c · Installation details ·
`370:30233` 07 · Installation complete · `375:30354` 07b · Replace · Complete

### Retired 2026-10-06 — do not build

**`194:10761`, the dedicated alo-provider explainer, is removed from the file.**
All three providers now lead to the shared connection screen `385:26346`. It is
named here so a reader meeting it in the older `70-28.xml` export knows it is
**gone rather than missed**.

## The corrected AI-choice flow, 2026-10-06 — superseded by the list above

**Given by the owner as an inventory** rather than enumerated from the
interface, and recorded here verbatim so a reader who cannot open Figma holds
the ids. Section root `383:26066`; the flow begins at `383:26067`.

| state | id | read into this repository |
|---|---|---|
| Arrival, nothing selected | `383:26067` | **yes** — structure measured 2026-10-06 |
| On this computer selected | `183:10679` | copy read, `../the-first-start.md` |
| On this computer detail | `194:10709` | no |
| Network machine selected | `383:26127` | no |
| Network machine detail | `383:26261` | no |
| My provider selected | `183:10683` | copy read, `../the-first-start.md` |
| My provider detail | `194:10735` | no |
| No AI selected | `183:10691` | no |
| No AI detail | `194:10787` | no |
| Provider list | `183:10687` | no |
| Network discovery | `384:26276` | no |
| Network pairing confirmation | `384:26324` | no |
| Ready with network machine | `384:26375` | no |
| Accessibility options | `375:30435` | no |
| Screen reader on | `384:26446` | no |
| Accessibility keyboard focus | `385:26281` | no |
| Choice keyboard focus ×4 | `384:26478`, `384:26513`, `384:26548`, `384:26583` | no |
| Small screen, 1280×720 | `385:26174` | no |
| Wide screen, 2560×1080 | `385:26208` | no |
| Long translation example | `385:26244` | no |
| Provider connection | `385:26346` | no |
| Screen reader enabled from Welcome | `387:26226` | no |

**`183:10687` is now *12g · Provider list / Arrival*.** Its earlier name in
`70-28.xml` — *14 · Setup / alo · Europe* — is no longer authoritative, and a
reader matching on that name matches the wrong thing.

**Three ids here are in `70-28.xml` under older names with different children.**
`183:10679` is *12 · Choose how alo helps* in the file and *14 · Setup / alo on
this computer* in the export; `183:10683` and `183:10691` are likewise changed.
For anything in this flow, **read the file, not the export**.

**`8:2` and `9:2` are not on `70:28`.** Neither appears anywhere in `70-28.xml`, so each
sits on a page this snapshot does not hold and whose id is not known. **That is the proof
that the page listing is wrong**, stated as a fact about two nodes rather than as a
suspicion: a file with one page cannot contain a frame that is on none of them.

They are **not exported** here for one reason: their responses were small enough for the
tool to return inline, so they did not pass through the step of the procedure that writes a
file. Transcribing them by hand would put the one thing this snapshot exists to prevent —
an unverifiable copy — into the snapshot itself. `0-1.xml` is the single exception, and
`MANIFEST.md` says so where somebody will read it.

## A frame's number does not identify it

Counted in `70-28.xml` at depth 2:

```text
names opening with a number    142
distinct numbers                49
numbers used more than once     15
14 -> 24 frames     10 -> 11 frames     16 -> 9     17 -> 9
```

**This is the mechanism that produced the `07` error** in three places: a page was named
after a frame's number. `78:1225` is `07 · Move a window`, so both halves of the wrong
sentence were real, which is how it survived being read.

A refresh, or anything written from this snapshot, **carries node ids**. A number may
accompany an id; it may never replace one. Where a family prefix is present the pair is
unambiguous — `Minimized panel / 07 · Collapsed rail` is one frame, `337:22408`. A bare
`07` is not, and a bare `14` is twenty-four.

## What is in the two that are missing, since it is already known

Recorded here because it bears on two questions the owner is holding, and because a
reader should not have to call Figma to learn that it exists.

**`8:2` Foundations** carries the palette as six core swatches — Light, Surface, Navy,
**Deep teal**, Soft light, Positive — and six semantic roles: Cool surface, Muted text,
Warning, Danger, alo on dark, Dark surface. The put-aside design note asks the owner
whether the panel *introduces anything new or only uses what is there*; this is the *what
is there*, and `docs/design/palette.toml` is the repository's copy of it.

**`9:2` alo OS UI primitives** carries the components as `<symbol>` nodes whose names hold
their variant properties in full, for example
`Kind=Permission, Theme=Light, Layout=Regular, Focus=Default`. Seventeen notification
variants across five kinds — Permission, Working, Finished, Failed, Message — in Light and
Dark, Regular and Compact, Default and Keyboard focus.

**This is the component inventory that `list_file_components_for_code_connect` refuses to
give.** That tool answers *you need a Dev or Full seat on an Organization or Enterprise
plan*; the components are nonetheless in the tree, because a component used in a file
appears in it. So the plan tier limits the **library listing**, not the inventory — which
is worth knowing before anybody concludes the components are unreachable.

## A note for whoever refreshes this

**Write a small document here whole; do not patch it.** Building this file the first time,
a patch script inserted the *A frame's number does not identify it* section **twice**: the
script anchored on the heading below it, the anchor survived the first insertion, and
re-running the script against a moved base matched and inserted again.

**The size assertion did not catch it.** The script intended 19 lines and added 19 lines —
correctly, for the second time. **A check that a patch added what it meant to add cannot
tell one application from two**, which is a real limit of an otherwise sound habit. For a
file this size the remedy is to write it in full and read it.
