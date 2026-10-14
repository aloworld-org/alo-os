# ADR 0113 — The terminal owns a gutter that says who is speaking

**Status:** **accepted, 2026-10-14, by the owner**, ruling on a design question
the Mac lane raised and answered wrongly.

## The question, and the wrong answer that preceded this

A program writing to a terminal can print **any bytes**, including alo's own
sentences. `docs/features.md` promises *Listening verified*, *Permissions have
not changed* and *Observed at 13:15* — the lines that carry every ounce of trust
in this product — and in a plain transcript a program can print all three, in
the same font, in the same stream.

**The lane's first answer was that rich results fix this**: alo draws, programs
print. That was wrong in two directions, and the desktop lane caught the first:

- **Richness is not the property.** What a program cannot do is write **outside
  its own region**. That is true whether the bytes inside are a graph or plain
  text, so rich rendering adds nothing to it.
- **And for parsed output it makes things worse.** A test table rendered from
  `cargo test`'s own bytes is alo-looking chrome built from program-controlled
  content. Teach a person that panels mean alo, and a program that prints bytes
  shaped like test output inherits that trust.

## The decision

**A narrow gutter beside the transcript belongs to the terminal. Programs cannot
write into it, move into it with escape sequences, or create its controls.**

It labels contiguous regions of the transcript by **origin**:

```
PROGRAM · cargo test
ALO · response
SYSTEM · observation
YOU · submitted request
```

Explicit words and a distinct boundary shape. **Colour may reinforce the
distinction and must never carry it alone.** Each region is labelled once, not
each line.

The labels live in the terminal's rendering and accessibility layers, **never in
the pty byte stream**. A program may print `SYSTEM · observation` inside its own
output; the terminal-owned region around it still says **PROGRAM**.

## Origin and truth are two questions, not one

This is the distinction the first answer collapsed:

- **Who produced this?** — the gutter answers it.
- **What evidence supports the claim?** — the gutter does not, and must not look
  as though it does.

An alo-generated sentence is not verified because alo generated it. A program's
output can be perfectly accurate without anybody having checked it. **Both
distinctions survive, separately.**

**Rich formatting never changes provenance.** A table parsed from `cargo test`
stays `PROGRAM · cargo test · formatted output`. It becomes
`SYSTEM · observation` only when a **separate trusted observer** supplies the
evidence — and then it says what was observed and when:

```
SYSTEM · socket observation
Process 8421 is listening on 127.0.0.1:3000.
Observed at 13:15. Application readiness not checked.
```

## What this promises, and what it cannot

**The enforceable promise is narrow and it is stated in those words:**

> Programs cannot impersonate terminal-owned provenance **within the live
> terminal**.

**No visual design makes an arbitrary screenshot unforgeable.** A malicious
image can imitate the whole interface and a crop can remove the gutter entirely.
So the distinction must survive an ordinary uncropped screenshot in both
appearances — and **we never claim a screenshot authenticates itself.**

For text leaving the terminal, a provenance-preserving transcript export is
offered. **Ordinary copying and piping keep their expected behaviour**, because
a copy that silently gained labels would be a different kind of lie.

## Rich rendering is a presentation layer, and that is load-bearing

```sh
cargo test              # may receive a rich view of recognised output
cargo test | grep fail  # ordinary bytes, unchanged
cargo test > out.txt    # ordinary bytes, unchanged
```

**No layout markup, provenance label or commentary is ever injected into
stdout**, and a program's output mode is never changed behind its back to obtain
richer data. *Only when viewed* means the graphical presentation is separate
from the command's bytes — **not** that a command behaves differently when its
window loses focus.

## Keyboard ownership

`Choose [1/2/3]:` reads terminal input while alo owns the foreground
interaction. Enter takes the displayed default; invalid input runs nothing;
`Ctrl`+`C` cancels the pending choice and returns to alo — it **does not**
approve the action and **does not** stop an unrelated server. When a foreground
child owns the terminal, input and interrupt routing belong to that child's
process group.

**So the four stops are explained by the current state rather than drawn as four
permanent buttons.**

## Who gets the next keystroke

> **The person must be able to tell who will receive their next input.**

Their own configured prompt means the shell — **custom prompts are preserved**,
and no example prompt is imposed. `alo ❯` means the conversation. An application
or a choice prompt identifies its own interaction.

**Prompt text alone is not an authentication boundary**, so input ownership also
appears in the terminal-owned provenance and accessibility structure.

## Autonomy is a state the terminal shows

The person's effective setting is visible, and it is not a vague *AI enabled*:

| State | What is shown |
|---|---|
| **Proposing** | the proposed action, its target and effects, waiting for the person |
| **Acting within your grant** | what is executing, the grant permitting it, and an immediately reachable stop |

*Acting* may include unattended execution where that was authorised. **It never
means alo can extend its own permissions**: anything outside the grant is
blocked or asks again.

The scope is reachable from a compact terminal-owned control — *alo · Can act in
Launch* — which opens to the permitted actions, resources, duration and how to
revoke. **A badge must not imply authority wider than the grant.** In the acting
state, confirmation prompts give way to visible progress and an inspectable
record of the authority used, and **recovery limits stay visible even where no
confirmation is asked.**

## The test that decides whether this works

**Three receipts, stacked, before the gutter is approved:**

1. program output claiming success,
2. alo explaining what it believes happened,
3. a system observation confirming one fact and leaving another unknown.

It must be immediately obvious that these are three sources, **without becoming
three large cards**, and it must hold in monochrome, in dark, and at narrow
width.

If it fails that test it is not built, whatever it looks like on one frame.

## Consequences

- **This is architecture, not decoration.** A region the pty cannot address is a
  compositor-owned surface, which is why it is a decision and not a style note.
- The terminal stays visually quiet. **The persistent structure says who is
  speaking and who owns input; everything else appears only when the task needs
  it.**
- `docs/features.md`'s claim that *a shell cannot close a window* was corrected
  in the same ruling: tools that request window operations exist on some
  systems. What alo adds is resolving the intended window and applying one set
  of permissions to it.
- Nothing here is built.
