# A printer is found, set up, and says what is wrong with it

**Date:** 2026-09-14
**Workstream:** v0.5 documents and paper — task 3 of
`docs/autonomy/v0-5-documents-and-paper-plan.md` (`ROADMAP.md`: **Printing**,
★ *Printers, solved — found, set up, and fixed when they stop*)
**Contributor:** Claude, as a worker in `C:\dev\alo-os`
**Status:** ready for integration — the code. Nothing here has printed on a
certified machine, and one behaviour of the rented printing service is
unmeasured (see *Owed*, first item).

## What changed, for a person

alo OS can now find the printers near a machine — on the office network and on
a USB cable — and set up the one a person picks without asking them about
drivers, queues or protocols. When a printer stops, the machine says which of
five things is wrong: out of paper, jammed, out of ink, it refused the document,
or it is not answering. Each comes with what to do about it, and a printer
reporting anything else is "not answering", with what the machine tried, never
a code. An assistant can print a document it was granted, after the person
approves the sentence. If the printer is across the network, the egress
indicator shows the document leaving before anything is sent.

## What changed, in the repository

New crate `crates/alo-printing`:

| File | What it holds |
|---|---|
| `src/ipp.rs` | The IPP message encoding (RFC 8010): write a request, read an answer defensively and with bounds |
| `src/http.rs` | One POST to the printing service and its answer, by length or chunks, bounded at 1 MiB |
| `src/service.rs` | `PrintingService`: the service's socket, or an address that must be loopback (`NotThisMachine` otherwise) |
| `src/reached.rs` | `Reached` (on this machine / across the network) and `Speaks` (driverless / needs its maker's program), from a device URI; `Reached::leaving` gives the `alo_egress::Leaving` |
| `src/printer.rs` | `Called` (the name a printer gave itself, or unshowable), the derived `Queue`, and `Printer` |
| `src/found.rs` | `find` (CUPS-Get-Devices) and `Found` |
| `src/setting_up.rs` | `set_up` (CUPS-Add-Modify-Printer with `everywhere`, then CUPS-Set-Default) and `CannotSetUp` |
| `src/stopped.rs` | `Stopped` and `Tried`: the closed set, read from `printer-state-reasons` |
| `src/asking.rs` | `how_is` (Get-Printer-Attributes), `this_machines_printer` (CUPS-Get-Default), `Condition`, `NoPrinter` |
| `src/document.rs` | `printable`: `alo_opening::decide` first; PDF, PNG, JPEG and plain text print |
| `src/printing.rs` | `print`: the verb carried out (Print-Job), and `NotPrinted` |
| `src/verbs.rs` | `print_document` |
| `src/words.rs` | 28 strings, each with a translator's note |

Elsewhere: `Cargo.toml` (member); `crates/alo-saying` collects the words
(`EVERY_LIST` 34) and lists CUPS in `EVERYTHING_WE_RENT` (16), as that file asks
of anything newly rented; `crates/alo-by-hand` is handed the verb;
`docs/by-hand.md` answers `print_document`; `docs/contracts/agent-verbs.md` has a
section for the verb and a row in the verb classes; the plan marks task 3 done.

## Decisions

**The printing service is spoken to, never run.** CUPS is the engine (ADR 0011).
`lpadmin` and `lp` would mean starting a program, which this product does not
do (`alo-bounding` documents why). So the crate speaks CUPS's own interface, IPP
over HTTP, on its own socket (`/run/cups/cups.sock`) or a loopback address.
That is about 600 lines in `ipp.rs` and `http.rs`, with no dependency added. A
test reads the manifest and the shipped source: no `std::process`, no
`Command::new`, no `UdpSocket`, and a connection is opened only in `service.rs`,
after the address was checked to be this machine's.

**Finding is CUPS's backends, not a second mDNS stack.** `alo-nearby` already has
one, but its wire code is private, and this plan reads that crate without
editing it. CUPS-Get-Devices asks the DNS-SD, IPP, USB, IPP-over-USB, socket
and LPD backends in one request. The rules are still `alo-nearby`'s: finding
grants nothing, a printer's name is data checked before it is shown (a newline
or an escape makes it unshowable, never cleaned), and finding nothing is an
answer.

**Driverless only.** A printer is set up from its own description of itself
(`ppd-name=everywhere`). A printer that needs its maker's program (`usb://`,
`socket://`, `lpd://`, a non-IPP DNS-SD service) is refused before anything is
sent, and the sentence says what kind of printer does set up by itself. No
driver is written and no installer is run. The queue name is derived from the
printer's name plus a 64-bit FNV digest of its address, so two printers of one
model never share a queue.

**One printer that works.** `set_up` makes the chosen printer CUPS's default,
and `this_machines_printer` reads the default back. So CUPS keeps the choice and
this crate keeps nothing that could disagree with it. `print_document` takes no
printer argument: a queue name in a sentence a person approves would be
machinery reaching them.

**Egress is an `alo_egress::Leaving`, not an `Errand`.** The plan says
*makes it an `alo_egress::Errand` so the indicator fires*. `Errand` is, by that
crate's own documentation, *egress with no agent behind it*: a closed list of
three reasons alo OS reaches the network by itself, with a tripwire test. Adding
a member would also edit a crate this plan may only read. A print from a verb
has an agent behind it, so it is `Leaving::because(agent, Why::Sending,
Destination::at(host))` on the **same indicator**. `print` refuses to send
anything to a printer across the network unless handed the `Departing` for
exactly that egress (same agent, `Sending`, same host). The indicator fires, and
the record can say whose authority it was under, which an errand could not.
The plan's intent, *the indicator fires*, is kept. Only the type differs, for
the reason above.

- **Consequence, stated rather than hidden:** under `EgressPolicy::NothingLeaves`
  (an organisation's `ThisMachineOnly`), and under `InTheBuilding` for an
  unpaired printer, an agent cannot print across the network. A person printing
  from the dialogue is not an agent's egress, and no type in `alo-egress` names
  person-caused egress today. Whether it should is a question for the integration
  owner, not a gap papered over here.
- A printer found through DNS-SD is counted as across the network whatever it
  resolves to. That can light the indicator once too often; the other mistake
  would be a silent departure.

**What "not answering" carries.** `Tried` is a closed set of three: asking this
machine's printing service (it did not respond), asking the printer (it said
nothing this machine can act on, which also covers door open, fuser, offline and
unknown keywords), and looking for its setup (the queue was removed). Warnings
and reports (`-report`, `-warning`, `-low`) are never stops. When several stops
are reported, the one said first is the one fixed first: a jam, then paper, then
ink.

**What an agent cannot do.** There is no verb to find or set up a printer, and a
test holds the list to one verb. `docs/features.md` says *the agent finds it,
sets it up*. That half needs a verb shape for a device: a grant covers a path or
an application, and a grantless verb needs a written reason in an ADR. It is
listed below as owed, not narrowed away, and the feature line stays unticked.

**Printing asks task 1 first.** `printable` runs `alo_opening::decide`. A file
whose name lies, a program, an empty or damaged file, or an office document
(converting is blocked on ADR 0039) is refused before a byte is sent, with
`alo-opening`'s sentence first and this crate's after it.

## Acceptance, and the test behind each clause

| Clause | Test |
|---|---|
| Finds printers on the local network and over USB | `finding_printers::printers_on_the_network_and_on_a_cable_are_found` |
| Sets one up without a person choosing a driver, a queue or a protocol | `finding_printers::a_printer_is_set_up_without_anybody_choosing_a_driver_a_queue_or_a_protocol` |
| Printing is an `alo_capability` verb | `printing_a_document::printing_is_a_verb_that_waits_for_one_approval_under_a_grant` |
| A printer not on this machine is an egress, and the indicator fires | `printing_a_document::a_document_to_a_printer_across_the_network_is_shown_leaving_before_it_is_sent` |
| …and nothing is sent without it (refusal) | `printing_a_document::without_the_indicator_nothing_is_sent_to_a_printer_across_the_network` |
| Which of a closed set is wrong, each with what to do | `a_printer_that_stopped::each_thing_that_is_wrong_is_said_with_what_to_do_about_it` |
| Anything else is *not answering* with what was tried, never a code | `a_printer_that_stopped::a_state_that_is_none_of_them_is_not_answering_with_what_was_tried_never_a_code` |
| Constraint: never added silently | `finding_printers::finding_a_printer_never_adds_one`, `verbs::tests::no_verb_an_agent_can_ask_for_adds_a_printer` |
| Constraint: no driver, no installer, CUPS unpatched | `printing_reaches_only_this_machines_printing_service::nothing_here_starts_a_program_or_reaches_past_this_machine`, `finding_printers::a_printer_that_needs_its_makers_program_is_refused_before_anything_is_sent` |

Other refusal paths tested: a set-up the printer does not answer, that the
account may not do (IPP 0x0403 and HTTP 401), or that the service does not
answer, with no default made; an approval for another verb; a grant revoked
after approval; a program named `invoice.pdf`; a printer refusing a format; a
printer out of paper while not accepting jobs; a service on another machine;
a default queue name that is not a plain name; malformed, truncated and
oversized answers. The tests use a printing service on loopback that decodes
each request with the crate's own decoder and records it, plus one over a Unix
socket.

## Verification

Run on the Windows gate host in WSL (Ubuntu), with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-88e6ebddb0cab76e`:

- `cargo fmt --all`: clean.
- `cargo clippy -p alo-printing -p alo-saying -p alo-by-hand --all-targets -- -D warnings`: clean.
- `cargo test -p alo-printing`: 33 unit, 22 integration (5 + 7 + 8 + 2), 0 failed.
- `cargo test -p alo-saying`: 63 + 4 + 1 passed.
- `cargo test -p alo-by-hand`: 27 + 13 passed.
- `cargo test -p alo-collected`: 8 + 11 passed.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-printing --no-deps`: clean.
- Each evidence test was run on its own before the handoff.

Not run: the full workspace suite (the supervisor runs it). No real CUPS: the
gate machine has `libcups2` and no `cupsd`, and installing one is shared
maintenance that needs an idle handoff.

## Owed

1. **Whether cupsd itself accepts `ppd-name=everywhere` on
   CUPS-Add-Modify-Printer. Unmeasured, and the first thing to check on a
   machine.** `lpadmin -m everywhere` may build the setup on the client side
   before sending it. If the scheduler answers *not possible* to the request
   this crate sends, the person currently reads *did not answer*, which would be
   wrong. The two alternatives each need a decision: CUPS-Create-Local-Printer
   (server-side driverless, but the queue is temporary), or `lpadmin` (starting
   a program, which needs an ADR). Whichever reality says goes in
   `docs/quirks.md`.
2. **Who may find and set up printers on a real machine.** CUPS's default policy
   probably puts CUPS-Get-Devices and CUPS-Add-Modify-Printer behind its admin
   group (unmeasured). The settings surface, or the privileged broker, has to
   run as a member. Meanwhile the refusal is said, not hidden
   (`CannotSetUp::NotPermitted`).
3. CUPS (and `ipp-usb` for IPP-over-USB) pinned in the image.
4. A real printer on a certified machine, over the network and over USB, and the
   stops measured against what a real printer reports.
5. The agent finding and setting up a printer: a verb shape for a device, which
   needs an ADR.
6. Offering `print_document` from `alo-turn`, with its record entry, and the print
   dialogue and printers pane in the shell.

## Proposed shared-document updates (for the integration owner)

- **CHANGELOG.md:** "Printers: alo OS finds printers on the network and on a
  cable, sets up the one you choose with no drivers to pick, and when a printer
  stops it says which of five things is wrong and what to do. An assistant can
  print a granted document after you approve it, and a document going to a
  network printer shows on the egress indicator first."
- **ROADMAP.md**, under ★ **Printers, solved** and **Printing**: `- [x] The code.`
  citing this report; `- [ ] On the machine.` with items 1–4 above.
- **QUEUE.md / STATE.md:** documents-and-paper task 3 done; task 4 (*"I can't
  open this file", said properly*) depends on 2 and 3, and 2 still waits on
  ADR 0039.
