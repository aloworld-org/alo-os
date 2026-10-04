# Known roots in the design file

**Why this file exists.** `get_metadata` with no node id reports **one** page —
`0:1 00 — Cover` — and the file demonstrably has more. So discovery cannot come from the
interface, and a node id that nobody writes down is a node id nobody can find again.

**This is that written-down list.** Every root here was reached by id and confirmed to
exist on 2026-10-04. The list is not closed: a root absent from it may still exist.

```text
file key   nDxyF5Ho9oC4RObjVzwBNJ
confirmed  2026-10-04, by the third PC
```

| id | kind | size | name | in this snapshot |
|---|---|---|---|---|
| `0:1` | canvas | — | 00 — Cover | yes, `0-1.xml` |
| `70:28` | canvas | — | 03 — alo OS · Living canvas | yes, `70-28.xml` |
| `70:29` | frame | 1440×960 | 01 · Your canvas | inside `70-28.xml` |
| `351:25693` | section | 4680×7400 | Dock edges · v0.01 specification | inside `70-28.xml` |
| `8:2` | frame | 1440×1266 | Foundations — alo OS | **no** |
| `9:2` | frame | 1440×2160 | alo OS UI primitives | **no** |

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
