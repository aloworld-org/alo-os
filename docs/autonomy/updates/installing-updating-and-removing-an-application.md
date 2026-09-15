# Installing, updating and removing an application

**Date:** 2026-09-15
**Workstream:** v0.5 — software and the web
(`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 1)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration. The on-machine acceptance — the rented tool
on a real machine — is pending, and why is under *What was not run*.

## What changed, for a person

A person can install an application from the places their machine installs from
— Flathub, or one their organisation runs — update it and remove it. While an
application is being fetched, the egress indicator says so in plain words
(*alo OS is installing an application from dl.flathub.org*). A new application
arrives allowed nothing: it has to ask for a folder, the camera or anything
else. An update is offered and waits: it is never applied while the application
is open, and it never restarts the computer. Removing an application ends
everything it had been allowed, and anything an assistant had been allowed to do
with it, at once. A place set up without checking signatures is refused, and so
is anything that arrives without a valid signature — in words, naming the place.
An assistant can propose installing an application; nothing is installed until
the person approves the sentence naming the application and the place.

## What changed, in the code

A new crate, `crates/alo-software`, one responsibility per file:

| File | What it holds |
|---|---|
| `src/source.rs` | `SourceName` (refuses anything that could be read as an option), `Configured` (what the rented tool says a place is), `Source` (its name, its network `Destination`, whether it checks signatures) |
| `src/bound.rs` | `Bound` — `Nobodys` or `Only { permitted, set_by }` — and `SetBy` (ADR 0016) |
| `src/enabled.rs` | `Enabled`: the places set up, under the bound, and `usable` — not enabled → outside the bound → not verified → nowhere to reach, in that order |
| `src/refusing.rs` | `NotDone`, every refusal a person reads, with `word` and `said` |
| `src/shown.rs` | `held_to`: an act only runs under the `Underway` for its own errand at its own place; `NotShown` (English, a code fault) and `Stopped` |
| `src/tool.rs` | `Tool`, the rented tool as seven questions and acts over checked values; `Failed` |
| `src/installing.rs` | `Wanted`, `installing` (asked before anything leaves), `install` (under the line), `Installed` |
| `src/updating.rs` | `looking_for_updates`, `offered` → `Offer` (no public constructor), `applying` (refuses an open application), `apply`, `Updated` |
| `src/removing.rs` | `remove`: the tool removes it, then every grant it held and every grant over it ends in the same call; `Removed` |
| `src/verbs.rs` | `install_application` (change, no grant, reason carried), `approved(&Authorised) -> Wanted`, `NotAnInstallation` |
| `src/asked.rs` | The exact argument lists the tool is started with: `--system`, `--noninteractive`, positional values after `--` |
| `src/heard.rs` | Reading the tool's answers: sources with their options, identifier lists, and why an act failed (signature failures first) |
| `src/rented.rs` | `TheRentedTool`: `/usr/bin/flatpak`, started directly with a cleared environment in the C locale |
| `src/words.rs` | Nineteen strings, each with a translator's note; none names the machinery |
| `tests/installing_updating_and_removing.rs` | The acceptance, against a stand-in tool that records every act it is asked |

Outside it:

- `crates/alo-egress`: `Errand` gained `InstallingAnApplication`,
  `CheckingForApplicationUpdates` and `UpdatingAnApplication`, with three words
  (ADR 0042 part 1); its tests count six.
- `crates/alo-keeping-up`: the three new errands join the exhaustive match in
  `Offered::heard` (refused, as every other errand is), and its test lists them.
- `crates/alo-record`: documentation that said *three reasons* no longer counts.
- `crates/alo-shell/src/egress_status_raster_tests.rs`: the indicator-row count
  is `3 * destinations + Errand::EVERY.len()` instead of `12`.
- `crates/alo-saying`: collects `alo-software`'s words (37 lists).
- `crates/alo-by-hand`: hands `alo-software`'s verb to the check;
  `docs/by-hand.md` answers `install_application`.
- `docs/contracts/agent-verbs.md`: *The installing verb*, the verb class row,
  and `alo-software` among the crates declaring verbs.
- `docs/decisions/0042-installing-an-application-is-an-errand-and-an-agent-only-proposes-it.md`.
- The plan: task 1 marked done; task 8 added.

## Decisions, and why

1. **Three new errands in `alo-egress`, although the plan does not own it.** The
   acceptance says installing and updating appear on the indicator *named as
   what they are*, and `Errand` is a closed list with no member for them. Using
   an existing member would make the indicator say something untrue; a second
   indicator is what law 1's design refuses. The change is additive (three
   members, three words) and ADR 0042 names it and every crate it touches, the
   way ADR 0040 did for `alo-capability`. It reaches `alo-shell` only through
   one test constant, and that crate's gate was run (below).
2. **Removing puts nothing on the indicator.** The plan lists removing among
   *errands that leave the machine*; it does not leave — the tool removes an
   application without reaching the network. A line for it would break the
   indicator in the other direction. ADR 0042 part 2; a test holds it.
3. **Removing ends grants over the application too**, not only those it held:
   an identifier removed today can be installed tomorrow from somewhere else,
   and an agent's grant to open *the* text editor was not a grant to open
   whatever next answers to that name. If the tool refuses the removal, nothing
   ends.
4. **Two steps around the indicator**, in `alo-keeping-up`'s shape, and each act
   holds the `Underway` to its own errand *and* its own destination. Every check
   before a line is shown is asked again under it, because the bound or the open
   applications can change in between.
5. **`install_application` requires no grant**, with its reason in ADR 0042 part
   3 and in the declaration, as contract rule 5 asks. It takes the place as a
   `name` argument so the sentence names it; the place is validated when it
   runs.
6. **The machine's installation (`--system`)**, because that is where places an
   organisation sets up live; polkit rules letting the signed-in person install
   there are the image's. Arguments after `--`, because `alo-applications`
   accepts an identifier beginning with `-` and the tool would otherwise read it
   as an option. The C locale, because failures are read to decide whether a
   signature was the reason.
7. **Signature checking is off only when the tool says `no-gpg-verify`**; options
   nobody recognises leave it on, which is the tool's own default.
8. **A place with no network address is refused** (a `file://` folder, an
   unknown scheme): an installation that cannot name where it leaves for cannot
   go on the indicator. Scope cut: installing from a local folder is not in this
   task.
9. **The organisation's bound is a value here, not yet a file.** ADR 0016 puts it
   in `/etc/alo/agentd.toml`, whose reader is `alo-agentd`, which refuses
   unknown sections and needs a `format` decision like `[questions]`. That is a
   contract change in another crate, so it is task 8 in the plan rather than a
   quiet edit here. Everything that decides with the bound, and the refusal
   naming who set it, is built and tested.

## Acceptance criteria and evidence

| Criterion | Test (`alo-software`, `tests/installing_updating_and_removing.rs`) |
|---|---|
| Installs from enabled sources, and arrives with no grants — the one list read after installing finds nothing new | `an_installed_application_arrives_with_no_grants` |
| Installing and updating are errands on the egress indicator, named as what they are | `installing_and_updating_are_on_the_indicator_named_as_what_they_are`, `an_act_under_another_line_reaches_nothing`, `removing_puts_nothing_on_the_indicator` |
| An update is offered, never applied while the application runs, and never restarts the machine | `an_update_is_offered_and_never_applied_while_the_application_is_open` |
| Removing ends its grants in the one list in the same act | `removing_an_application_ends_its_grants_in_the_same_act`, `removing_on_a_machine_without_an_agent_ends_what_the_application_held`, `a_removal_that_did_not_happen_ends_no_grant` |
| A source that fails verification is refused in words, never installed from | `a_source_that_fails_verification_is_refused_in_words_and_never_installed_from` |
| An agent proposes through a verb a person approves, never installs by itself | `an_agent_proposes_an_installation_and_never_installs_one_by_itself` |
| An organisation's source list bounds, refused out loud | `a_place_outside_the_rule_is_refused_naming_who_set_it` |

## Verification

Run on Windows Server 2022 through WSL 2 Ubuntu, with
`CARGO_TARGET_DIR=$HOME/alo-builds/alo-os-2-72aa7fda7f7de151` (the directory the
supervisor's gates build in), from
`/mnt/c/dev/alo-os-2`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-software -p alo-egress -p alo-keeping-up -p alo-record -p alo-saying -p alo-collected -p alo-by-hand -p alo-citing --all-targets -- -D warnings` — clean.
- `cargo test -p alo-software` — 44 unit, 12 acceptance, 1 doctest: pass.
- `cargo test -p alo-egress -p alo-keeping-up -p alo-record` — pass.
- `cargo test -p alo-saying -p alo-collected` — pass (the new words are
  collected, and no sentence or note names a rented component).
- `cargo test -p alo-by-hand -p alo-citing` — pass (the verb is answered by
  hand, and ADR 0042's citations resolve).
- `cargo clippy -p alo-shell --all-targets -- -D warnings` and
  `cargo test -p alo-shell` — clean and pass; the indicator raster test draws
  all 15 rows (nine agent lines, six errands) on its 3840×2160 picture.

### After the first handoff was refused

The supervisor's `rustdoc, warnings denied` gate (`cargo doc --workspace
--no-deps` with `RUSTDOCFLAGS="-D warnings"`) refused the first handoff on two
links, and the second worker fixed exactly those:

- `src/shown.rs` linked the crate-private `held_to` from its module
  documentation; it is now named in code quotes, not linked, because a public
  page cannot link to a private item.
- `src/verbs.rs` linked `crate::installing`, which is both a module and the
  function re-exported from it; the link is now `mod@crate::installing`, the
  module, which is where "arrives granted nothing" is explained.

Then, the same way, in WSL 2 Ubuntu against the supervisor's build directory:
`cargo doc --workspace --no-deps` (warnings denied) — clean; `cargo fmt --all
--check` — clean; `cargo clippy --all-targets -- -D warnings` across the
workspace — clean; `cargo test` for `alo-software`, `alo-egress`, `alo-record`,
`alo-keeping-up`, `alo-saying`, `alo-by-hand` and `alo-shell` — pass.

## What was not run

- **The rented tool on a machine.** No host this task ran on has it installed,
  and installing it is shared maintenance that needs an idle handoff
  (`SHARED_MAIN.md`). The argument lists and the output reading are
  unit-tested against the tool's documented shapes; **the on-machine
  acceptance** is: on a certified machine with Flathub set up system-wide,
  install `org.gnome.TextEditor` through `TheRentedTool` and see the indicator
  line; set a remote's `gpg-verify=false` and see `NotVerified`; look for and
  apply an update with the application closed and open; remove it and read the
  grants file. Where the real output differs from `heard.rs`, `docs/quirks.md`
  records it.
- **Wiring.** No surface in `alo-shell` calls this crate yet, `alo-turn` does not
  offer `install_application`, and `alo-agentd` does not read a software bound.

## Proposed changes to shared documents (for the integration owner)

- **CHANGELOG.md:** *Applications can be installed, updated and removed from the
  places a machine installs from. Installing and updating show on the egress
  indicator; a new application is allowed nothing; updates wait until the
  application is closed; removing one ends everything it was allowed; places
  that do not check signatures are refused. An assistant can propose an
  installation, which waits for your approval.*
- **ROADMAP.md (v0.5, Software):** *install sandboxed applications, update and
  remove them* — model and rented door built; on-machine acceptance and the
  organisation's bound in the machine description (plan task 8) outstanding.
- **QUEUE.md / STATE.md:** reference this report; note ADR 0042 and the
  `Errand` addition for lane A (`alo-egress`) and the desktop lane (one test in
  `alo-shell`).
