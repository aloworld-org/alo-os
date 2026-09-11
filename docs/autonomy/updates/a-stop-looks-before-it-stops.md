# A stop looks for the handoff before it stops

- Date: 2026-09-11
- Workstream: the delivery supervisor (`tools/kernel-loop`)
- Contributor: Claude Code
- Status: **a graceful stop discarded a finished task; it looks first now.**

`stop` says it *finishes what it is doing and begins nothing else; nothing is
discarded*. Tonight it discarded something. Lane B was asked to stop at 20:51
while a worker was mid-task. The worker finished properly and wrote a valid
handoff at 20:58:20. At 20:58:43 the loop answered *nobody handed over its
work*, stepped over the task, and ended — with the handoff sitting exactly
where it looks for one.

The cause is one line of ordering. `waiting_for` checked the stop file
**before** it looked for a handoff, so a stop older than the handoff won every
time. That turns *finish what is in hand* into *throw away what was just
finished*, which is the opposite of the sentence on the tin.

The look now comes first and the stop second. A stop asked for while a worker
runs takes that worker's handoff if it is there and returns at once if it is
not — the second half held by a test that insists the answer comes back in less
than one polling interval rather than after an hour of looking every ten
seconds. The first half is held by a test that writes a stop file, then a
handoff, and refuses any answer but the handoff.

The task it dropped was recovered by hand — the handoff was still on disk — and
published as `347c4d7`. Nothing was lost; twenty minutes were.

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick.
**QUEUE.md/STATE.md** — `stop` means what it says again.
