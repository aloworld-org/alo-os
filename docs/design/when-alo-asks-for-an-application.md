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

## State

**Decided and not built.** Nothing in this note reaches a person yet. The
pieces it needs, none started:

1. `alo-capability`: an *until sign-out* grant kind (`Until`), and the grants
   file's additive field for it (`docs/contracts/grants-file.md`).
2. `alo-agentd` / `alo-protocol`: the *an application is wanted* message to
   the person's shell, carrying the identifier and the task, and never to
   the agent.
3. `alo-shell`: the sheet (drawing, keys, screen reader), the grant through
   `Changing::granted`, taking a task's grants back when the task ends, and
   clearing session grants at sign-out and at sign-in.
4. The retry of the one action inside the agent's turn. This touches the
   multi-step turn another lane owns, and is coordinated with it.
