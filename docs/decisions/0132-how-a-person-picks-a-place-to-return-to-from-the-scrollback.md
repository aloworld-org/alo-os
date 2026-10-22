# ADR 0132 — How a person picks a place to return to from the scrollback

**Status:** **proposed, 2026-10-22, by the Mac lane.** It is a decision and not a
commit because every road adds to a **published contract**, and the one that
looks cheapest is the one `docs/contracts/shell-integration-marks.md` makes a
promise about.

**Context:** `docs/features.md` ★ *scrollback you can move along, not only read*
— the half of its sentence still unbuilt,
[ADR 0127](0127-a-persons-own-commands-are-bracketed-in-a-session-they-asked-for.md)
(a person's own commands are bracketed in a session they asked for),
[ADR 0130](0130-what-takes-the-bracket-before-a-persons-own-command.md) (what
takes the bracket — and why not the mark),
[ADR 0123](0123-alo-learns-what-a-person-typed-from-the-shell-not-the-screen.md)
(alo learns what a person typed from the shell),
[ADR 0131](0131-what-happens-to-what-is-in-a-folder-when-it-goes-back.md),
`docs/contracts/shell-integration-marks.md`,
`crates/alo-terminal/src/what_the_shell_says_about_the_prompt.rs`,
`crates/alo-as-it-was`, `crates/alo-back`.

## The question

The promise is **two halves in one sentence**: *scroll up, choose one, and your
files are as they were before it ran.* As of 2026-10-22 both mechanisms exist —
the bracket, and the folder going back — and **the choosing is by typing a
place's name**, not by picking a row of scrollback:

```
alo-back                      the places you can return to, newest first
alo-back <place> <folder>     put that folder back as it was
```

A place is named by an opaque directory and a moment. **Nothing connects a place
to the row of output the command it was taken for produced**, and *scroll up,
choose one* is exactly that connection.

## What was measured, 2026-10-22

| asked | answer |
|---|---|
| does a place carry which command it was for | **no, deliberately.** ADR 0130: *not the command, not its arguments, not the directory it will run in.* A request carrying what a person typed would hand alo a copy of every command they run, for a service with no use for it |
| does the terminal know every command's text | **no.** `alo-typed` fires on `Ctrl`+`X` `Ctrl`+`A` **only** — `bind -x` in `alo.bash`, on that one chord. Nothing runs while the person types, which ADR 0123 chose on purpose |
| does the terminal know where each command's output begins | **it is told and it throws it away.** Of the five things `AShellSaid` can be, **four reach no production code at all**: `APromptIsAboutToBeDrawn`, `ThePromptHasEnded`, `TheCommandIsRunning` and `TheCommandFinished` are named only in tests. `TheyTyped` is the one with a caller. Counted across 3,015 files with a control for a variant that does not exist |
| can the terminal read the places itself | **yes, with no privilege.** `docs/contracts/kept-undo-folder.md` gives a person's own directory to that person, and the compositor runs as them |
| could a place's name carry the asking shell's process | **no.** The daemon learns the peer's credentials from the socket, and the peer is the `$( )` subshell `alo-before` runs in — a pid of no use to anybody. The shell's own pid could only arrive by being *sent*, which is a request field and a contract change in its own right |

**So the one thing missing is small and it is not derivable:** which place belongs
to which command boundary. Everything else is built.

## The options

### Option A — `alo-before` prints a mark naming the place it just kept

The reply gains `named: Option<String>` (additive). `alo-before`, which already
runs inside `PS0` — *after* the person presses Enter and *before* the command
runs — prints one OSC 133 mark carrying that name. The terminal already parses
that family, already receives `TheCommandIsRunning` at the same instant, and
would keep the pair against the row where the output starts.

- **The contract question, and it is the whole decision.**
  `docs/contracts/shell-integration-marks.md` promises *a forged mark can make
  alo explain a command nobody typed, and it cannot make alo do anything*, and
  ADR 0130 refused the mark as the **trigger** on exactly that sentence. This is
  not a trigger: the mark would **label a row**, and the act still re-reads the
  authoritative list off the disk and refuses a name that is not in it. **The
  worst a forged mark achieves is labelling a row with another of this person's
  own places** — which they can already name directly on the command line. That
  reading may be right and it is not this lane's to assert.
- **For it:** nothing new learns anything about a command. The mark carries a
  directory name alo chose, never a word the person typed.
- **Against it:** a sixth thing in a published contract, and a person reading
  their own `.bashrc` now sees alo printing something on every command.

### Option B — the terminal pairs places to boundaries by order and moment

No new surface. The terminal counts command boundaries and the places that
appeared between them.

- **Against it, decisively:** `/var/lib/alo-undo/<uid>` is **one directory for
  all of a person's shells**. Two terminal windows interleave their places, and
  order within one window says nothing. Pairing by nearest-preceding moment is a
  guess, and it is wrong **exactly** when two commands land in the same second —
  which a person running a loop does constantly. The cost of being wrong is
  putting back the wrong folder from the wrong moment, which is the one mistake
  this whole feature exists to let them undo.

### Option C — leave the choosing as it is, and say so in `docs/features.md`

What this repository does today. The sentence is already narrowed there.

- **For it:** it costs nothing, and the feature works. A person who wants to go
  back names a place and a folder.
- **Against it:** *scroll up, choose one* is the sentence that makes this
  different from every other terminal, and it is the half a person would
  actually use. Typing an opaque directory name is the thing the promise was
  written to avoid.

## What would have to be true either way

**The four unused marks need a consumer or an explanation.** They are parsed,
tested and in a published contract, and they reach nothing. Option A gives two
of them a caller. Option C leaves four declared-and-undispatched sequences in a
contract, which is worth writing down wherever this lands — *the shape this
repository has found six times* is a mechanism nobody calls, and this is one.

## What this does not decide

- **Anything about the whole home**, which ADR 0127 term 2 settled: at the next
  sign-in, never by a privileged helper.
- **What the row of scrollback looks like once it can be picked** — a surface,
  and the design file is where a surface's appearance comes from.
