# When alo asks for an application during a task

ADR 0143 Q10 names two places where a person lets alo use an application:
*in Settings, and on a surface the compositor draws when alo asks*. Settings
was built on 2026-11-15 (`the-settings-window.md`, *Giving alo an application,
from the terminal's own picker*). This note is the second place: the sheet
that appears **during a task**, when alo needs an application it has not been
given.

## What the owner decided, 2026-11-16

The coordinator put four readings to the owner, and the owner accepted all
four as proposed. Relayed by the coordinator, in these words:

1. *"This task" ends when the task ends, meaning its window closes or it is
   stopped. It shows in Settings while it lasts.*
2. *"This session" lasts until sign-out: a new additive grant kind, removed at
   sign-out and again at the next sign-in. Update
   `docs/contracts/grants-file.md` in the same PR.*
3. *"Remember" lasts until removed, labelled "Remember", and is recorded as
   not tied to a project until projects exist.*
4. *The sheet replaces the refusal at the default level. Only the person's
   press grants. The one action is retried and still needs its own approval
   card. "Not now" stays refused and recorded.*

And for the tests: *"Not now", a task-length grant gone the instant the task
ends, a session grant gone after sign-out and after a power-cut sign-in, and
an agent never able to make the grant itself.*

The frame the sheet is drawn from is `501:35539` *Settings detail / window*:
*Select an application window* / *Choose one window to grant access.* / *Only
the selected window is included. Press Escape to cancel without granting
access.*, with the durations *This task*, *This session* and *Remember for
this project* from the terminal settings frames (`500:29477`).

## What has to be true, end to end

- **The trigger is the daemon's, the grant is the person's.** When an
  application verb is refused for want of a grant at *ask me each time*,
  the daemon tells the person's shell which installed application the call
  meant and which task asked. **The agent is told only the refusal it is told
  today.** It never learns what is installed from it
  (`crates/alo-turn/src/naming_an_application.rs`), and nothing it sends
  can make a grant.
- **The sheet** is drawn by the compositor over the asking task's window: the
  application by its name, *This task*, *This session*, *Remember* and *Not
  now*. Escape is *Not now*.
- **A press grants**, through the one road Settings already uses
  (`alo_changing::Changing::granted` with a sealed pick), with:
  - *This task*: a grant the shell takes back the instant that task's window
    closes or the task is stopped;
  - *This session*: a new kind, *until sign-out*, which the shell removes at
    sign-out and removes again at the next sign-in, in case the machine lost
    power before sign-out. It is an additive change to the grants file;
  - *Remember*: *until I remove it* (an existing kind), not tied to any
    project.
- **Then the one action is asked again**, and it still waits for its own
  approval card. Making a grant approves nothing.
- **Not now** leaves the action refused. The refusal is in the record, as
  every refusal is.

## The owner's answers on the sheet, 2026-11-16

Four readings were put to the owner before the sheet was built. Relayed by the
coordinator:

1. ***This task* on the disk**: approved as proposed. It is written as an
   *until sign-out* grant, and the shell takes it back the moment the task's
   window closes or it is stopped. So a machine that lost power mid-task still
   clears it at the next sign-in. Its row in Settings reads *until this task
   ends*, and a *This session* row reads *until you sign out*.
2. **A grant is alo's, not one task's.** The grants know an agent, not a task.
   So while a *This task* grant lasts, alo may use that application in any of
   its tasks. The owner: ***"Yes, and say so plainly."*** The line under *This
   task* says *alo may use it in any task until this task's window closes or
   you stop it*.
3. **The words**: approved as written (`crates/alo-shell/src/the_application_sheets_words.rs`).
4. **When the agent pauses**: after any refused application verb, because the
   agent cannot tell why a call was refused and must not learn what is
   installed. The shell asks the daemon what the turn wanted. With nothing
   wanted, the turn goes on at once and no sheet appears.

### What was read off frame `501:35539`

| Drawn | Taken from | What was read off it |
|---|---|---|
| *Let alo use {application}?* | `501:35565` *Select an application window* | heading, at 25 |
| *alo asked to use {application}. Choose how long.* | `501:35566` | line, 16 under the heading |
| *This task*, *This session*, *Remember*, each in its own card with a line under it | `501:35567`, `501:35570` *Terminal / Settings choice*, and the lengths from `500:29477` | one choice per card, 16 apart |
| *Only {application} is included. Press Escape to say not now.* | `501:35573` | the note, 16 under the last card |
| *Not now*, then the navy *Let alo use {application}* | `501:35575` (160), `501:35577` (280), from the left, 12 apart | the actions, 16 under the note |

The sheet is 680 wide, the frame's width. It is never wider than the display
allows, and it is centred on the display as *Open with* and the GitHub sheet
are, with nothing behind it dimmed.

**A departure, for the next person to take to the design.** The note above
says the sheet is drawn *over the asking task's window*. It is centred on the
display instead. A task window can be on another display, or put aside, and
the two sheets the owner has already approved are both centred. The keyboard
starts on *This task*, the shortest length, and Enter grants the length it is
on through the navy button, which is ringed.

## State

*Updated 2026-11-16.* The first two pieces are on `main`; the last two are
in review. **Nothing reaches a person until all four have landed**, because
the sheet is raised only from a turn the agent has paused.

1. `alo-capability`: the *until sign-out* grant kind and the grants file's
   field for it — **on `main`** (#1314).
2. `alo-agentd` / `alo-protocol`: the person's shell asks which
   applications the turn wanted (`FromAPerson::Wanted`), never the agent —
   in review (#1320).
3. `alo-shell`: the sheet, the grant through `Changing::granted`, a task's
   grants taken back when it ends, and session grants cleared at sign-out and
   at sign-in — in review (#1321).
4. The pause and the retry: `alo-agent` pauses after a refused call for an
   application and sends the identical call again once given (#1325, the
   agent's half); the shell's half, which raises the sheet and tells the
   agent, follows once #1320 and #1321 are on `main`.
