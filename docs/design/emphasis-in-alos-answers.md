# Bold, headings and code in alo's answers

**The owner's decision, 2026-10-28: alo's emphasis is drawn, not stripped.**

A model writes Markdown. Until this date a task window took the marks out
(#985), because the design draws a message in one style, Body/Large, and the
measured frames hold no bold body, no heading inside a message and no code face.
The owner chose to draw them instead. The design file still gives no sizes or
weights for them, so **every one is a style the design's type scale already
has**; nothing here is a new figure.

| alo writes | drawn as | from the type scale |
|---|---|---|
| ordinary words | Body/Large, Manrope **Regular 400**, 16 / 25 | what the design draws a message in |
| `**bold**`, `__bold__` | Manrope **SemiBold 600**, at the block's own size | the scale's one heavier weight |
| `#`, `##`, `###` heading | **Heading/Medium**, Manrope SemiBold 22 / 30 | the task title's style |
| `####` to `######` | as bold | — |
| `` `code` ``, a fenced block's lines | **Geist Mono Medium 500**, at the block's own size | `System/Mono`'s face and weight |

Three choices inside that, and why:

- **All three heading levels are one style.** The scale also has Heading/Large
  (28) and Display/Medium (36); a reply with three sizes of heading reads as a
  document dropped into a conversation. The owner's own example was *a heading
  uses the existing title style*.
- **Code keeps the line's size, not System/Mono's 12 / 18.** A word of code in a
  16 / 25 sentence drawn at 12 shrinks mid-sentence; the face and weight say
  *code*, the size stays the sentence's.
- **Ordinary words are now Regular, as the design's table always said.** Every
  style in a task window was drawn SemiBold until this change, because no
  Regular Manrope was bundled — and bold, the scale's SemiBold, would then have
  been indistinguishable from the words around it. `Manrope-Regular.ttf` is the
  upstream project's own static cut; `crates/alo-shell/fonts/README.md` records
  its revision and hash. This touches every message, a person's included, and
  the activity and approval lines (Body/Medium), and the destination (12 / 22
  Regular): all of them are Regular in the design.

## What is still only taken out, or left alone

- **Links** are said as `words (address)`: nothing in a task window can be
  followed, and where a link goes is still said rather than hidden.
- **List marks** `-`, `*`, `+` become `•`; numbered lists are left as written.
- **Images and tables** are not read at all: their characters reach the person
  as written.
- **Single `*` and `_`** are left alone — too often a multiplication or a file
  name.

## Whose words are read

**Only alo's replies and questions.** A person's words are drawn exactly as they
typed them, marks included, and a proposed change is the sentence a person
approves, so nothing reads or restyles it. `crates/alo-shell/src/alos_marked_words.rs`
is the reader; `crates/alo-shell/src/task_text.rs` maps each mark to the style
above.

## How it is checked

By ink and by face, not by the ask. In `crates/alo-shell/src/task_text_tests.rs`,
run by every `cargo test` of the shell: the same words carry more ink bold than
plain; a heading line measures taller than a body line; each mark's first glyph
is read back to the family and weight that drew it; a person's `**` are inked.
None of this has been looked at on a booted machine yet.
