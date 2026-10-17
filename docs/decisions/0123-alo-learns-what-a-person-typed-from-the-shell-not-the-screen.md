# ADR 0123 — alo learns what a person typed from the shell, never from the screen

**Status:** **proposed, 2026-10-17**, by the Mac lane under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision, following [ADR 0085](0085-how-a-person-reaches-settings.md)'s
precedent. The owner rules; nothing here is built until they do.

## The question

Two promises in `docs/features.md` need the text of a command a person has
typed:

- **Explain a command before it runs** — needs it **before Enter**, while the
  line can still be edited or abandoned;
- **Hand somebody a session they can replay** — needs it **after**, with what
  each command changed.

alo does not have it. How does it learn it, and what is that knowledge worth?

## What was measured, 2026-10-17

All four on this lane's VM, against the tree at `c4f5481a`.

| Asked | Answer |
|---|---|
| anything in this repository keeping the typed line | **nothing.** `what_a_person_typed` turns a key into bytes and `a_person_typed` writes them to the pseudoterminal, keeping none |
| the command in front | a **process id** — `/proc` describes something already running, which is too late for the first promise |
| can the engine's listener carry a prompt mark | **no.** `alacritty_terminal`'s `Event` carries `MouseCursorDirty`, `Title`, `ResetTitle`, `ClipboardStore` and `ClipboardLoad`. There is no semantic-prompt event |
| can a `Handler` method see one | **no.** `vte-0.15.0`'s own `osc_dispatch` (`src/lib.rs:1329`) ends at `unhandled(params)` for an OSC number it does not know, and it does not know 133. The sequence is dropped inside the engine |

So the mechanism does not exist at any layer, and the two cheap-looking roads to
it are the two below.

## Three roads, and why two are closed

**A. Read it off the screen.** Rejected. It decides what the *person* typed from
what a *program* printed, which is [ADR 0113](0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md)'s
fault one storey up — there about who spoke, here about what was said. A
right-hand prompt, a continuation line, a reflow at a resize, a syntax-colouring
line editor that rewrites what it echoed, or a prompt that echoes nothing at all
each produce a **confident wrong answer**, and the person reads that answer as
the command they are about to run.

**B. Keep our own copy of the bytes we forward.** Rejected, and this one is
demonstrable rather than arguable: the shell owns the line editor, so the first
`Up`, `Ctrl`+`R`, `Ctrl`+`W`, multi-line quote or bracketed paste makes our copy
and the shell's buffer disagree — **and nothing tells us they disagree.** A
silent divergence in the thing being explained is worse than having no
explanation, because the second says *I do not know* and the first does not.

**C. The shell says so.** Chosen. The only component that knows what is in the
line editor is the component that owns it.

## The decision

**1. The cycle is read as OSC 133, which is not ours.** The semantic prompt
marks — `A` prompt start, `B` command start, `C` command executed, `D` exit code
— already exist and are already emitted by what people have installed for kitty,
iTerm2, WezTerm and VS Code. A person who has shell integration gets this
working on alo OS without touching their shell again, and an adapter author has
one sequence to learn rather than an alo-specific one. This serves *replay* and
the exit codes under *find a command by what it did*.

**2. The unrun command is a mark the person asks for, from inside the line
editor.** No shell hook runs *while* somebody is typing; `B` fires when the
prompt is drawn, before there is anything to report, and `C` fires after Enter,
when an explanation is a history note rather than a warning. So the explain
chord is bound **in the shell**, by the snippet, as a `zle` widget or a
`bind -x`: the person presses it, the shell reads its own buffer and emits

```text
OSC 133 ; alo-typed ; <base64 of the command> ; <decoded length in bytes> ST
```

The first parameter is a namespaced token rather than a letter, so no conforming
reader can mistake it for `A`–`D`, and the `alo-` prefix cannot collide with a
key another terminal defines later. This is additive to OSC 133, not a new
number squatted from somebody else's range.

[ADR 0113](0113-the-terminal-owns-a-gutter-that-says-who-is-speaking.md) decides
why it must be this way round: *when a foreground child owns the terminal, input
and interrupt routing belong to that child's process group*. The shell has the
keys. alo does not take the chord; the snippet binds it.

**And this is the charter's context rule satisfied by construction rather than
by policy.** *Context is offered, never watched.* There is no mark between a
prompt and an asking, so there is nothing for alo to read while somebody types —
not because a background reader was forbidden, but because none can exist. That
is a stronger guarantee than any setting, and it is the main argument for this
road over A and B, both of which require alo to watch continuously and then be
trusted not to.

**3. The declared length is what makes a truncated command refusable.** `vte`
caps an OSC payload at `MAX_OSC_RAW` = 1024 bytes and, measured in its own suite
at `src/lib.rs:1020`, **dispatches the truncated payload rather than dropping
it**. A long command would therefore arrive as a fragment and be explained as
though whole — the same wrong-answer fault as road A, arriving by a different
door. So the snippet declares the command's byte length, and **a payload that
does not decode to exactly that length is refused**, with the person told the
command was too long to read rather than shown an explanation of its first half.
*A single number cannot be caught being wrong; two that must agree can.*

**4. What a mark reports is the shell's claim, not a fact alo established.**
Any program in the session can print the sequence. Every surface built on this
carries that standing, worded as the rich result's *Not independently verified*
already is. A per-session secret inside the sequence buys less than it appears
to, because any program in the session can read the environment that would hold
it. **What bounds the harm is that a mark grants nothing**: it cannot run
anything, cannot change or hold a grant, cannot approve a change and cannot
relabel the gutter. A forged mark can make alo explain a command nobody typed.
It cannot make alo *do* anything.

**5. It never touches the gutter.** ADR 0113 stands unchanged. A prompt mark
says where a prompt and a command are; it does not say who spoke, and the gutter
is decided by origin rather than by bytes. This is a test, not a sentence.

**6. The person installs it.** Law 5. alo ships the snippet, the sentence saying
in plain words what it reveals — *every command you run, to alo, on this
machine* — and a one-press way to add it. alo does **not** write to a person's
shell startup unprompted, not on first sign-in and not helpfully. Without the
snippet the terminal works exactly as it does today, and *explain a command
before it runs* says it needs shell integration rather than guessing.

**7. It is read with a second parser, and no engine is patched.** Because the
sequence dies inside `vte` (measured above), the reader cannot be an event or a
`Handler` method. `alacritty_terminal` re-exports `vte` (`src/lib.rs:20`) and
`vte::Parser::advance` takes any `Perform`, so the grid runs the same bytes
through a second parser whose `Perform` implements `osc_dispatch` and nothing
else. One extra pass over bytes already in cache; no fork, no source patch,
`VoidListener` keeps its job. *Engines are configured, never written in.*

## What this does not decide

- **Which surface asks for an explanation, and what the chord is.** Separate,
  and ADR 0085's subject matter rather than this one's.
- **Replay's format.** This gets the commands and their exit codes. *What each
  one changed* is the bracket of [ADR 0045](0045-what-undoing-rewinds-to.md)
  and is decided there.
- **Which shell ships a snippet first.** Scope, not a decision — and *certified
  before compatible* answers it: one shell that works completely.

## Consequences

- `docs/contracts/` gains the sequence: what alo reads, what an adapter must
  emit, how the length is declared. A public surface, so additive and versioned.
- `alo-terminal` gains the second parser and a reader for the two marks.
- *Explain a command before it runs* and *hand somebody a session they can
  replay* become buildable; both are **nothing** in
  `docs/autonomy/the-terminal-in-full-plan.md` today and blocked on this and
  nothing else.
- A person without the snippet sees no change at all.

## The tests that decide whether this works

Written here because they are the ADR's falsifiable half:

- a forged `OSC 133` from an ordinary program changes **nothing** a person is
  told about who is speaking;
- a mark **split across two reads** is still read, because a pseudoterminal
  hands over bytes at arbitrary places and a sequence broken mid-payload is the
  ordinary case rather than the strange one;
- a payload longer than the parser's cap is **refused**, not explained;
- a declared length that disagrees with the decoded payload is **refused**;
- with no snippet installed, nothing in the product claims to know the typed
  line.
