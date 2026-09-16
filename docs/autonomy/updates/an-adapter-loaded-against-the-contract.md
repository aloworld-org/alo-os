# An adapter, loaded against the contract

**Date:** 2026-09-16
**Workstream:** v0.5 — software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 5)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration.

## What a person gets

An installed application can now become something the agent works with, the way
`docs/features.md` promises (★ *Application adapters — installed applications
become agents with typed verbs*). The first one is the text editor every fresh
machine has. An agent can ask to **open a document in GNOME Text Editor** or to
**open a new text editor window**. The person sees the sentence first — *open
/home/anna/Notes/march.txt in GNOME Text Editor* — and approves it once, and the
text editor does exactly that and nothing else.

Four things a person can rely on:

- **the agent reaches only what was granted.** The text editor must be granted to
  the agent, and so must the document's folder. An agent holding neither is not
  even offered the text editor's verbs;
- **nothing an agent writes runs.** An adapter that would hand the application a
  script, a command or free text is refused before the machine will load it;
- **everything is written down.** What ran and what was refused are in the record,
  in the words the person was shown;
- **nothing becomes agent-only.** Each thing an adapter can do comes with how a
  person does it in the application themselves, and an adapter without that is
  refused.

When it does not happen, the person is told why in plain words: the text editor
could not be reached, does not offer what was approved (usually a different
release), refused it, or did not answer. In that last case the person is told
that whether it happened is not known, rather than being told it failed.

## What changed

### The new crate, `crates/alo-adapters`

Eighteen files beside `lib.rs`, one subject each.

| | |
|---|---|
| `adapter.rs` | `Adapter`: the application, the releases it supports, the mechanism, its words and its verbs. Data, written as constants. |
| `adapter_verb.rs` | `AdapterVerb` and `Reaches`: the verb contract's six fields, plus `by_hand` and `carried_out`. `ONLY_ITS_APPLICATION` is the written reason for a verb that takes no path. |
| `adapter_arg.rs` | `AdapterArg`, `Kind`, `Offer`. The kinds include `Text`, `Script` and `Command` **so that a declaration can say it and be refused by name**. |
| `invocation.rs` | `Invocation`, `DBusMethod`, `Part`: the object, interface, method and a closed list of parameter shapes. The destination is never declared; it is always the adapter's own application. |
| `mechanism.rs` | `Mechanism`: `api`, `accessibility`, `dbus`, `synthetic`. |
| `loading.rs` | `load` and `Loaded`: every rule, then `alo_capability::Verb::checked`. |
| `becoming_code.rs` | The tripwires: methods named for running something, and the three methods whose first parameter chooses an action. |
| `bus_names.rs` | Well-formed object paths, interfaces and methods. |
| `not_loaded.rs` | `NotLoaded`: every refusal, in English with a `Display`, for the adapter author (the verb contract's rule for declaration-time errors). |
| `adapters.rs` | `Adapters`: one adapter per application; `declare_into`; `offered_to`, which offers only adapters whose application the agent holds. |
| `driving.rs` | `Driving` and `Driven`: from a redeemed `Authorised` to one message, asking again whether the application is granted and installed. Not `Clone`; `deliver` consumes it. |
| `message.rs` | `Message` and `Argument`: what an approval becomes, as data. |
| `file_address.rs` | A path as a `file://` address, byte by byte. |
| `delivering.rs` | `Delivers` and `NotDelivered`, with the words for each outcome. |
| `session_bus.rs` | `SessionBus` (Linux): sends the message and maps what the bus answers. The only file that touches the machine. |
| `text_editor.rs` | `TEXT_EDITOR`, the reference adapter, and its seven words. |
| `verbs.rs` | `shipped_adapters`, `adapter_verbs`, `declare_into`. |
| `words.rs` | Six sentences of the crate's own, plus every shipped adapter's words. |

### Elsewhere, each a registration a new verb-declaring crate owes

- `Cargo.toml`: the new workspace member.
- `crates/alo-saying`: collects `alo-adapters`' words. `EVERY_LIST`, `ONE_STRING_EACH`
  and the count test go from 43 to 44.
- `crates/alo-by-hand` (test and dev-dependency): the seven crates that declare verbs,
  with the two adapter verbs added to what the machine ships.
- `docs/by-hand.md`: `text_editor.open_document` and `text_editor.new_window`, each
  quoting `A text editor and an image viewer, so a fresh machine is not helpless`.
- `crates/alo-software/tests/what_a_fresh_machine_has.rs` (and its dev-dependency):
  the terminal test is handed every verb-declaring crate, so `alo-adapters` joins
  it. The adapter verbs take no application argument, so the terminal test's
  subject set does not change; an adapter *for* the terminal is refused at load.
- `docs/contracts/app-adapters.md`: an additive section, *How this machine loads
  one*. It covers the declaration's fields, the `adapter.verb` naming, `reaches`
  with the rule 5 reason, `by_hand`, the parameter list, every refusal, and what a
  person is told. Nothing existing changed.
- `docs/contracts/agent-verbs.md`: `alo-adapters` is named among the crates
  declaring verbs, and the *Adapters* row of the verb classes says where they run.
- `docs/autonomy/v0-5-software-and-the-web-plan.md`: task 5 marked done.

**Not touched:** `alo-capability`, `alo-applications`, `alo-portals`, `alo-granted`,
`alo-secrets`, `alo-egress`, `alo-access`, `alo-image`, `image/`, `alo-shell`.

## Decisions, and why

### Registering adapter verbs needed no change to `alo-capability`

The plan says the task stops at a decision record if it did. It did not:

- **The name.** `Verb::checked` already accepts dots in an identifier, so an
  adapter verb is `text_editor.open_document`. The prefix makes it unique across
  adapters, and `Adapters::add` refuses a second adapter of one name.
- **The grant over paths** is `Requires::grants_over`, like every verb. A path
  argument not listed there is refused at load (`OutsideItsGrant`).
- **The grant over the application** is not an argument, so `Call::permitting`
  cannot ask it. It is asked where the application is reached:
  `Driving::of` calls `Grants::permitting(Ask::Application)` at the
  authorisation's own moment, before anything is sent. This is the same place
  and order `alo_applications::Reaching` uses for installed applications, and
  for the same reason: an ungranted application's refusal is identical whether it
  is installed or not, and a test holds that. `Grants::permitting` also refuses a
  person's own application whatever the grants hold (ADR 0043). **To avoid a
  person approving something that would then be refused**,
  `Adapters::offered_to` offers an agent only the verbs of adapters whose
  application it holds a grant over.
- A verb that takes no path uses `Requires::nothing_because(ONLY_ITS_APPLICATION)`.
  That reason is written in the declaration and in the contract, as rule 5 asks.

### Declared in Rust, not read from a file at run time

"Loaded as declared data" is met by constants: every field is data, and nothing an
adapter declares is code the machine runs. A file read at run time would need
words that are not `'static`. `alo_strings::Word` is `&'static str` by design, and
the verb contract says a verb is declared **from `Word`s** so that the approved
sentence is the one a translator was handed. Loading signed adapter files at run
time is the v1 *adapter allowlist* and *SDK* lines in `docs/features.md`. When it
comes, it needs an `alo-strings` decision about words that arrive at run time, and
then a matching change in `alo-capability` (`Verb::checked` takes `Word`). **That
is the change this task would have had to stop at, and it is v1's, not v0.5's.**
Nothing in `docs/features.md` is narrowed by this: the v0.5 line promises
adapters, and the v1 lines promise the published SDK and the signed allowlist.

### What is refused, and why a type that could not say it is not enough

`alo-capability` already guarantees that no argument **carries** free text. An
adapter adds the other half: where a validated value **lands**. So `load` refuses
three ways a model's words could become code:

1. **an argument declared as `Text`, `Script` or `Command`.** These kinds exist so
   an author who writes them is told, by verb and argument, to split the verb;
2. **`Part::Evaluated`**, a parameter the application interprets. It is
   declarable so a declaration can be honest, and it is never loaded;
3. **a method that runs something, or an action an argument chooses.**
   `ActivateAction` with the action's name taken from an argument would be one
   verb whose meaning the model picks from everything the application registers,
   *quit* included. The three methods that take an action name
   (`org.freedesktop.Application.ActivateAction`, `org.gtk.Actions.Activate`,
   `SetState`) must name the action as a literal. The method-name word list is a
   **tripwire and says so**, exactly as `alo_capability::verb`'s is; review of a
   short, readable declaration is the boundary.

Also refused: `synthetic` (the contract calls it unauditable, and the plan puts it
at v1 and disabled by policy); `api` and `accessibility` until this machine
carries them out (a verb the machine cannot carry out is never offered); a verb
with no `by_hand` (ADR 0009); a word used and not declared (it would reach a person
as a key); an argument never sent (the person approved a sentence naming it); a
parameter of the wrong kind or from an undeclared argument; malformed names; no
release; no verbs; one name twice; one application with two adapters; and
everything `Verb::checked` refuses.

### Why GNOME Text Editor, of the seven applications a fresh machine ships

- **Its interface is its own, and is addressed by its own name.** A GNOME
  application holds its identifier (`org.gnome.TextEditor`) on the session bus.
  It answers `org.freedesktop.Application` on `/org/gnome/TextEditor` with
  `Open(as, a{sv})` and `ActivateAction(s, av, a{sv})`, and upstream registers
  `new-window` among its actions (`src/editor-application-actions.c`, read
  2026-09-16). The message's destination is therefore the application itself.
- **Dolphin** answers `org.freedesktop.FileManager1`, a shared interface under a
  shared name. Upstream `src/dbusinterface.cpp` registers that name, and its
  Flathub build is not allowed to own it (no `--own-name` in its finish-args, read
  2026-09-16). A sentence saying *in Dolphin* could be carried out by another file
  manager, or by none.
- **Firefox**'s automation interface evaluates script, which is exactly what this
  crate refuses. **Ptyxis** is a person's own (ADR 0043).
- The text editor's Flathub build has `--filesystem=host`, so a file address it is
  handed is one it can open.

Two verbs: `open_document` (a change, with a grant over the document) and
`new_window` (a change that reaches only its application). **Not quitting**: that
would close whatever the person had open, and no sentence about *quit* names what
that is. **Not saving or typing**: the interface offers neither as a typed method.

### An application that does not answer is recorded as run

The record has no *failed* shape, and three of the four outcomes are definite:
not there, does not offer, and refused all mean nothing was done. Those become
`Refused::worded_elsewhere` with the words the person reads, which follows
`alo_applications::Reaching`'s precedent for *not installed*. **Did not answer is
different**: the message reached the application under the approval and nobody
knows what happened. A record that left it out would record less than happened.
So it is `Driven { unanswered: true }`: it is recorded with `Entry::ran`, and the
person is told *whether it did what was approved is not known*.

## Acceptance, clause by clause

| Clause | Held by |
|---|---|
| Loaded as declared data: application, typed verbs with validated arguments, mechanism | `alo_adapters::Adapter`, `load`; `verbs::tests::the_shipped_adapters_load_and_their_verbs_are_named_under_them` |
| **Refuses any adapter whose verb takes a script, a command or free text that becomes code**, with a test that submits one | `an_adapter_whose_verb_takes_a_script_is_refused` (script, command, free text, a name handed to an interpreter); `a_method_that_runs_things_or_lets_an_argument_choose_the_action_is_refused` |
| Each adapter verb approved and recorded like any verb (ADR 0001) | `an_adapter_verb_is_approved_once_sent_once_and_recorded` (one approval, cannot be answered twice, one message, recorded with approval, grant and sentence); `every_refusal_of_an_adapter_verb_is_recorded` (7 refusals, each recorded, nothing sent after a grants refusal); `what_is_not_granted_is_neither_offered_nor_proposed`; `an_unanswered_message_is_recorded_as_run_and_said_as_not_known` |
| Has a by-hand road (ADR 0009) or is refused for lacking one | `a_verb_with_no_by_hand_road_is_refused`; `alo-by-hand`'s `every_verb_this_machine_ships_can_be_done_by_hand` now covers both adapter verbs |
| One reference adapter for an application task 2 ships, end to end, with its own automation interface | `the_reference_adapter_is_for_an_application_a_fresh_machine_ships` (identifier and release read from `shipped.toml`); `the_text_editor_receives_what_was_approved_and_it_is_recorded` (a real bus); `an_application_not_there_or_not_offering_it_is_refused_in_words` (a real bus) |
| If registering needs `alo-capability` to change, stop at a decision | Not needed. See *Decisions*; `alo-capability` is untouched |
| Constraint: contract additive; no screenshots and synthetic input | `app-adapters.md` gained a section and changed nothing; `screenshots_and_synthetic_input_are_refused_and_so_is_what_is_not_carried_out_here` |

## Verification

This host (Windows Server 2022) cannot build `ring`, which the test dependencies
reach through `alo-software` → `alo-proxy` → `ureq`, because it has no C
compiler. The gates were therefore run in WSL Ubuntu on the same working tree,
with `CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151` (this
checkout's own). The crate's Windows-only code path (`file_address.rs` on a
non-Unix host) was linted on Windows. All executed 2026-09-16:

| Command | Where | Result |
|---|---|---|
| `cargo fmt --all -- --check` | WSL | clean |
| `cargo clippy -p alo-adapters -p alo-saying -p alo-by-hand -p alo-software --all-targets -- -D warnings` | WSL | exit 0 |
| `cargo clippy -p alo-adapters -- -D warnings` | Windows | exit 0 |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-adapters --no-deps` | WSL | exit 0 |
| `cargo test -p alo-adapters` | WSL | 12 unit + 13 + 2 integration, all pass |
| `cargo test -p alo-by-hand` | WSL | 27 + 13, all pass |
| `cargo test -p alo-saying` | WSL | 63 + 4 + 1, all pass |
| `cargo test -p alo-software` | WSL | 75 + 12 + 5 + 8 + 4 + 1, all pass |
| `cargo test -p alo-collected` (reads the lists edited in `alo-saying`) | WSL | 8 + 11, all pass |

Each evidence line in the handoff was also run on its own with
`cargo test -p <crate> --test <target> -- --exact <name> --include-ignored`, and
each ran exactly one test and passed.

The real-bus test starts `dbus-daemon` from a configuration that names no service
directory, so nothing can be activated and the machine's own session is never
reached. If `dbus-daemon` is missing, the test fails rather than skipping. While
building it, a stand-in that held the name and served no objects produced no
reply at all. The call timed out after `HOW_LONG` (20 s) and came back as
*did not answer*, which is the road `session_bus.rs` maps it to. The committed
test serves a different object instead, so the bus answers *unknown object*
(*does not offer*) at once.

Not run: the full workspace suite, per the task instructions. The supervisor runs
it.

## What is not shown, and what would show it

- **GNOME Text Editor itself.** No machine this ran on has it installed. The
  on-machine acceptance is on a certified machine with the shipped Flathub build
  of `org.gnome.TextEditor` 50.x and a signed-in session. With the editor **not
  running**, approve `text_editor.open_document` for a granted file, and confirm
  that the bus starts the editor and it opens that file. Then approve
  `text_editor.new_window` and confirm one empty window. Then remove the grant over
  the application between approval and sending, and confirm the refusal and that
  no window appears. **Whether the sandboxed build is started by the bus** depends
  on its exported activation file, and this task has not seen that on a machine.
  If it is not, the first approval answers *could not be reached*, and the fix is
  a `docs/quirks.md` entry and a decision, not a guess here.
- **A turn does not offer adapter verbs yet.** Wiring `shipped_adapters` into
  `alo-agentd`'s offered list and executor is `alo-agentd`'s, as it is for printing
  and installing. This lane does not edit that crate (see task 8's hold).
- `api` and `accessibility` are declared and refused. Task 6 is the accessibility
  fallback, and it is a different thing from an adapter that uses the tree.

## Proposed changelog entry

> **Applications can become agents, starting with the text editor.** An adapter
> gives the agent a short list of things it can do in one installed application.
> The first is GNOME Text Editor: *open a document* and *open a new window*. Each
> is approved as a sentence, reaches only the text editor and documents a person
> granted, and is written in the record. An adapter that would let the agent hand
> an application a script or a command is refused before it loads, and so is one
> that offers something a person could not do themselves in the application.

## Proposed queue and roadmap updates

- `ROADMAP.md` v0.5, *Application adapters, and the accessibility fallback for
  applications without one*: the adapter half is built and shown on a real bus.
  The line stays unticked until the fallback (task 6) exists and the text editor is
  shown on a machine.
- `docs/autonomy/QUEUE.md`: task 5 done. Task 6 (*the accessibility fallback*) is
  unblocked. Add a follow-up for `alo-agentd`: offer `alo_adapters::shipped_adapters`'
  verbs to a turn and carry them out through `Driving` and `SessionBus`.
- `docs/autonomy/STATE.md`: reference this report.
