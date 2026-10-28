# The walk that reached the end of Demo 1 — 2026-10-28

**Machine:** qemu on the development PC, 4 GB, 2 CPUs, software rendering, no
graphics card. **Not a certified PC**: every claim below is about this virtual
machine, measured once, by hand, by a person pressing keys through qemu's
monitor. Nothing here runs again on its own — the checks that do are named
where they exist.
**Image:** built from `main` at `fd85c22a`, which holds #970–#979.
**Model:** `gpt-4o` at OpenAI, the owner's own key, typed in on the machine.

## How far it got

**Every step of `docs/walks/a-day-on-alo-os.md` was done**, in one sitting on
one machine, with one restart in the middle (below, *A reply goes nowhere*).
The first walk (2026-10-25) reached step 11; the sixth and seventh stopped at
14 and 15.

| Step | Result | What made it possible |
|---|---|---|
| 1–4 Boot, language, account, desktop | yes | — |
| 5 Home folders | yes, and **the home is a btrfs subvolume**: `ls -id /var/home/alo` reads 256 | #978 |
| 6 Put a file in Documents | yes, from alo's own terminal (`Ctrl`+`Alt`+`T`) | #974 |
| 7 Give alo Documents, until removed | yes, with Settings drawn over the open terminal | #977 |
| 8 The grant survives | yes — after a restart | — |
| 9 Add the provider | yes | — |
| 10–11 Ask; one sentence waits | yes; the card was readable with the terminal open behind it | #977 |
| 12 Allow once | yes: *Done.* — plain, so a place to return to **was** kept | #978, #979 |
| 13 The file changed | yes | — |
| 14 Open the record (`Super`+`R`) | yes: the rename, and each departure to OpenAI | #975 |
| 15 Undo | **yes**: `alo-back` listed the rename, asked for the folder's name, and put `march-invoice.pdf` back; what the folder held was kept beside it as `Documents.alo-was-here-…` | #978 |
| 16 Revoke | yes: *The grant was revoked. It has already stopped*, and the next question was answered *No folder has been granted* | — |
| 17 Egress | **not seen on screen** — see below | #976, #980 |

## Step 17, said precisely

The indicator was not photographed lit. A question with a 400-word answer was
photographed every two seconds; the one frame taken while *alo is working*
shows no line. **This machine draws its screen about 4.5 seconds behind**, so a
blank frame cannot tell *not drawn* from *too quick to catch*, and this walk
does not claim either.

What does stand is `crates/alo-agentd/tests/nothing_leaves_before_the_desktop_is_told.rs`
(#980), which **runs on every build**: through the service's production start,
a desktop watching what leaves holds a line naming the provider at the moment
the provider receives the question, and an empty list afterwards. It was made
to fail twice on purpose before it was trusted. Seeing the line on a screen
still wants a PC that draws in time.

## What the walk found, not yet fixed

1. **A reply typed into a finished task window goes nowhere.** The window says
   *alo is working* forever; no request starts (no `alo-asking@` unit in the
   journal), Escape does not close it, and the keyboard stays in it — so the
   terminal behind could not be reached until the machine was restarted.
2. **The terminal starts in `/`**, not the person's home, and is drawn with no
   window around it.
3. **The record does not say which grant a change ran under, or that the
   person approved it** — which step 14 of the walk's own plan asks for.
4. **An answer's Markdown is shown raw** (`**Header**`), and a long answer runs
   off the bottom of the window with no way to scroll.
5. **Wording and layout**: the conversation shows *a change is waiting for you
   to answer, as number 0*, and a long title wraps into the *alo task* label.

Each is being fixed on its own branch. Smaller things seen and recorded for
later: a fresh machine installs its applications for about an hour and says
nothing; the PIN field draws four dots however many digits are typed.

## Instrument faults, so nobody re-learns them

- **Keep the firmware's settings with the disk they describe.** A fresh disk
  booted with the previous disk's `vars.fd` stops at the UEFI shell. Copy
  `OVMF_VARS_4M.fd` afresh with every new disk.
- **A held change lapses.** Approving minutes after a proposal answers
  *ItLapsed*, correctly; approve while the card is fresh.
- **One early-boot freeze** after a hard stop — *Failed to fork off sandboxing
  environment for executing generators: Protocol error*, then *Freezing
  execution* — did not recur on the next boot. Recorded, not explained.
