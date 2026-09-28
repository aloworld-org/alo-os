# One quirk, one file

`../quirks.md` holds 203 entries and is where three lanes collided: 28 commits in
one week, every one appending at the same place, and an hour lost on 27 September
resolving two entries that did not disagree about anything except position.

**A new quirk is a new file here.** Adding a file cannot conflict with adding a
different file, which is the whole of the reason this directory exists.

The 203 in `../quirks.md` are not moved. 187 files cite that path in prose, and
nothing parses it — so moving them would break every one of those citations
silently, as prose rather than as a failing test. The cure would be a larger
collision than the disease.

## What an entry says

The same four things `../quirks.md` asks for, because a reader should not have to
learn two shapes:

- **Version:** what was running, exactly, and on which machine.
- **Whose:** ours, or the engine's, or the platform's.
- **Behaviour:** what happened, with the output that shows it.
- **Our response:** what we did, and what holds it.
- **Date:** when it was measured.

And one rule worth more than the shape: **write what you measured, not what you
concluded.** Four mechanisms were asserted here on 27 September that turned out to
be wrong, each costing somebody else time because they believed them. A quirk with
an honest *cause unknown* is worth more than one with a confident wrong cause.
