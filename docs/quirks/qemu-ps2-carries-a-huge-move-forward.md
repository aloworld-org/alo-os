# qemu's PS/2 mouse carries a huge move forward, and pays it out a little at a time

**Version:** QEMU 8.2.2 (Debian 1:8.2.2+ds-0ubuntu1.18), `-machine q35`, the board's
own i8042 PS/2 mouse and no tablet, in WSL Ubuntu on the walk machine. The guest's
kernel names it `ImExPS/2 Generic Explorer Mouse` (4-byte packets).

**Whose:** qemu's. Not alo's, and not the guest kernel's.

**Behaviour.** On 2026-11-10 a walk of a booted image of `main` sent qemu's monitor
`mouse_move 200 150` (the pointer moved), then `mouse_move -8000 -8000`, then
`mouse_move 300 300`, `1 0`, `-1 0` and later `50 50`. The pointer stayed drawn at
the top-left corner through all of them, and every click after a jump like that
landed there too — on the 2026-11-09 walk as well. It looked like the shell's
pointer position had been poisoned by one huge delta.

It had not. `ps2_mouse_send_packet` in `hw/input/ps2.c` clamps each packet to ±127,
sends packets only while its 16-byte queue has room — four 4-byte packets — and
**keeps the remainder** in `mouse_dx`/`mouse_dy`. The remainder is only sent at the
next input sync, which is the next `mouse_move` or `mouse_button`. So one
`-8000` becomes a debt that every later command pays 508 units of, whatever that
command asked for. Measured with no guest at all: qemu under `-qtest`, a script
acting as the i8042 driver, reading every byte the mouse queued
(`/mnt/c/dev/ps2-debt-probe.py` on the walk machine):

```
mouse_move 200 150           -> 2 packets (8 bytes), guest received dx=+200 dy=+150
mouse_move -8000 -8000       -> 4 packets (16 bytes), guest received dx=-508 dy=-508
mouse_move 300 300           -> 4 packets (16 bytes), guest received dx=-508 dy=-508
mouse_move 1 0               -> 4 packets (16 bytes), guest received dx=-508 dy=-508
mouse_move -1 0              -> 4 packets (16 bytes), guest received dx=-508 dy=-508
mouse_move 50 50             -> 4 packets (16 bytes), guest received dx=-508 dy=-508
mouse_move 0 0  (x10)        -> -508,-508 each
mouse_move 0 0               -> 1 packets (4 bytes), guest received dx=-30 dy=-30
12 zero moves until qemu had nothing left to send; total dx=-7450 dy=-7500
mouse_move 50 50             -> 1 packets (4 bytes), guest received dx=+50 dy=+50
```

The totals equal what was asked for (`200-8000+300+50`), so nothing is lost and
nothing desynchronises — the moves arrive, late, in the wrong direction for the
person watching. Each `mouse_move 300 300` after the jump reached the guest as
`-508,-508`, which the shell rightly clamped to the corner. No kernel `psmouse`
line appears because the stream is well-formed.

**What the shell does with it, held by a test.** The shell clamps a relative delta
of any size to the output and moves from the edge on the next one:
`a_huge_relative_jump_is_clamped_and_the_next_move_still_moves` in
`crates/alo-shell/tests/direct_pointer/mod.rs`. That test runs in CI with the rest
of `alo-shell`'s suite; the qemu measurement above was made once, by hand, and
nothing runs it again.

**Our response.** The walk tool was the fault. `/mnt/c/dev/click-at.sh` on the walk
machine (not in this repository) did the `-8000` jump and then one move by the
offset, so it had been clicking at the corner. It now pins to the corner in steps of
`-127`, moves out in steps of at most 60, and sends three empty moves before the
press so anything still carried is paid first. Any monitor `mouse_move` whose
component is larger than 508 should be read as *some of this arrives later*.

A person on a real machine is not affected: a real mouse reports a movement as it
happens, in small packets, and nothing ever sends it -8000 at once.

**Date:** 2026-11-10.
