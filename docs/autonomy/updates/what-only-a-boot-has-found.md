# What only a boot has found

Date: 2026-10-11. Lane: the third PC. Status: a measurement, not a proposal.

Written because a second lane noticed the shape before I did, reading my own
reports back to me: *three faults today that only a boot could have caught is
a pattern about the gate rather than three anecdotes about pull requests.*

The count is five, and the gate was green for every one of them.

## The five

Each of these shipped past `cargo fmt`, `cargo clippy -D warnings`, the whole
test suite and CI, and was found by starting the machine.

| | what was wrong | how it was found |
|---|---|---|
| 1 | `getty@tty1` restarted the instant the greeter exited and drew a console login over a running desktop, taking the keystrokes meant for the sign-in fields | the screen, and `agetty: tty1: invalid character 0x9 in login name` — which was a Tab pressed at the greeter |
| 2 | the person had no home: `sysusers.d` declares one and `systemd-sysusers` writes a passwd line and never a directory | the desktop said so itself, every time, to nobody: *the canvas layout was not kept: Disk(PermissionDenied)* |
| 3 | nothing could make the first account; the greeter says *make one to sign in* and there was no road past it | three byte-identical frames at 4, 5 and 6 minutes |
| 4 | the serial console that made a boot readable also started `serial-getty@ttyS0` — an interactive login on a physical port, added by a change whose own note said it cost nothing | the audit trail of the first boot that carried the console |
| 5 | the journal stopped at 95 seconds: systemd writes unit messages to the console only until `systemd-journald` takes over, and everything this machine draws starts after that | reading that boot's log and finding the greeter absent from it |

Four of those are faults in what the image *is*. The fifth is a fault in the
instrument, found by using it — which is the ordinary way an instrument is
found to be wrong, and the reason it is on the list rather than filed
separately.

## What the gate cannot see, stated plainly

Every one of the five is a disagreement between two things that are each
correct alone:

- a unit and another unit that want the same screen;
- a declaration of a home and the tool that was supposed to make it;
- a sentence on a screen and the absence of a door behind it;
- a kernel argument and the getty systemd spawns because of it;
- a console and the point at which systemd stops writing to one.

A test runs one side. **The gate is a quality bar on each part, and the five
faults were all in the joins.** That is not a criticism of the gate — it is
what a gate is — but it means *green* has never been evidence that a machine
boots, and `ROADMAP.md` is right to keep that half of its line empty until
somebody has watched it happen.

## Two more, found by reading rather than booting

Worth recording beside the five, because they are the same fault class and a
different instrument caught them.

- **The card's path never painted the window edge layer.** `NativeLayers`
  grew an `edges` field and only `software_scanout` was taught to paint it;
  `scene_drawing`, which is where the card's road ends, named it zero times.
  Found by reading the two files beside each other, with a positive control.
  A machine with a graphics card drew no window edge at all and every test
  passed.
- **A unit naming `/dev/dri/card0`** — right on the one machine it was
  measured on, wrong on most UEFI machines, and defended in a comment rather
  than fixed. Found by a second lane reading a pull request description.

Neither was a boot and both were a comparison: two files against each other,
and a description against a reader who had not written it. **The common
element is not the boot. It is that something looked at two things that were
supposed to agree.** A boot is the cheapest way to make a hundred such
comparisons at once; a second reader is the cheapest way to make the ones a
boot cannot.

## What this is not an argument for

It is not an argument for a weaker gate, a slower gate, or a gate that boots
a virtual machine on every change. `docs/booting.md` already says what a
virtual machine cannot show, and four of the five above were found on one,
which is as much as that proves.

It is an argument for one habit, which cost nothing and found all five:
**boot the thing, and read what it says.** Before 2026-10-11 a booted alo OS
machine said almost nothing a reader could keep — the serial console that
fixed that is itself fault 4's parent, and the journal now reaches a file.

## Where the evidence is

- The boot logs and screenshots are kept on the third PC's build machine at
  `/root/boot-evidence/`, checksummed, with the serial log and the frames.
- Faults 1 and 2 are checked by `crates/alo-image`:
  `nothing_else_takes_the_shells_screen` and
  `the_person_has_a_home_of_their_own`, each with its refusal tested and each
  mutation caught by its own test.
- Fault 4 is checked by `no_unit_names_a_card_number`'s neighbour,
  `a_serial_port_that_offers_a_login_is_caught`.
- Fault 5 is checked by `a_console_that_hears_only_early_boot_is_caught`.
- Fault 3 has a door now — `alo_greeting::Greeting::makes_an_account` — and
  no caller; the surface that would call it is the compositor lane's.
- The edge-layer fault is checked by `both_painters_draw_the_same_layers`,
  which fails against the code as it actually was. It is named without a path
  on purpose: it lands with the fix rather than with this report, and a
  pointer written before its subject exists is the thing
  `a-pointer-to-a-file-that-never-existed` ruled on — *a quotation written in
  the form of a pointer is a pointer*.

Every one of those checks was written after the boot that found the fault.
None of them would have been written without it.
