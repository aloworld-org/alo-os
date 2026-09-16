# The accessibility fallback, for applications without an adapter

**Date:** 2026-09-16
**Workstream:** v0.5 — software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 6)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration.

## What a person gets

Most applications will never have an adapter. `docs/features.md` promises that
**they are still readable and operable through their accessibility tree** — the
same tree a screen reader uses. Two things are now possible, for an application
a person has granted:

- **the agent can read what the application's windows show**: the words, the
  controls, what is typed into ordinary fields, and whether each control is on,
  off or greyed out. It reads at that moment, for that turn, and keeps nothing;
- **the agent can ask to press one control**, named by what kind it is and the
  name it shows. The person sees exactly what will be pressed and where —
  *press the button named “Sign in” in org.example.Mail* — and approves it once,
  and that one control is pressed once.

What a person can rely on:

- **a password is never read.** The agent is told there is a password field, and
  what it is called, and nothing about what is in it — not even how long it is;
- **nothing is guessed.** If no control of that name is on the screen, or two
  are, or the window is too large to read whole, or the control is greyed out,
  nothing is pressed and the person is told why. Nothing is ever found or clicked
  by where it is on the screen;
- **what cannot be read is said.** A control with no name, a drawing the
  application paints itself, and a part of a window another program shows are
  each described in words as something the agent cannot read or press;
- **only what was granted.** Another application's windows are never read, even
  one that gives itself the granted application's name. An application with its
  own adapter is reached only through that adapter;
- **everything is written down**, what ran and what was refused, in the words
  the person was shown.

## What changed

### `crates/alo-adapters`, new files

| File | One responsibility |
|---|---|
| `fallback_verbs.rs` | The two verbs, declared: `accessible.read_window` (read) and `accessible.activate_control` (change), each requiring a grant over `application`. |
| `fallback_words.rs` | Every string the fallback says — 33 words, each with a translator's note. |
| `pressable.rs` | `Pressable`: the closed list of seven kinds of control a press can name, offered as a choice. |
| `role.rs` | `Role`: an `AtspiRole` number as one of fifteen kinds, with a password field a role of its own. |
| `states.rs` | `States`: showing, usable, on or off, from the tree's two state words. |
| `accessibility_tree.rs` | `AccessibilityTree`: everything the fallback may ask a tree — and nothing about position, and no subscription. |
| `walking.rs` | The walk: only the application its sandbox names, only what is showing, never a password field's text, bounded. |
| `not_walked.rs` | Why nothing was read, in words. |
| `shown.rs` | `Shown`, `ShownWindow`, `Seen`, `Contents`, `Limit`: what is handed back. `Seen::of` withholds a password field's text whatever it is handed. |
| `fallback_reach.rs` | The four questions asked before the tree is touched: this verb, granted, installed, no adapter of its own. |
| `reading_windows.rs` | `ReadingWindows` → `WindowsRead`: one authorised read, consumed. |
| `activating.rs` | `Activating` → `Pressed`: one approval, the control found again, one press. |
| `not_pressed.rs` | Why nothing was pressed, in words. |
| `accessibility_bus.rs` (Linux) | `AccessibleSession`: the session's accessibility bus, one plain method call per question. |
| `whose_connection.rs` (Linux) | Which installed application holds a connection, from its held process's sandbox. |

### Changed

- `crates/alo-adapters/src/lib.rs`: modules, re-exports, crate documentation.
- `crates/alo-adapters/src/verbs.rs`: `declare_into` also declares the fallback's
  two verbs, so `alo-by-hand` holds them to ADR 0009; `adapter_verbs` stays the
  adapters' alone.
- `crates/alo-adapters/src/words.rs`: collects the fallback's words; its
  *nothing names the machinery* test now covers them.
- `crates/alo-adapters/src/loading.rs`, `not_loaded.rs`: an adapter named
  `accessible` is refused (`NotLoaded::TheFallbacksName`).
- `crates/alo-adapters/Cargo.toml`, `Cargo.lock`: `alo-portals` as a Linux
  dependency, for `Sandboxes` and `HeldProcess`. It adds no crate to the tree.
- `crates/alo-adapters/tests/the_accessibility_fallback.rs`: the acceptance
  against a stand-in tree that logs every question.
- `crates/alo-adapters/tests/the_accessibility_fallback_on_a_real_application.rs`:
  the acceptance against real GTK windows on at-spi2's own bus.
- `docs/contracts/app-adapters.md`: an additive section, *The accessibility
  fallback, for an application with no adapter*.
- `docs/contracts/agent-verbs.md`: an *Accessibility fallback* row in the verb
  classes.
- `docs/by-hand.md`: `accessible.read_window` and `accessible.activate_control`.
- `docs/quirks.md`: three entries under *Application automation*, the section's
  first.
- `docs/autonomy/v0-5-software-and-the-web-plan.md`: task 6 marked done. Task 7
  is already written there.

**Not touched:** `alo-capability`, `alo-access`, `alo-portals`,
`alo-applications`, `alo-granted`, `alo-egress`, `alo-agentd`, `alo-shell`,
`image/`. `alo-saying` needed no change, because its count reads
`adapter_words()`.

## Decisions, and why

### The read is a read, and the press is what is approved

The acceptance says the two verbs are *each proposed as a sentence that names the
control and the application and approved like any change*. A read names no
control, and ADR 0001 §5 is explicit: **a read answers inside the turn**.
`alo_capability::Proposal::checked` refuses to put a read to a person at all
(`ProposalError::ReadDoesNotWait`). Declaring the read as a change so that it could
be approved would break the adapter contract's rule 1 (*be honest about effect*).
So:

- `accessible.read_window` is a **read**. It needs a grant over the application,
  which is the person's deliberate act, and ADR 0001's *cannot read the screen
  without invocation* is kept because it runs only inside a turn;
- `accessible.activate_control` is a **change**, and it is the verb the clause
  describes: its sentence names the kind of control, its name, and the
  application.

This reading does not narrow anything in `docs/features.md`. If the owner wants a
read of another application's window to wait for approval, that is a change to
ADR 0001 §5, not to this crate.

### Not an adapter, and not for an application that has one

The contract lists *accessibility* as an adapter mechanism. An adapter using it
would still be refused at load, as task 5 left it. The fallback is a different
thing: **two verbs this machine ships**, for applications nobody wrote an adapter
for. An application with an adapter is refused by the fallback. The text editor
adapter deliberately has no *quit* verb, and a generic *press any control* on the
same application would undo that decision. The verbs are named under
`accessible`, and loading refuses an adapter of that name, so no adapter verb can
share the prefix.

### An application is who its sandbox says, never who it says

An application's root on the accessibility bus carries a name the application
chose. `whose_connection.rs` does what `alo-portals` does for a portal caller, and
reuses its types rather than restating them:

1. asks the bus daemon for the connection's credentials;
2. holds the process by a descriptor (`HeldProcess`);
3. reads its `.flatpak-info` (`Sandboxes`);
4. counts the answer only while the process is still the one held.

A program with no sandbox is nobody. **That includes the desktop's own shell**, so
no agent can read or press the approval it is waiting for. The payroll window in
the real-application test is never addressed. In the stand-in test, an impostor
naming its window after the granted application is never asked anything.

Flatpak routes a sandboxed application's accessibility traffic through its own
`xdg-dbus-proxy`, run inside the application's sandbox. This is what makes the
held process's `.flatpak-info` the application's on a real machine. It is part of
the on-machine acceptance below, not shown here.

### A password field is its own role, never asked, and withheld twice

- **`Role::PasswordField` is not a kind of text field.** No match on text fields
  can reach it by accident.
- **The walk never asks it for its text.** This matters beyond principle: GTK 3
  answers with one bullet per character, which is the password's length
  (`docs/quirks.md`), and another toolkit can answer with the password itself.
- **`Seen::of` drops text beside a password field whatever it is handed.** A
  mistake in the walk is still not a password in the answer.

A mutation check was run: making the walk ask password fields for their text fails
both `a_password_fields_contents_are_never_asked_for` (the stand-in) and
`a_real_applications_password_field_is_never_read`. The real test caught it from
the bus monitor, naming the exact `GetText` call. The mutation was reverted.

### Nothing by position, nothing subscribed, nothing kept

- **`AccessibilityTree` has no question that answers or takes a position**, so
  nothing built on it can click a point. The real-application test also asserts,
  from the bus, that no `Component` or `Image` call is made by anyone.
- **Every call is a plain method call, never a proxy.** A zbus proxy asks the bus
  for signals. No event is registered, and the test asks the registry
  (`GetRegisteredEvents` is empty) and the bus (no `AddMatch`, no
  `RegisterEvent`).
- **An `AccessibleSession` is made for one turn.** The real test waits for its
  connection to leave the bus once the read is done.
- **A press walks again.** Nothing read earlier finds the control, and a control
  renamed in between is not pressed.

### Nothing is guessed

A press is refused, in words, for:

- no match;
- **more than one match** (two *Help* buttons);
- a window too large to read whole, even with one match, because *the only one*
  cannot then be known;
- a greyed-out control;
- a control with no action named `click`, `press`, `activate`, `toggle` or `jump`
  (matched without regard to case).

The action is chosen from that written list, never by the agent. An application
that does not answer the press is recorded as run and said as *not known*, exactly
as task 5 decided for adapters.

### Seven kinds of control, one name, no text

`kind` is a choice of button, check box, radio button, switch, menu item, tab and
link. Its words are translated into the sentence. `name` is `Takes::Name` of up to
200 characters: exact, with no path separators and no control characters. There
is **no verb that types**. Filling a field would be free text arriving in an
application, which the contract forbids an adapter and this fallback does not
reinstate.

### Bounds

A read covers at most 2,000 things, 48 levels deep, and 4,000 characters of any one
text. A node with more children than that is not asked for them one by one. Past
any bound, the answer says not everything was read.

## Acceptance, clause by clause

| Clause | Held by |
|---|---|
| Readable and operable through its tree, through two typed verbs | `the_two_verbs_are_a_read_and_a_change_and_take_nothing_else`; `a_window_is_read_within_its_grant_and_recorded`; on real GTK windows, `a_real_applications_password_field_is_never_read` (reads the form) and `a_real_control_is_pressed_by_its_kind_and_name` (the check box is ticked afterwards) |
| Activation proposed as a sentence naming the control and the application, approved like any change | `one_approval_presses_the_control_it_names_once_and_is_recorded` (sentence, one approval, cannot be answered twice, one press, recorded with approval and grant); `a_real_control_is_pressed_by_its_kind_and_name` (exactly one `DoAction` on the bus) |
| Read at invocation for that turn only, never watched | `with_no_invocation_the_tree_is_never_asked`; `the_control_is_found_again_at_the_moment_it_is_pressed`; from the bus, no `RegisterEvent`/`AddMatch`, empty `GetRegisteredEvents`, and the reading connection gone after the turn (`a_real_applications_password_field_is_never_read`) |
| A password field's contents never read, against a real application's password field | `a_real_applications_password_field_is_never_read` (GTK 3 `GtkEntry` with `visibility` off; no `Text`/`EditableText` call to it; no answer carries the password; `Contents::Withheld`); `a_password_fields_contents_are_never_asked_for` |
| What it cannot do — no name, a canvas — said in words, never guessed with coordinates | `what_cannot_be_read_is_said_in_words`; `nothing_is_guessed_when_the_control_is_not_the_only_one`; on the real form, the nameless button and the drawing area as `Limit`s, and no `Component` or `Image` call on the bus |
| Every refusal recorded, and nothing pressed | `every_refusal_is_recorded_and_nothing_is_pressed` (13 refusals, including not granted identically whether installed, revoked between approval and press, expired, not installed, another verb's authority, and every tree refusal); `an_unanswered_press_is_recorded_as_run_and_said_as_not_known` |
| Constraint: no screenshot, no synthetic pointer event, no coordinates; the tree is the rented AT-SPI's | No such question exists on `AccessibilityTree`; `AccessibleSession` speaks at-spi2's published interface; the real test runs against at-spi2's own launcher and registry |

## Verification

This host (Windows Server 2022) has no C compiler for `ring`, which the test
dependencies reach, so the gates ran in WSL Ubuntu 24.04 on this working tree.
The target directory was `CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151`,
this checkout's own. The real-application test needs `dbus-daemon`,
`at-spi-bus-launcher`, `broadwayd` and GTK 3's `gtk-builder-tool`, all present
there. **It fails rather than skipping** if any is missing. All runs were on
2026-09-16:

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p alo-adapters -p alo-by-hand -p alo-saying -p alo-software -p alo-portals --all-targets -- -D warnings` | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-adapters --no-deps` | exit 0 |
| `cargo test -p alo-adapters` | all pass: 18 unit, 13 + 2 (task 5), 11 + 2 (this task) |
| `cargo test -p alo-by-hand` | 27 + 13, all pass: both new verbs answered in `docs/by-hand.md` |
| `cargo test -p alo-saying` | 63 + 4 + 1, all pass |
| `cargo test -p alo-software --test what_a_fresh_machine_has` | 4 pass: the terminal test now also covers both fallback verbs, which take an application |
| `cargo test -p alo-portals --no-run` | compiles: `alo-portals`' dev-dependency on `alo-saying` now reaches `alo-adapters`, a cycle Cargo allows through dev-dependencies, as that crate's manifest already notes for `alo-keyring-fixture` |

Each handoff evidence line was also run on its own with
`cargo test -p alo-adapters --test <target> -- --exact <name>`, and each ran exactly
one test and passed. The real-application tests take about three seconds, and no
`at-spi`, `broadwayd` or `gtk-builder-tool` process is left behind afterwards.

Not run: the full workspace suite, per the task instructions. The supervisor runs
it.

## What is not shown, and what would show it

- **A GTK 4, Qt or browser application on a certified machine.** GTK 4 publishes
  no tree on Broadway (`docs/quirks.md`), so the real test uses GTK 3. The
  on-machine acceptance needs a signed-in Wayland session with Loupe (GTK 4,
  shipped) installed from Flathub and granted. Read its window and confirm its
  controls and title are there. Press a named button and confirm it acted. Then
  open a GTK 4 password dialogue — Papers' *unlock document* on an encrypted PDF
  is one — and confirm the field reads as a password field with nothing in it,
  with `dbus-monitor --address $(the accessibility bus)` showing no `GetText` to it.
- **Flatpak's accessibility proxy and the sandbox reading.** The on-machine run
  above should also confirm that the process behind a sandboxed application's
  accessibility connection is the proxy inside its sandbox, and is named by its
  `.flatpak-info`. If it is not, every sandboxed application reads as nobody and
  is refused. That would be safe, and it would be a `docs/quirks.md` entry and a
  decision, not a loosening here.
- **A turn does not offer these verbs yet.** Wiring `fallback_verbs` into
  `alo-agentd`'s offered list and executor, with `AccessibleSession::of_this_person`
  made per turn, is `alo-agentd`'s, as for adapters, printing and installing. This
  lane does not edit that crate (see task 8's hold).
- **The shell's own windows.** They are not a sandboxed application and are never
  reached, by design. Whether the approval surface also refuses activation from an
  accessibility client at all is the shell's question, not asked here.

## Proposed changelog entry

> **Applications without an adapter can still be used by the agent, safely.** For
> an application a person has granted, the agent can read what its windows show
> and ask to press one button, check box, tab or link by the name it shows. Each
> press is approved as a sentence naming the control and the application. A
> password field's contents are never read. Nothing is found or clicked by where
> it is on the screen, and if a control is missing, greyed out or not the only one
> of its name, nothing is pressed and the person is told why.

## Proposed queue and roadmap updates

- `ROADMAP.md` v0.5, *Application adapters, and the accessibility fallback for
  applications without one*: both halves are now built. The fallback is shown on
  real GTK 3 windows over at-spi2's own bus. The line stays unticked until the
  text editor adapter and the fallback are shown on a certified machine (see
  above).
- `docs/autonomy/QUEUE.md`: task 6 done, and task 7 (*every sentence, and the
  walk*) is unblocked. Extend task 5's follow-up for `alo-agentd`: also offer
  `alo_adapters::fallback_verbs` to a turn and carry them out through
  `ReadingWindows` and `Activating` over an `AccessibleSession` made for that turn.
- `docs/autonomy/STATE.md`: reference this report.
