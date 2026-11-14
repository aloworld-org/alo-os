# Walks on GitHub, under KVM

A walk is somebody using a booted alo OS machine the way a person would, to
see whether the two ends of a feature meet. Until now every walk was done by
hand on the lane machine, where qemu had no KVM and emulated the processor:
first-boot installs took about 75 minutes, keys arrived 30 to 100 seconds late,
shifted keys came out garbled when sent quickly, and a screenshot taken after
a key often showed the screen before it. Each of those cost a wrong cause at
least once.

GitHub's hosted Linux runners give a job `/dev/kvm`, so the same disk boots
there at the processor's own speed. The **Walk** workflow
(`.github/workflows/walk.yml`) builds the image from `main`, writes a disk,
boots it and walks it with `crates/alo-walking`, with nobody at a keyboard.

## How to run it

On GitHub: **Actions → Walk → Run workflow**, on `main`. It takes about
MEASURED (below). When it ends, the run's summary page has a table of every
check, and the artifact **the-walk** has:

- one picture per look, numbered in the order taken (`01-first-start.png`, …);
- `report.txt` — every key pressed and every check, with the time on the
  machine's clock;
- `serial.log` — the machine's serial port, which carries its journal, with
  any secret the walk typed taken out.

The artifact is kept for **seven days** and then deleted by GitHub.

The disk is never uploaded, and the image is never pushed, signed or
published. It is built from `main` whichever branch started the run, because a
walk here is a walk of `main`. `crates/alo-walking/tests/the_walk_workflow_publishes_nothing.rs`
holds the workflow to all of that on every gate.

## What the first walk does

The keys below were read from the code they reach (`the_first_walk.rs` names
the files), not remembered.

| # | Does | Checks |
|---|---|---|
| 1 | waits for the journal to say *the sign-in screen is going up* | it is said within 20 minutes of power-on |
| 2 | Tab to Continue, Enter | the screen changes (a clock with no region reads as the United States, which has several zones, so this is the *which time zone* sheet) |
| 3 | Escape, then Enter | back on the overview, then on to the name |
| 4 | types `walker`, Enter; Tab ×3 to *Use password only*, Enter; types the password twice, Enter | each screen changes, and the journal says `alo-sessiond: a desktop is running` within 3 minutes of the last Enter — the session opens when the account is made |
| 5 | *How much can alo do*: Tab, Enter | the journal says `how much can alo do: …` within a minute |
| 6 | waits | the journal says *N of this machine's applications arrived, 0 still owed* within an hour of power-on — **and** the same line says *0 still owed* |
| 7 | Super+I, End, Tab | Settings opens, on Accessibility, with the ring on *Screen reader* |
| 8 | Space | the journal says `Started alo-screen-reader.service` within a minute of the press |
| 9 | watches two minutes | **no** line since the press says *strange signature*, *Traceback*, `alo-screen-reader.service: Failed` or `…: Main process exited` |

A look **fails when the screen did not change** within its time, and the walk
stops at its first failure: every check after it reads **NOT REACHED** rather
than being left out, so a short walk never reads as a walk that passed.

## The provider section, and the one secret it needs

After the screen reader, the walk can add an AI provider in Settings, typing
its key. That section is **off**: it runs only when the person starting the
workflow ticks *Also add an AI provider*, **and** the repository has the
secret. Without both it is skipped and the report says why.

To switch it on, the owner adds one repository secret:

- **Name:** `ALO_WALK_PROVIDER_KEY`
- **Value:** an OpenAI API key (the walk adds provider `openai`, address
  `https://api.openai.com`, model `gpt-4o-mini`) — ideally one made for walks,
  with a low spending limit, since every run uses it.
- **Where:** the repository's *Settings → Secrets and variables → Actions →
  New repository secret*.

The key is typed over QMP and never written to the report, which says only
how many characters were typed; GitHub masks it in the run's log; and the
serial log that is uploaded has every occurrence of it replaced. **Its keys
are unverified**: they are the walk by hand's of 2026-11-14, which opened
Settings fresh, and whether they reach the same rows from where this walk
leaves Settings has not been seen.

## What a pass proves, and what it does not

**A pass proves** that on that run, on that commit of `main`, the screens
changed when the keys were pressed and the journal said the sentences named
above, in time. Each check's evidence is in the report and the pictures.

**A look's pass is weak on purpose.** It proves the screen *changed*, not that
it changed to the right thing: the harness cannot read the screen. The
pictures are what show which screen it was, and a person reads them. The
journal checks are the strong ones, because each is a sentence only the
thing being checked writes.

**It does not prove the machine works on a PC.** This is still qemu: a virtio
display, a PS/2 keyboard and mouse, user networking, and a processor the
runner lends. CLAUDE.md's rule — *an integration test on real hardware for
anything that touches the machine* — stands, and nothing here satisfies it.
What it replaces is the walk by hand in qemu on the lane machine, not a walk on
a PC.

**It does not run by itself.** Under CLAUDE.md's *say whether a measurement was
performed once or runs forever*: the harness's own unit tests run on every gate
(they need no qemu); **the walk runs only when a person starts it**. Whether it
should run on every merge is the owner's call.

## Measured on the first runs
