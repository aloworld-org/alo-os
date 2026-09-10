# Every verb, and how a person does the same thing by hand

`docs/features.md` promises at v0.01 that **anything an agent verb can do, a
person can do by hand**. [ADR 0009](decisions/0009-a-good-computer-without-the-agent.md)
is where that was decided, and where it was widened past the case it was written
for: the agent is unavailable for six reasons and only one of them is a choice.
It was declined at setup. No model has been downloaded. The machine is offline.
The provider is down. The key expired. **The money ran out.**

In every one of them the machine must lose *convenience* and never *capability*,
because a capability an agent has and a person does not makes whoever cannot pay
a second-class user of a computer they own — and this product is explicitly for
individuals as well as companies.

This file is that rule, verb by verb. `crates/alo-by-hand` is what makes it true
of the verbs rather than of a list somebody wrote once: every verb the machine
declares must be answered here exactly once, every answer must be about a verb
the machine still declares, and **a verb added and not answered fails the gate in
the change that adds it** — which is the only moment anybody has the knowledge to
answer it. `docs/contracts/agent-verbs.md` asks the question there, as rule 7 of
adding a verb.

## What an answer has to be

**A promise in `docs/features.md`, quoted.** Not a sentence somebody wrote here.
ADR 0009's rule is not *have an idea about how a person might manage*; it is *no
surface may be left out because an agent can do it instead*, and the only place a
surface is committed to is the definition, with a tier. So the answer quotes the
definition, and **the release that owns it is read off the line** rather than
asserted beside it.

**Or the debt, with the release that owes it** — `**Owed at:** [v0.5] …`, where
the release has to be one the definition actually makes promises for. One verb
needs this form today, and writing it that way is what keeps it visible.

What no check can reach is whether a surface *really* lets a person do what the
verb does. That is the reader's, and it is why this document is prose with
sentences in it rather than a table of ten ticks.

## What this document already shows, which is not comfortable

**Six of the ten verbs ship at v0.01 and their plain way arrives at v0.5.** The
file manager, the search, the text editor and the terminal are all v0.5; the six
file verbs are v0.01. So on a v0.01 machine with no agent — or with an agent
whose provider is down — a person cannot list, read, find, rename, move or
archive anything at all, and the four application verbs are the only ones whose
plain way lands in the same release as the verb.

That is a scope fact rather than a bug in a crate, and it is not this check's to
refuse: v0.01 is *it boots and the agent acts*, the desktop is v0.5, and moving
either is the owner's decision and nobody else's. What is not acceptable is that
it was true and unwritten. It is written here now, and `ROADMAP.md`'s v0.5 exit
gate is where it comes due.

## Every verb, one at a time

### list_folder

**By hand:** a person opens the folder and reads what is in it, in the file
manager — `A file manager, with trash, and archives that open`.

### read_file

**By hand:** a person opens the file in whatever opens that kind of file, which
is what every desktop does — `A text editor and an image viewer, so a fresh
machine is not helpless` for the two a fresh machine has, and `File associations
— what opens what, changeable by a person` for choosing something else.

### find_in_folder

**By hand:** a person searches for the file themselves, by name, in the file
manager — `Search your own files, without asking anything`, which the definition
already words as this rule: *the agent's "where is that file?" is a nicer way to
reach this; it is not the only way, and a machine whose only search is a
conversation is a machine somebody locked out of their own documents*.

### rename_file

**By hand:** a person renames the file where it is, in the file manager —
`A file manager, with trash, and archives that open` — or in a shell, because
`Law 2 forbids the *agent* running arbitrary commands; it says nothing about a
person`.

### move_file

**By hand:** a person drags the file into the folder, or cuts and pastes it, in
the file manager — `A file manager, with trash, and archives that open`, with
`Copy, cut and paste — text, images and files, across applications` for the
keyboard way.

### archive_folder

**Owed at:** [v0.5] — nothing in `docs/features.md` promises **making** an
archive by hand. The file manager line promises `archives that open`, which is
the other direction, and stretching it to cover creating one would be this check
lying in the first change that used it. The honest state is that this verb can do
something no surface can, and the answer is a line in the definition that does
not exist yet: an archive a person makes from a folder, in the file manager, at
v0.5 with the rest of it. Proposed to the owner in
`docs/autonomy/updates/every-verbs-by-hand-answer.md`; the scope gate is theirs.

### open_application

**By hand:** a person opens it from the launcher, the way they open anything —
`Launcher and window management: open, focus, close, tile`.

### focus_application

**By hand:** a person clicks the window, or switches to it with the keyboard —
`Switching between windows, and between applications`, and
`Launcher and window management: open, focus, close, tile`.

### close_application

**By hand:** a person closes the window, and the application gets to ask about
unsaved work exactly as it does for the verb —
`Launcher and window management: open, focus, close, tile`.

### arrange_application

**By hand:** a person drags the window to an edge to take half the screen, or
maximises it — `Window management: move, resize, snap, tile, minimise, maximise,
close`. The verb offers the left half, the right half and the whole screen, and
each of those three is one of *snap*, *tile* and *maximise* on that line.

## What this document does not do

It does not say the plain way is **built**. Every entry above names a promise
with a tier on it, and a tier is a commitment rather than a screen; `A file
manager, with trash, and archives that open` is v0.5 work that has not started.
The distinction is deliberate: what ADR 0009 forbids is a surface being *left
out* because an agent covers it, and the place a surface stops being left out is
the definition. Whether it then gets built is `ROADMAP.md`'s exit gate, and
whether it works is a test on a certified machine.

It does not answer for anything but verbs. ADR 0009's rule is wider — *no surface
may be left out because an agent can do it instead* — and applying it to
`docs/features.md` as a whole is what found the file search that nobody had
promised. `crates/alo-reconciling` and `docs/autonomy/v0-01-evidence.md` are that
half.
