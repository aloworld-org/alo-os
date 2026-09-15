# What a fresh machine has, so it is not helpless

**Date:** 2026-09-15
**Workstream:** v0.5 — software and the web
(`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 2)
**Contributor:** Claude Code worker in `C:\dev\alo-os-2`
**Status:** ready for integration. Installing the list with the rented tool on a
real machine is still to do, as it is for task 1; *What was not run* says what
that check involves.

## What changed, for a person

A new machine comes with seven applications, so a person can get going before
installing anything: a web browser (Firefox), a file manager (Dolphin) and an
application that opens archives (Ark), a text editor (GNOME Text Editor), an
image viewer (Loupe), a document viewer (Papers) and a terminal (Ptyxis). Each
one is installed from the same place and in the same way as anything the person
installs later. Each is updated the same way, and the person can remove any of
them, the browser included. Each arrives with no permissions.

The terminal belongs to the person. No assistant can be given access to it, and
no assistant request can name it. This covers opening it, focusing it and
installing it. It also covers any verb added later. If someone tries to give an
assistant access to a terminal, they are told: *a terminal is yours alone —
whatever is typed into it runs, so no agent can be granted one.* An application
the person chose can still hand something to the terminal.

## Where this work came from, and what is new in it

The list and the refusal were built on 2026-09-15 and **held back unpublished**
on the branch `held/software-task-2-awaits-capability`, because the refusal is a
change to `alo-capability`, which this plan reads and does not own. The owner
answered that question in ADR 0043, accepted 2026-09-16 as option **C**, and gave
this plan that one additive change. This change is the held work rebased onto
current `main`, with three differences:

- the word's key is `capability.grant.a-persons-own`, as ADR 0043 part 2 names
  it, rather than the held branch's `capability.grant.persons-own`;
- the browser is pinned at **156.0**, which is what the place a fresh machine
  installs from published on the day this was verified again (the held branch
  said 155.0.1, its release a day earlier);
- the renumbering of task 1's decision from 0041 to 0042 is **not part of this
  change**: it was published with task 1 and is already on `main`. This change
  contains no ADR edit at all — ADR 0043 is on `main` as the owner accepted it.

## What changed, in the code

**`crates/alo-software`** (this plan's crate):

| File | What it holds |
|---|---|
| `shipped.toml` | The decided list, as data: `format = 1` and one `[[application]]` per role with `role`, `identifier`, `source`, `version`, `licence`. The installer plan reads this file |
| `src/role.rs` | `Role`, the closed list of seven roles, and `Role::EVERY` |
| `src/shipped.rs` | `Shipped::decided` / `Shipped::read`, `Pinned` (with `wanted()`, the same `Wanted` a person's installation is), `NotShipped`, `THE_LIST`, `WHERE_IT_IS` |
| `src/lib.rs` | The two modules and their re-exports, plus a new section of the crate documentation |
| `Cargo.toml` | `serde` and `toml`. As dev-dependencies: `alo-files`, `alo-finding`, `alo-measuring`, `alo-printing` and `alo-by-hand`, so the terminal test is run against every verb the machine ships |
| `tests/what_a_fresh_machine_has.rs` | The acceptance tests, against a stand-in for the rented tool |

**`crates/alo-capability`** (additive, exactly ADR 0043 part 2 and nothing else):

| File | What changed |
|---|---|
| `src/persons_own.rs` (new) | `A_PERSONS_OWN` (seven terminal identifiers) and `is_a_persons_own` |
| `src/grant.rs` | `GrantError::APersonsOwn`; `Grant::checked_for` refuses to grant an agent any application on the list |
| `src/grants.rs` | `Grants::permitting` refuses an agent's request for one of those applications before reading the list, so a grant written into the file by hand gives nothing |
| `src/call.rs` | `Call::permitting` refuses a call that names one of those applications in any argument, even when the verb requires no grant |
| `src/words.rs` | `PERSONS_OWN` (`capability.grant.a-persons-own`), with a translator's note; `EVERY_WORD` goes from 51 to 52 |
| `src/lib.rs` | The module, the re-exports, and a sentence beside *there is no grant to `/`* |

`Reach`, `Ask`, `Grantee` and `NotGranted` are untouched, as ADR 0043 requires:
no existing variant, signature or sentence changes, and nothing outside the crate
has to be edited to keep compiling.

**Documents:** `docs/contracts/agent-verbs.md` gains a section, *And no agent
reaches a terminal*, so an adapter author reads the rule where they read the
verbs; the plan marks task 2 done.

## Decisions, and why

### 1. The list, with the reasons for each choice

Every identifier, version, licence and verification status was checked against
the catalogue of the place a fresh machine installs from
(`flathub.org/api/v2/appstream/<id>`), and **re-checked on 2026-09-15 for this
publication**. The criteria were the plan's — licence, maintenance and
accessibility — plus translations into the 24 EU languages, which `CLAUDE.md`
makes a rule.

| Role | Application | Version | Licence | Why |
|---|---|---|---|---|
| Web browser | `org.mozilla.firefox` | 156.0 | MPL-2.0 | Published and verified by Mozilla itself. It uses an engine that is not Chromium, so the web keeps more than one engine. Its enterprise policies can turn telemetry off, which task 3 needs. Its accessibility tree is complete, and it is translated into every EU language |
| File manager | `org.kde.dolphin` | 26.04.3 | GPL-2.0-or-later | Verified and published by KDE, with trash built in. **GNOME Files (`org.gnome.Nautilus`) is not published there** (the catalogue answers 404), so it could not be installed the way task 1 installs things |
| Archives | `org.kde.ark` | 26.04.3 | GPL-2.0-or-later | Same developer and release as the file manager, and verified. It is a separate role because *archives that open* needs an application, and two applications under one role would give a list that cannot say which one is missing. File Roller was also considered: not verified, and far behind its own release |
| Text editor | `org.gnome.TextEditor` | 50.1 | GPL-3.0-or-later | GNOME core, verified, and built on GTK 4, whose accessibility works through the accessibility tree task 6 reads |
| Image viewer | `org.gnome.Loupe` | 50.0 | GPL-3.0-or-later | GNOME core and verified. It decodes each image in a separate sandboxed process, which is the right design for untrusted files |
| Document viewer | `org.gnome.Papers` | 50.2 | GPL-2.0-or-later | GNOME's current document viewer, verified. Evince is the one it replaced |
| Terminal | `app.devsuite.Ptyxis` | 50.1 | GPL-3.0-or-later | Verified, actively maintained, and GNOME's default terminal. GNOME Console (`org.gnome.Console`) is not published there either (404) |

Ark's catalogue entry states its licence in the older spelling `GPL-2.0+`; the
list writes the current SPDX spelling of the same licence,
`GPL-2.0-or-later`, which is what its own developer publishes.

**The browser, and the alternatives recorded.** Chromium
(`org.chromium.Chromium`) is packaged by volunteers rather than its developer, is
unverified, and would be a second Chromium-engine browser on a web that already
has too many. GNOME Web (`org.gnome.Epiphany`) is verified and GPL, but websites
work less reliably with its engine, and a fresh machine's browser has to open the
sites a person actually uses. Brave and similar browsers are run by companies
whose business depends on the browser. Choosing Firefox is not a partnership:
nothing is patched, rebranded or configured beyond what task 3 decides.

**Mixing GNOME and KDE applications** is deliberate. alo OS draws its own shell,
so neither toolkit is the house style. The choice follows what is published
upstream and can be installed.

### 2. "Pinned" means the release the list was decided at, not a commit the tool is held to

Task 1's `Tool::install` installs the stable build that a place currently
publishes. The rented tool can hold an application to a commit, but a version
number is not something you can install by. So `version` records the upstream
release this list was reviewed at. A newer release reaches the machine as an
offered update, like any application's update, which is what *updating it is no
different* requires. Commit pinning was not added: it would make shipped
applications update differently from everything else. On an installed machine,
the installer plan can compare the version the tool reports with this file.

This is exactly why the browser moved from 155.0.1 to 156.0 between the held
branch and this publication, and why that was a one-line data change rather than
anything in the code: the file records what the place published on the day, and
the machine catches up through the same offered update a person sees.

### 3. The list is a TOML file inside `alo-software`, not in `image/`

The installer plan needs data it can read, and `image/` is off-limits to this
plan. The file sits next to the crate that validates it, and a test checks that
the file on disk matches, byte for byte, the copy built into the crate. The
reader refuses unknown keys, so no `kept` or `removable = false` key can be
added. That is also how *a person may remove any of them* is held: nothing can
mark an application as protected.

### 4. The terminal is kept away from agents in `alo-capability`, not in this crate (ADR 0043)

Keeping `alo-software` from naming the terminal would still let
`open_application`, a later adapter verb (task 5) or the accessibility fallback
(task 6) reach it through a grant. Every one of those passes through the grant
check, so that is where the refusal goes. It works the same way as *there is no
grant to `/`*. Three places hold it: making the grant, checking a request against
the list, and checking a call. `NotGranted` gains no variant, because `alo-turn`
(a crate this plan does not own) matches on every one of them; the refusal a call
receives is `Never`, which is true — nothing the list holds covers the terminal,
and nothing it could hold would.

**The list is fixed and does not claim to be complete.** It holds every terminal
found by identifier on that place on 2026-09-15: Ptyxis, Black Box, Boxi,
Contour, Konsole, QMLKonsole and WezTerm. Refusing by what an application
declares itself to be (the `TerminalEmulator` category) is the right general
rule, and ADR 0043 records it as a follow-up for the applications plan, which
owns what an application declares. A test refuses any fresh-machine list whose
terminal is not on the closed list, so the shipped terminal cannot drop off it.

### 5. The shipped terminal can reach the host, and that is said rather than hidden

Ptyxis's own published permissions let it start a shell on the machine itself.
That is what a terminal is for: a sandboxed terminal that could see only its own
sandbox would be the toy `docs/features.md` rules out. It is installed through
the rented tool like everything else, so it is not an *unsandboxed installation*
in the v1 sense. But its sandbox should not be read as a wall, and ADR 0043 §3
and the comment in `shipped.toml` say so. This is also why no agent may be
granted it.

### 6. The word's key follows the ADR

ADR 0043 part 2 names the word `capability.grant.a-persons-own`. The held branch
had declared it `capability.grant.persons-own`. A key is a contract with
translators, so the ADR's spelling wins, and it is changed here before anything
is published under either.

### 7. Identifiers on the list are held more strictly than verb arguments

`alo-applications` accepts an identifier that begins with `-`; task 1 handled
that by putting `--` before arguments to the rented tool. The list refuses such
identifiers outright, because the installer plan reads the file too and no
upstream publishes an identifier like that.

## Acceptance criteria and evidence

| Criterion | Test |
|---|---|
| A decided list of six things (seven roles), each a pinned upstream application with its source identifier, version and licence, handed to the installer plan as the file it reads | `. alo-software what_a_fresh_machine_has the_decided_list_names_each_role_as_a_pinned_upstream_application` |
| Each is installed the way a person installs one, and updated and removed the same way | `. alo-software what_a_fresh_machine_has a_fresh_machine_installs_updates_and_removes_them_as_a_person_does` |
| A person may remove any of them, including the browser | `. alo-software what_a_fresh_machine_has a_person_may_remove_any_of_them_the_browser_included` |
| The terminal is a person's: a test holds that no verb reaches it | `. alo-software what_a_fresh_machine_has the_terminal_is_a_persons_and_no_verb_reaches_it` (every verb alo OS ships, with the list of verb-declaring crates checked against the workspace) |
| …refused when the grant is made | `. alo-capability lib grant::tests::there_is_no_grant_to_an_agent_over_a_terminal` |
| …refused when a grant was written into the list by hand | `. alo-capability lib grants::tests::a_terminal_written_into_the_list_by_hand_permits_an_agent_nothing` |
| …refused when a call names it, whatever its verb requires | `. alo-capability lib call::tests::a_call_naming_a_terminal_is_refused_whatever_its_verb_requires` |
| A fresh-machine list whose terminal an agent could reach, or which hides one under another role, is refused | `. alo-software lib shipped::tests::a_terminal_an_agent_could_be_granted_is_refused` |

Each of those eight was also run on its own with `--exact`, and each reported
`1 passed`.

## Verification

Run on Windows Server 2022 through WSL 2 Ubuntu, from `/mnt/c/dev/alo-os-2`,
with `CARGO_TARGET_DIR=/root/alo-builds/alo-os-2-72aa7fda7f7de151`. (Windows
itself cannot build this workspace here: a dependency's build script needs a C
compiler that is not installed, which is why every gate below is the Linux one.)

- `cargo fmt --all`, then `cargo fmt --all --check`: clean.
- `cargo clippy --all-targets -- -D warnings`, whole workspace: clean, finished
  in 2m 33s.
- `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-capability -p alo-software
  --no-deps`: clean.
- `cargo test -p alo-capability -p alo-software`: 162 + 8 capability tests,
  51 + 12 + 4 software tests and the doctests, all passing.
- `cargo test -p alo-citing -p alo-collected -p alo-by-hand`: pass — the
  contract's new link to ADR 0043 resolves, the new word is in a collected
  crate's list, and the verb-declaring crates are the six the terminal test is
  handed.
- The eight evidence tests, each on its own with `--exact`: each `1 passed`.

The full workspace suite was not run here, as instructed; the supervisor runs it.

## What was not run

- **The rented tool on a machine.** The same gap as task 1: no host this ran on
  has it. The check on a certified machine: install each `Pinned::wanted` through
  `TheRentedTool` from the place set up system-wide; confirm the version the tool
  reports matches `shipped.toml`; confirm Dolphin's trash and Ark's opening of a
  `.zip` work; remove Firefox and confirm nothing reinstalls it. Where the tool's
  real output differs, record it in `docs/quirks.md`.
- **The installer plan does not read the file yet.** That is its lane's work. The
  proposed text is below.
- **Wiring.** No surface in `alo-shell` shows the list yet.
- **Accessibility was judged from what each upstream publishes**, not measured
  with a screen reader on a machine. Task 6 is where that gets measured.

## Limitations

- The closed list of a person's own applications is by identifier and is not a
  proof of completeness: a terminal published after 2026-09-15, or from an
  organisation's own place, is not on it until somebody adds it. ADR 0043 records
  the general rule as a follow-up for the applications plan.
- `version` is a record, not a constraint the tool enforces; nothing yet compares
  the installed version with the file. The installer plan can.

## Proposed changes to shared documents (for the integration owner)

- **CHANGELOG.md:** *A new machine comes with a web browser, a file manager that
  opens archives, a text editor, an image viewer, a document viewer and a
  terminal. Each is installed, updated and removed like any other application,
  and you can remove any of them. The terminal is yours alone: no assistant can
  be given access to it or name it in a request.*
- **ROADMAP.md (v0.5, The ordinary desktop):** *a text editor, an image viewer, a
  terminal* — decided and held in tests (the list and ADR 0043); installing them
  on a machine is outstanding.
- **Installer plan (`docs/autonomy/v0-5-the-installer-plan.md`), for its owner:**
  *the fresh machine's applications are read from
  `crates/alo-software/shipped.toml` (`alo_software::shipped::WHERE_IT_IS`),
  `format = 1`; an image that names another list is refused.*
- **QUEUE.md / STATE.md:** task 2 of the software plan is done; reference this
  report, and note that ADR 0043's additive change to `alo-capability` has landed
  with it, so the applications plan's lane will see `persons_own.rs` in the crate
  it owns.
