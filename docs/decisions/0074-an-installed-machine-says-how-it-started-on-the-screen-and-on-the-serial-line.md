# ADR 0074 — An installed machine says how it started, on the screen and on the serial line

**Status:** proposed, 2026-09-28. One question is asked of the owner at the end,
because it is about what ships to a person rather than what a walk can read.

Written by the development PC's lane, which owns `image/` and the installer.

## What forced it

`the_kept_computer_is_the_same_computer_twice` was read as a hang, twice, and it
was not one. Measured on 2026-09-27: the last line on the serial console was
OVMF's `PageFaultExitBoot` at `ExitBootServices`, then nothing for the full
480-second wait, one vCPU at 100%, no disk bytes across ten-second samples —
this repository's whole stall signature. The screenshots say the opposite. Both
rounds reached `fedora login:` on tty1 in about forty seconds, kernel
6.19.14-101.fc42.x86_64, network up, `alo-boundaryd` loading its programs.

**The machine was idle at a login prompt, and idle is indistinguishable from hung
when the console is silent.**

The cause is one line. The installer environment sets a console on its own
command line — `image/installing/grub.cfg` passes
`console=ttyS0,115200 console=tty0` — which is why every step up to and including
the install can be read. The **installed** system's boot entry carries none:

    options root=UUID=c583cbbc-… rw ostree=/ostree/boot.1/default/81fdfdcf…/0

So from the moment the firmware hands over, the serial line goes quiet and stays
quiet. A walk written to watch the installed system is waiting for a sentence
that cannot arrive, and `console.wait_for` against it can only ever time out.

## The decision in one line

**The install writes `console=ttyS0,115200 console=tty0` as a kernel argument**,
through `bootc install to-disk --karg`, so that an installed machine says how it
started on the screen *and* on the serial line if one exists.

## Why that order, and not the other one

The last console named takes `/dev/console`. With `tty0` last, a person's screen
is `/dev/console` and the kernel's messages appear on it; the serial line gets a
copy. Reversed, `/dev/console` would be the serial port and a laptop's owner
would watch their machine start in silence while the messages went to a socket
nobody is holding.

This is also exactly what the installer environment already does, so the machine
before the install and the machine after it behave the same way. That is worth
more than it sounds: the walks that read one and then the other stop needing to
know which half they are in.

On a machine with no serial port, `ttyS0` does not exist and the kernel drops it.
Nothing changes for that machine.

## Output is not a login, and the difference is the whole security question

`console=ttyS0` makes the kernel **write** to the serial port. It does not by
itself let anyone **type**. Typing needs a getty on that port, which is a
separate unit — `serial-getty@ttyS0.service` — and on this base it is started by
a generator when the kernel command line names a serial console.

So the two halves can be decided separately, and this repository is in a position
to decide them separately because it controls the image:

- **the output half** costs nothing and buys every walk that watches an installed
  machine, plus a person's ability to say what their machine did when it will not
  come up;
- **the login half** is a real surface. A serial console with a getty on it is a
  root-capable terminal for anything that can reach the port — which on a virtual
  machine is the host, and on a physical machine is whoever has the connector.

**Recommendation: take the output, refuse the login.** Write the console
argument, and mask `serial-getty@ttyS0.service` in the image so that naming the
console does not hand anybody a prompt. A person who wants one turns it on
deliberately, which is the same shape as every other capability in this system.

## What this does not decide

- **Whether a walk may rely on it.** It may read the serial line, but the
  acceptance for installer task 21 also requires the walk to assert the machine
  came up rather than printing that it did not. That is the walk's own work.
- **The screen.** Nothing here changes what is drawn; `console=tty0` puts kernel
  messages on tty1 alongside, which is what a Linux machine has always done.

## Rejected

- **`console=ttyS0` alone.** It moves the kernel's messages off the screen a
  person is looking at. Correct for a virtual machine, wrong for a laptop, and
  this is one image.
- **A `kargs.d` drop-in in the image instead of `--karg`.** Both are
  configuration rather than a patch, so ADR 0011 permits either. `--karg` is
  chosen because the installer is the thing that knows it is installing, and a
  drop-in would also apply to an image used some other way.
- **Leaving it and reading the screen.** The screenshots did answer the question
  on 2026-09-27 — by hand, after the fact, by a person looking at two `.ppm`
  files. That is not something a walk can assert, and every later walk that
  watches the installed system would inherit the same blindness.

## What is asked of the owner, because it is not ours

**May a machine alo OS installs carry a serial console at all?**

The output half is recommended without reservation. The login half is the
question: masking `serial-getty@ttyS0.service` keeps the port readable and mute,
and not masking it gives a root-capable prompt to anything that can reach the
connector. **Recommendation: mask it**, and let a person who wants a serial login
enable it themselves.

If the answer is that no shipped machine may name a serial console at all, then
installer task 21's constraint applies and the walks get their observability from
the screen asserted properly — and this record says so rather than leaving the
walks to time out.
