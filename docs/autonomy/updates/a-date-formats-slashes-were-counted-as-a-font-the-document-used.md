# A date format's slashes were counted as a font the document used

**2026-09-27.** `crates/alo-converting` told a person that a converted document
had lost a font it was never set in. The name it gave them was the engine's
default — `Liberation Serif` — for a document whose only family is Garamond. The
cause is two slashes in a date format. One condition fixes it, and there is a
test with the format in it.

## How it was found

Not by looking. `cargo test --workspace` on the machine that gates this
repository failed on
`crates/alo-converting/tests/converting_a_real_document.rs`:

```
---- an_older_word_document_is_converted_and_what_it_lost_is_named ----
  left: {FontSubstituted(FontName("Garamond")), FontSubstituted(FontName("Liberation Serif")),
         FieldFixed(Date), Comments}
 right: {FontSubstituted(FontName("Garamond")), FieldFixed(Date), Comments}
```

`main` was red at `da7e7769` before anything of mine was applied — checked with
a clean tree at that commit, which is the only way to know whose failure it is.
It is a real wrong answer and not a fixture drifting.

## What was wrong

A `.doc` is an OLE2 compound file that nothing here reads, so ADR 0011's rule
applies and `crate::inventory::read_from` takes the other road: the engine
renders the original into OpenDocument, and that rendering is inventoried. The
rendering of `tests/documents/sample.doc` on this machine holds this, in
`office:automatic-styles`:

```xml
<number:date-style style:name="N1">
  <number:month/><number:text>/</number:text>
  <number:day/><number:text>/</number:text>
  <number:year/>
</number:date-style>
```

Two text nodes. Each holds a slash. Neither is on a page, in a paragraph, or
under any style — so `family_around` found no style around them, fell back to
the default for the paragraph family as it is meant to, and got the rendering's
default: `Liberation Serif`. The document was then recorded as setting text in
it, and the copy does not contain it, so the difference reported it as
substituted.

Every text node in the part, with what is around it — the two at the top are the
whole of the bug:

| text | where it is | style around it |
|---|---|---|
| `/` | `automatic-styles/date-style/text` | **none** |
| `/` | `automatic-styles/date-style/text` | **none** |
| `Disan Ssebowa` | `body/text/p/span/annotation/creator` | excluded: an annotation |
| `A comment, so a conversion has one…` | `…/annotation/p/span` | excluded: an annotation |
| `Notes kept in the older shape` | `body/text/p/span` | `T1` → Garamond |
| `This paragraph is set in Garamond…` | `body/text/p` | `P2` → Garamond |
| `Saved on: ` | `body/text/p/span` | `T1` → Garamond |
| `9/27/2026` | `body/text/p/span/date` | `T1` → Garamond |

## Why nobody saw it

Because it is invisible wherever the engine's default family survives the
conversion. The reported loss is the difference between what the original sets
and what the copy contains; a PDF that embeds Liberation Serif answers *carried*
and the miscount never becomes a sentence. This machine's LibreOffice renders
the whole document in Noto Serif — `/BaseFont/BAAAAA+NotoSerif-Regular` is the
only font in the PDF — so here the miscount had nothing to hide behind.

**It was wrong on every machine.** What differed was whether it showed. That is
worth saying plainly, because the first guess when one machine fails a test is
that the machine is wrong, and the machine was right.

Two hypotheses were checked and dropped before the real one, both recorded so
nobody spends the time again: the machine has no `fonts-liberation` — installing
it and rebuilding the font cache changed nothing, because LibreOffice ships its
own Liberation faces in `/opt/libreoffice26.2/share/fonts/truetype`; and the
walk's element stack might be popped by self-closing tags — it is not, `Walk`
pushes only for `empty: false` and quick-xml emits no `End` for an empty
element.

## The fix

One condition, next to the one that already excludes an annotation:

```rust
if shows(one)
    && walk.within("body")
    && !walk.within("annotation")
    && let Some(family) = family_around(styles, &around)
```

**Only text inside `office:body` is text the document sets.** This is the other
half of a rule `crate::inventory::pages` already states from its own side — *a
family counts when text is set in it*, written there after a rendering declared
five families for a document that used two. A style's own literal text is not
text set in anything.

It is the general rule rather than *not inside `automatic-styles`*, because the
date format is one of several: a sequence declaration holds a name, a
configuration setting holds a value, a font-face list holds family names. Naming
the one part that bit would leave the others.

The regression test is
`a_number_formats_own_text_is_not_a_family_the_document_sets`, written from what
this machine's engine actually produced: a `M/D/YYYY` date style, a sequence
declaration, and `Liberation Serif` as the default paragraph family. It fails on
the old code and the assertion says why.

The quirk is in `docs/quirks.md` under *Pinned engines*, with the version, what
the engine does and what we do about it — because the wrong version of this code
reads as obviously right, and the next person needs the engine's behaviour rather
than a rule to obey.

## What this leaves

- **A conversion's inventory is measured against four real documents and one of
  those measurements was wrong for weeks.** The fixtures caught it in the end,
  on the first machine whose fonts differed. Every other reader in
  `crate::inventory` resolves a family the same way; `word`, `excel` and
  `powerpoint` read their own zips, where the equivalent of a number format's
  literal text is a cell format's, and `excel` already holds the rule that *a
  format given to an empty cell shows no text*. Worth one pass by whoever owns
  the lane, not by me.
- **This machine's LibreOffice renders everything in Noto Serif**, including
  text set in a family it has. Not chased, because no promise depends on which
  face a substitution lands on — only on the substitution being reported. It is
  noted because it is why the fixtures behave differently here, and a lane
  measuring type on this machine should know before it starts.

## The gate

`crates/alo-converting`: `cargo fmt --check`, `clippy -D warnings`, and its 17
test binaries — 81 unit tests and every integration test, the real-document
conversions included, which need the engine and run it. Then the workspace.

This is the documents-and-paper lane's crate and not mine. It was taken because
`main` was red on the machine that gates, which blocks every lane on it, and the
finding was already in hand; task 2 of `v0-5-documents-and-paper-plan.md` is
where it belongs and it is named there.
