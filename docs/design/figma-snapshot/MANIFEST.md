# The committed Figma snapshot

**What this is.** The node tree of the design file, exported and committed, so that a
design change shows up in this repository as a **diff** instead of as nothing. Before this,
the only record of the file was prose written by hand after reading it — and on 2026-10-04
that prose was 55 top-level frames behind without anything saying so.

**Approved by the owner on 2026-10-04**, with the size explicitly accepted: *an untraceable
design change is the larger problem.*

## What is in it

| file | root | what it is |
|---|---|---|
| `70-28.xml` | `70:28` | page *03 — alo OS · Living canvas* — 2,247,251 bytes, 25,217 lines, 20,100 nodes |
| `0-1.xml` | `0:1` | page *00 — Cover* |
| `ROOTS.md` | — | every root id known to exist, exported or not, because the interface will not list them |

```text
file key        nDxyF5Ho9oC4RObjVzwBNJ
exported        2026-10-04
exported by     the third PC, through the Figma MCP server on the owner's own account
figma revision  NOT AVAILABLE — see Limitations
```

**Those three figures were stale until 2026-10-04 and nothing noticed.** The `#489`
refresh carried the owner's edits into `70-28.xml` — five fewer nodes, four fewer
lines, 1,206 more bytes — and left this table describing the file as it had been.
A description of a file that the file cannot contradict is the recurring fault in
`docs/quirks/`, so it is now asserted: `a_palette_with_one_source.rs` reads these
numbers back off `70-28.xml` and fails if they drift again.

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

**A failed refresh must leave this snapshot alone.** The procedure writes only after every
check in step 5 and 6 passes, so a partial or refused response leaves the last good
snapshot in place. **Report the failure; never commit a shorter tree as an update** — a
smaller file is indistinguishable from a design that lost 55 frames.

## What a new frame is, and is not

A frame appearing here is **design evidence**. It is not authorisation to build anything:
`CLAUDE.md` binds building to `docs/features.md` with a tier, inside the current release.
A design that draws something this repository has not promised is a question for the owner,
not a licence.
