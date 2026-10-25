# A screen that did not change is not a thing that did not happen

**Date:** 2026-10-25, during the first walk of `docs/walks/a-day-on-alo-os.md`.
**Caught by:** acting on the surface anyway, each time, rather than by anything
that failed.

## What was concluded

Three times in one walk, from a screenshot that looked the same as the one
before it:

1. **"`Super`+`A` did nothing."** The screenshot after the chord was
   byte-identical in size to the bare desktop.
2. **"The provider was not saved — the walk stops here."** After filling the
   form in and pressing Enter, Settings still said *nothing answers questions*
   and showed no provider row. This was reported to the owner as a blocker.
3. **"The PIN field drops keys."** Six keystrokes produced four dots, which
   matched a note already in this lane's memory blaming input lag.

## What was true

1. **The chord worked.** Typing immediately afterwards landed twenty-nine
   characters in a panel that had been open the whole time; the next screenshot
   showed it. The panel simply had not been drawn when the first shot was taken.
2. **The provider was saved.** Closing Settings and reopening it showed
   `gpt-4o / openai`, chosen, with the key in the keyring. The *list* had not
   been redrawn. The blocker reported to the owner did not exist, and had to be
   retracted in the next message.
3. **The field holds more than it draws.** Eight keystrokes still produced four
   dots, and the account was created and the session started — so the field
   stored what was typed and draws at most four. The memory note blaming input
   lag was a misdiagnosis, and *"fixed by retyping slowly"* credited a fix that
   had never worked.

## The mechanism

**An unchanged surface has at least three explanations, and only one of them is
"nothing happened":** the thing has not been drawn yet, the thing was drawn
somewhere this surface does not show, or the thing genuinely did not happen. A
screenshot cannot tell them apart, and on a machine that lags input by seconds
the first is the most likely of the three.

What makes this expensive rather than merely wrong is the direction of the
error. Each time, the false conclusion was **the pessimistic one** — it did not
work, it was not saved, it dropped the keys — and a pessimistic conclusion feels
like the careful one. It reads as diligence. In case 2 it produced a confident
report of a blocker that did not exist, which is worse than saying nothing.

It is the same family as this lane's settle-detection fault, where two identical
frames both predated the machine's redraw and an Enter that worked read as a key
that did nothing. That was fixed with a fixed wait before comparing. **The fix
did not generalise, because the problem was never only about timing** — case 2
had no timing component at all.

## The cure

**Act on the surface before concluding it is dead.**

- A field that looks unfocused: **type into it** and look again. One keystroke
  settles what any number of screenshots cannot.
- A list that looks unchanged after a write: **close it and reopen it** before
  reporting. A stale view and a failed write are indistinguishable from outside,
  and one of them is not a bug.
- A field that shows fewer characters than were sent: **send more**. If the
  count does not move, it is a drawing limit, not a dropped key — and the two
  call for opposite responses.

**And when the conclusion is pessimistic, hold it to a higher bar than when it
is optimistic**, because nobody argues with bad news and it therefore gets
reported unchecked. The retraction in case 2 cost more than the check would
have.
