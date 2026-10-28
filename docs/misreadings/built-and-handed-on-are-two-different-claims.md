# Built and handed on are two different claims

**Found 2026-10-27 walking Demo 1 on a booted machine. Nothing caught it: every
test was green, and the comment that described the fix was wrong about the code
it described.**

## What was concluded

On 2026-10-17 the nested backend was found to draw no terminal, and fixed. The
note on `TheTerminalToDraw` explained the difference between the two backends:

> passed `None` for the terminal picture … while `direct_desktop` drew it from
> the same helper.

So the real-display backend was taken to be the one that worked. The fifth walk
(2026-10-27) then pressed `Ctrl+Alt+T`, saw nothing, and put it down to
applications still installing.

## What was true

`direct_desktop` **built** the terminal's picture every frame, through
`drawn_terminal`, and passed it into `frame_pictures`, which stored it in
`DesktopPictures.terminal`. It then built the painter's layers by hand: the
desktop, the status area and Settings, with `..NativeLayers::nothing()` for
the rest. That *rest* was the terminal, the camera-and-microphone indicator and
the notifications. All three were laid out on every frame since #710 and thrown
away before painting. Nothing failed, so nothing was logged. The one test that
checked what reaches a card copied the same three-field literal.

## The mechanism

**A value that is built is not a value that is used, and a struct-update
default (`..nothing()`) makes the gap invisible.** Every field the literal did
not name compiled as *absent*, and absent is a legal state for each of them:
no terminal open, nobody watching, no notifications. So the omission and a
quiet machine were the same frame. The 10-17 fix compared the two backends at
the step where they called the helper. They differed one step later, where the
helper's result was used.

## The cure

- **One constructor for the layers**, `DesktopPictures::layers`, used by every
  backend. A new picture is then added in one place, or it is visibly absent.
- **When checking that something reaches a screen, follow the value to the
  call that consumes it**, not to the call that produces it. `drawn` in a
  comment should mean painted.
- **A symptom on a walk is not explained by the first thing still in
  progress.** Installs were running during the fifth walk, and they had
  nothing to do with this.
