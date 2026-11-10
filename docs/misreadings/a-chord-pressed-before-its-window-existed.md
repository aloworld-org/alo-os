# A chord pressed before its window existed read as a lost key

**Found 2026-11-10, by reading the walk's serial log against the screenshots'
own timestamps. Nothing caught it automatically; it looked like a fault in the
shell's key path.**

## What was concluded

From the walk of `f028d5ec` in qemu (`w20-16` to `w20-22` in
`C:\dev\boot-evidence\the-walk\`): *the first Alt+F3 after an application opens
is lost.* Files was opened from the launcher (Super+Space, `files`, Enter), the
walk waited 45 seconds, pressed Alt+F3 and then Right — no window panel
appeared, and Right moved Dolphin's own selection (`w20-17`). A second Alt+F3
later worked (`w20-20`, `w20-22`). The suspects named were the focus handover
from the launcher, or the band's keyboard state not yet made for a new window.

## What was true

The shell said what it did with that chord, in the serial log:

```
[3263.37] alo-desktop: org.kde.dolphin is asked to start, by the person's own press
[3293.25] flatpak: First start: running kbuildsycoca6
[3336.47] flatpak: Call for getting org.freedesktop.portal.FileChooser version failed
[3351.75] alo-shell: a chord was pressed and could not be carried out — no focused window for the requested layout command
[3357.20] flatpak[5086]: Call to org.freedesktop.portal.Settings.ReadAll failed
```

*No focused window* is `WindowCommandError::NoFocusedWindow`, which
`Server::dispatch_window_command` returns when `Surfaces::keyboard_root` finds
no application window holding the keyboard. Alt+F3 is the only chord pressed
in that stretch that takes that road.

**Dolphin was not on the screen when it was pressed.** The two screenshots
taken just before it, `w20-16-filesa.png` (written 10:36:00) and
`w20-16-files.png` (10:36:21), both show alo's terminal and no Dolphin. Enter
reached the shell at 3263 s and the second screenshot finished about 86 seconds
of wall clock later, so it shows the screen at about 3349 s; Alt+F3 was sent
straight after it and refused at 3351 s. New Dolphin processes appear at 3357 s,
and Right, about 15 seconds later, reached a Dolphin window. This was Dolphin's
**first start** under flatpak — it rebuilt its service cache before drawing
anything — on a virtual machine that takes 4.5 seconds to deliver a key, and it
took about 90 seconds to put up a window.

So the chord reached the shell, and was answered correctly for the screen as it
was: alo's terminal was in front, it has no window panel, and no application
held the keyboard. A window the person started is given the keyboard in the
frame it maps (`crate::a_window_the_person_opened`), which is why Right went to
Dolphin once it was there.

## The mechanism

**A fixed wait stood in for *the window is there*.** The walk waited 45 seconds
because that had always been long enough, and the screenshot that would have
shown it was not was taken and not looked at before the next key went. Every
later observation was then read against a window that the step assumed, so
*Right moved Dolphin's selection* read as *the panel never opened* instead of
*the panel was asked for on a screen with no Dolphin on it*.

**And the refusal was quiet on the screen and loud in the log.** Alt+F3 with
nothing to open a panel on changes nothing a person can see, which is right —
but it means a screenshot cannot tell *refused* from *lost*. The log line that
can was there the whole time.

## The cure

- **Before a chord that acts on a window, look at the screenshot and see the
  window.** A wait is not a check; a picture with the window in it is.
- **When a key seems lost, grep the serial log for `a chord was pressed`** near
  the moment it was sent before suspecting the key path. The shell names every
  chord it could not carry out, with the reason.
- **An application's first start is not its usual start.** A flatpak's first
  launch on a fresh image builds caches first; give it its own, longer wait.
