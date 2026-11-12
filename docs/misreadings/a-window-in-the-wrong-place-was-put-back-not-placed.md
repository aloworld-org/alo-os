# A window in the wrong place was put back, not placed

**Found 2026-11-11, by the lane that fixed it. Nothing caught it: the
placement tests were green because placement was never asked.**

## What was concluded

On a boot walk, VSCodium opened from the Bar and landed over alo's terminal.
The brief read it as the placer failing — *placement ran before VSCodium's
real size was known*, *an Electron window that sizes itself after mapping*,
*alo's terminal not counted*, *no free space found* — and asked for a fix to
placement.

## What was true

The window was never placed. The disk's canvas layout
(`~/.config/alo/canvas-layout.toml`, read off the stopped walk disk with a
read-only loop mount) held `com.vscodium.codium` at `x = 226, y = 167` from
the **previous** session — alo's own *in front, offset* corner beside the
terminal there — and this session's window landed on exactly that corner. A
window whose application has a remembered place is given it by
`Server::put_back_where_it_was` before the placer runs, and the placer then
skips it as already placed. That road asked whether the place was reachable,
never whether another window now stood on it.

The size story was real too, and the same file showed it: VSCodium's first
frame was 457 × 157 and it settled at 1200 × 800. It was not this fault's
cause.

## The mechanism

**Two roads put a new window somewhere, and their symptoms are identical.**
A window on the wrong spot reads as *placement chose the spot*, because
placement is the road everybody has been working on. Nothing in the serial
log names either road, so the log could not tell them apart, and every
hypothesis about the placer was consistent with the screenshot.

The pixel coincidence made it worse: the remembered corner was itself an old
placer answer (48 down and across from the terminal), so the position looked
exactly like the placer's rule 5 at work.

## The cure

Before diagnosing *why did alo put this window here*, read the canvas layout
on that machine's disk and ask whether the application had a remembered place.
If it did, the window was put back, not placed. A position equal to an old
placer answer proves nothing about which road gave it this time.
