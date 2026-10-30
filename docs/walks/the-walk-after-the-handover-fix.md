# The walk after the hand-over fix — 2026-10-30

**Machine:** qemu on the development PC, software rendering, no graphics card.
**Not a certified PC**: every claim below is about this virtual machine,
measured once, by hand, by keys sent through qemu's monitor. Nothing here runs
again on its own; the checks that do are named where they exist.
**Image:** `main` at `ab247650` merged with #1031's head `bdab939a` (local merge
`605908f3`). #1031 landed afterwards with one documentation line changed.
**Model:** `gpt-4o` at OpenAI, the owner's key, typed in on the machine.
**Screenshots:** `twelfth-*.png`, kept on the walking machine, not in the
repository.

## Two images that did not get this far

- **Main at `4eb66709` (with #1026):** the first account's desktop came up and
  was pushed off the screen seconds later by a restarted sign-in screen. Fixed
  by #1029 (the sign-in screen steps aside when the seat hands the display on).
- **Main at `ab247650` (with #1029):** the image did not build. #1027 made
  `alo-desktop` depend on `alo-agentd`. That pulled the kernel program into the
  compositor's image step, which has no `bpf-linker`. CI builds with every tool
  present and passed. Fixed by #1031. Its test
  `crates/alo-desktop/tests/the_desktop_is_built_without_the_agent_service.rs`
  now runs on every build.

## How far it got

| Step | Result |
|---|---|
| First start, by keyboard | yes: Tab reaches Continue; Enter on Keyboard opens the picker and Escape returns (#1026) |
| Account "anna", PIN, desktop | **yes, and the desktop stayed** (#1029) |
| Terminal, file in Documents | yes: the strip shows the folder, *Local* and *anna* |
| Give alo Documents, until removed | yes, after the walk script learned **End**: Settings now holds the region list, so sixty Downs no longer reach the last row |
| Add the provider | yes: gpt-4o, openai |
| Ask; one sentence waits; Allow once | yes: *Done.* |
| The record (`Super`+`R`) | yes: *Today*, a time on each entry, the rename *said yes before it happened*, and *something left this machine … openai* |
| Undo (`alo-back`) | yes: the rename was listed, the folder put back, and what it held kept beside it as `Documents.alo-was-here-…` |
| Revoke | yes: *It has already stopped*; the next question was answered *Nothing has been granted* |
| A follow-up in the same window | yes: *how do I grant it again?* was answered in context |
| The egress indicator | **not seen**: see below |
| Stop | not tried: no request lasted long enough to reach (see below) |

## What the walk found

1. **The indicator was not on screen in the one frame taken while *alo is
   working*.** As on 2026-10-28, one blank frame does not prove the indicator
   was not drawn. A helper is measuring the paint order in a test.
2. **The keyboard can be in two places.**
   - After F6 put the focus ring on the Dock's Settings button, `Ctrl`+`Alt`+`T`
     sent letters to the terminal, but Enter still pressed the Dock button.
   - It opened Settings three times, including after Escape. Only a second F6
     cleared it.
3. **A finished task window cannot be closed from the keyboard.**
   - After Escape it stayed in front.
   - `Alt`+`F4` logged *a chord was pressed and could not be carried out — no
     focused window to close*.
   - Keys went to the hidden terminal behind it, which then showed a bash
     completion prompt nobody had seen. Only a click on its X closed it.
4. **The task window's composer row draws three empty grey boxes.** No Stop
   control was seen while *alo is working*.
5. **The terminal's monospace `r` draws as a struck-through glyph** (`-rw-r--r--`).
6. **alo declined a non-file request**: *write a 700 word story* was answered
   *I can help with file management tasks…*.
7. **The desktop is in no login session.**
   - In alo's terminal, `cat /proc/self/cgroup` printed
     `0::/system.slice/alo-desktop.service`.
   - `loginctl show-session self` printed *Caller does not belong to any known
     session*.
   - `timedatectl set-timezone Europe/Berlin` printed *Interactive
     authentication required*.
   - logind's `CanReboot` and `CanPowerOff` each answered `"challenge"`.
   - **So the person cannot change the time zone, restart or power off without
     a password that nothing on screen can take.**
   - `crates/alo-image/tests/the_person_may_set_the_time_zone_and_the_agent_may_not.rs`
     passes regardless. It reads the rule, not who the rule will see.
   - The owner chose the ordinary fix: the sign-in screen starts the desktop
     inside the person's session.
8. `nmcli general permissions` timed out after 10 seconds. This was not followed
   up.

## Still owed to the next walk

- Everything in `checks-for-the-next-walk.md` that needs a region chosen at
  first start: the `/etc/localtime` label, the German home folders, and the
  first-start screens in German.
- Stop, the indicator lit, and the session checks above, once the fix lands.

## Ways of walking that were learned

- **The pointer is a relative PS/2 mouse** (*ImExPS/2 Generic Explorer Mouse*
  in the log). qemu's `mouse_move` is a delta, accelerated by libinput. Small
  steps from a known edge (`mouse_move -4 0`, a third of a second apart) land
  where they are aimed; one large move does not.
- **`type-text.sh` has no key for `|`, `&` or `"`.** It stops at the first one
  and leaves the line half typed. Clear the line with `Ctrl`+`C`, not
  `Ctrl`+`U`, before retyping.
