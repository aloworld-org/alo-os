# Walk — a day's work on alo OS

**What this is.** The one walk that has never been done: every step below has
been proven somewhere, and **all of them have never been done in one sitting on
one machine by a person pressing the keys.** Until that happens there is no
demo, and the owner's standing goal — *a person can work on a PC with alo OS
all day, and a demo video can be made* — is not met.

**The rule for this walk: nothing is simulated.** No test harness approves on
the person's behalf, no key is typed by a script where a person would type it,
and no step is reported as working because its test passes. Where a step cannot
be done, the walk **stops there** and that is the finding.

## Before it starts

| | |
|---|---|
| machine | named in the report — a real PC, or qemu, and which one is said plainly |
| image | the pinned image, by its digest |
| provider key | the owner's own, read from the keyring the machine stores it in |
| network | watched at the interface, so egress can be counted |

## The steps, and what each one proves

1. **Boot.** The machine starts from the image and reaches a screen.
2. **Pick a language.** The language sheet appears before anything else, which
   is deliberate: typing presupposes the layout this screen chooses.
3. **Make an account.** Name and a PIN or password. *Proves: a fresh machine
   has somebody on it.*
4. **The desktop appears.** The Dock holds what this machine actually has.
   *Proves: the greeter handed the graphics card over — a named race.*
5. **The home folders are there.** Documents, Downloads, Pictures.
   *Proves #956, which landed 2026-10-25. Before it, a person's home was
   empty and there was nothing to grant.*
6. **Put a file in Documents** — any file, named something a person would
   recognise, e.g. `march invoice.pdf`.
7. **Settings → Give alo a folder → Documents → until I remove it.**
   *Proves #957 and #959 together: a grant can be made at all, and a grant
   with no end is on the list with the words "until you remove it" beside it.*
8. **Close Settings and open it again.** The grant is still there.
   *Proves it reached the disk, not just the screen.*
9. **Add the provider.** Settings → the answering section → name, endpoint,
   key → **test it before saving** → save → choose the model.
   *Proves the key is stored in the keyring and the choice is written.*
10. **`Super`+`A`, and ask for work in plain words.**
    *"Find the invoice in my Documents and rename it to invoice-march.pdf."*
11. **alo proposes one sentence and waits.** The sentence says what it will do.
    Nothing has happened yet. *Proves: reads answer, changes wait.*
12. **Press approve, once.** *Proves: one approval, exactly one execution.*
13. **The file is renamed, and you can see it.** Open Files and look.
    *Proves the thing no measurement in this repository had reached until the
    `the_agent_really_does_the_work` test: the agent actually does something.*
14. **Open the record.** It says what ran, under which grant, and who approved
    it. *Proves law 1 and the record.*
15. **Undo it.** The file is back to its old name.
16. **Revoke the grant in Settings.** Ask again. It is refused.
    *Proves a revoked grant takes effect on the very next question.*
17. **Read the egress count.** With a local model the day's inference egress is
    **zero**; with the owner's provider it is not, and the indicator showed
    every call. *Proves law 1 either way — the claim is the measurement, not
    the promise.*

## What this walk deliberately does not show

- **alo working inside an application** — Blender, a browser, WhatsApp. The
  accessibility fallback that would do it is built and tested against real GTK
  windows, and is **not on the list of verbs a model is offered**. That is
  Demo 2 and it is weeks away, not days.
- **Printing, converting a document, or searching.** All three are built and
  none is on that list either.
- **A second person on the machine.** The machine is wired to uid 1000.

## Where it stops being a demo and starts being a finding

Any step that cannot be done is written up the same hour, in
`docs/misreadings/` if something we believed was wrong, or as a plain gap in
the report if it was simply never built. **A walk that stops at step 11 is a
more valuable result than a walk that is quietly helped past it.**
