# ADR 0082 — A shipped machine shows its start-up on its own screen, and opens no serial console unless asked

**Status:** **proposed, 2026-10-02.** Task 21 of
`docs/autonomy/the-installer-plan.md` asks this question directly — *say
whether a serial console on a shipped machine is acceptable at all, or whether it
is something the installer turns on only when it was asked to* — and says the
answer *is made once and written down, not slipped into a test.* So it is written
here rather than decided by whichever argument a walk happened to need.

## The fact that forces the question

The install writes **no console argument at all**. `Writing::arguments` in
`alo-installing` passes `--source-imgref`, `--wipe`, `--filesystem` and the disk,
and nothing else, so the boot entry the install leaves behind reads:

```
options root=UUID=… rw ostree=/ostree/boot.1/default/…/0
```

The **installer environment** sets a console on its own command line, which is why
every walk up to and including the install can be read on the serial line. **The
system the install leaves behind does not.** From the moment the firmware hands
over, the serial line goes quiet and stays quiet.

Measured on 2026-09-27 by the first run of
`the_kept_computer_is_the_same_computer_twice`: both starts of the kept computer
looked exactly like a hang — last line `PageFaultExitBoot` at `ExitBootServices`,
then nothing for the full 480-second wait, one vCPU at 100%, no disk bytes moving.
The screens said the opposite: both rounds reached `fedora login:` about forty
seconds in, network up. **The machine was idle at a login prompt, and idle is
indistinguishable from hung when the console is silent.**

So a quiet serial line on an installed machine is not evidence of anything, and
`console.wait_for` against one can only ever time out.

## The decision

**The install writes `console=tty0`, always. It writes a serial console only when
it was asked to.**

```
always            console=tty0
only when asked   console=tty0 console=ttyS0,115200n8
```

### Why `tty0` always, rather than nothing

Nothing is what we have, and it is not neutral: with no `console=` the kernel
picks, and what a person sees when their machine starts is left to a default
nobody chose. `console=tty0` says *the screen in front of them* out loud. On a
laptop with no serial port it changes nothing observable, which is the point — **a
real machine boots exactly as before.**

### Why the serial line is not unconditional

**A serial console is a console: anything that can reach the port can type at it.**
It is a login surface, and it is one that needs no password prompt to be worth
attacking — a machine that will print its boot to a port will take input from it.
On a sovereign workstation sold on the claim that nothing leaves silently, shipping
an extra way in by default is the wrong default even where the port is usually
absent.

The usual counter — *the certified laptop has no serial port, so the argument is
inert* — is true and is an argument for it being **cheap**, not for it being
**right**. An argument that is inert on the machine we certify and live on the
machine somebody else runs is exactly the kind that gets shipped unexamined.

### Why not refuse it outright

Because the walks need it, and a walk is not a person's machine. The plan's
constraint says that if a serial console is judged unacceptable then *the walk gets
its observability another way — the screen, asserted against properly.* That is a
real alternative and a worse one here: reading a screen means parsing a picture,
which is how this task was discovered in the first place — *the only way to check
such a walk today is to look at a picture by hand.*

So the answer is neither *always* nor *never*: it is **asked for**, which is the
same shape as every other capability in this system. `docs/features.md`'s standing
rule is that the person grants and the agent never assumes; a console a machine
opens to the outside is a grant, and grants are enumerated and deliberate.

## What this is not

**Not a patch to any engine** (ADR 0011). `bootc` already offers `--karg` on
`install to-disk`, and a `kargs.d` drop-in in the image is the other offered road.
This is configuration of the engine, chosen between two things it already does.

**Not a change to what the installer environment does.** That environment sets its
own console and keeps it: every walk up to and including the install stays readable
exactly as it is today.

**And not a new distinction.** `alo-installing`'s `console` module already divides
the world the right way — `every_console` for what a person reads, `every_serial_line`
for what the machinery reads, with `tty0` and `tty1` named as screens and `ttyS0`,
`hvc0` and `ttyAMA0` named as lines to somewhere else. This decision applies that
same division to the machine the install leaves behind, rather than inventing one.

## The consequence somebody will meet

A walk that wants to read an installed system must ask for the serial console at
install time. A walk that forgets will find the line quiet — and **that is the
state this ADR exists to stop being mysterious**, so the asking is in the
installer's own API rather than in a comment: a caller writes
`Writing::of(pin, disk)` and gets a person's machine, or names the console and
gets a watched one.
