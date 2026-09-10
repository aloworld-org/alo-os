# What a person signs in at — the decision task 10 turned out to be

**Date:** 2026-09-10
**Workstream:** v0.01 delivery (`docs/autonomy/v0-01-delivery-plan.md`)
**Task:** 10 — The image carries the shell, the session and the daemon
**Contributor:** Claude, in `C:\dev\alo-os-claude`
**Status:** ready for integration. **The task itself is blocked**, on the ADR
this change publishes, and the plan says so with the reason.

## What was asked, and what was there

Task 10 says the image should carry a shell to boot to, a session to sign in
to, and the daemon — and it accepts on a VM boot, a real local-model turn, and
an update that rolls back.

Before writing anything I asked what the repository actually has. Four answers,
each verified rather than remembered:

- **`crates/alo-shell` has no binary.** No `src/main.rs`, no `[[bin]]`, no
  `[[bin]]` anywhere outside `alo-agentd`, `alo-boundaryd` and the BPF target.
  The compositor is a library nothing starts. **There is no shell for an image
  to carry.**
- **Nothing authenticates anybody at a screen.** `crates/alo-accounts` is
  complete and is a library; lane B's own report says the entry surface stayed
  with the compositor lane. There is no session to sign in to.
- **The image boots to a text console.** `image/` installs two daemons, two
  units, two directories, two logins and one description, and nothing else.
  `alo-agentd.service` is `WantedBy=`/`BindsTo=`/`After=` `user@1000.service`
  — a session **nothing on the image can cause**.
- **A VM, a local model and an update channel are none of them reachable from
  this checkout.** That half of the acceptance is a machine question and stays
  one; `docker` exists here, `qemu` and `bootc` do not, and the image build
  pulls a Fedora base and two Rust toolchains.

So the task is not *hard*, it is **unstartable**, and it is unstartable for a
reason nobody had written down: **what runs before anybody is signed in has
never been decided.** Which unit the image enables, what `graphical.target`
pulls in, whether `alo-accounts` is the authenticator or a development
artefact, and what is allowed to turn a correct password into a `logind`
session — every one of those is downstream of one fork.

Per the standing instruction for exactly this case, **the decision became the
task.**

## What changed

### `docs/decisions/0024-what-a-person-signs-in-at.md` — new, PROPOSED

Three options, what each buys and costs, a recommendation, and the consequences
of accepting *and* of rejecting it.

- **Option A — rent a greeter and let PAM authenticate.** Buys the hard part
  (the session transition) as somebody else's solved problem. Costs the account
  model: `alo-accounts` either dies or becomes a PAM module, which is a C ABI,
  which is `extern "C"`, which is `unsafe` — forbidden workspace-wide. **Taking
  A means asking for that exemption, on the authentication path.** The ADR
  states that cost plainly rather than choosing for anyone; I may not weaken a
  gate and did not.
- **Option B — alo OS's own sign-in surface, PAM underneath only for the
  transition.** Recommended. alo OS owns the first screen — its words, its 24
  languages, its palette, its accessibility — and the account model that was
  built is the one that runs. Costs a **second privileged component** beside
  ADR 0018's loader, which is named as the price rather than smuggled in: it
  takes an already-authenticated uid, asks `logind` to open a session, and can
  do nothing else, with its capability set held by a `crates/alo-image` check
  beside the loader's.
- **Option C — rent a display manager whole.** Rejected and recorded as
  rejected, against ADR 0002.

The ADR is explicit about **what it does not know**: whether `logind` will open
a session for a caller that is not `pam_systemd`, on the pinned base. It says
that is the first thing the implementation measures and that `docs/quirks.md`
is where the answer goes, rather than assuming it. A decision document that
guessed there would be worse than no decision document.

### The one thing that could be finished without the answer

The ADR's recommendation rests on a premise: **first boot asks a person to make
an account, because the image ships none.** That was true by accident. It is
now a check.

- **`crates/alo-image/src/accounts.rs`** — new. `TheStore` reads whether an
  image ships the file a sign-in reads, by the path `alo-accounts` owns rather
  than by a second spelling of it. The name is looked at rather than followed,
  so a **symbolic link** where the store goes counts as shipped — the one
  disguise a listing of the image's files does not show, and the worse of the
  two, because `alo-accounts` refuses to open a link at all, so such a machine
  is one nobody can sign in to.
- **`crates/alo-image/src/checking.rs`** — a tenth check:
  `AnAccountShippedWithTheImage`. It is the only check in that file that passes
  by something being **absent**, which is exactly why it has to exist: a store
  committed into `image/etc/alo/` sits beside the machine description, is
  copied by the same `COPY` line, looks like the file that belongs there, and
  hands every holder of the image a login on every machine built from it. No
  build can see it.
- **`crates/alo-image/src/image.rs`** — the description's path is now derived
  from `crate::description::THE_DESCRIPTION` instead of being written out a
  second time, so `/etc/alo` is one string in this crate rather than two.
- **`crates/alo-accounts/src/place.rs`** — new, and the reason is one file, one
  responsibility. `THE_ACCOUNTS` lived in `keeping.rs`, which is `cfg(unix)`,
  so **where the store is** was out of reach of anything not running on a
  machine — including `alo-image`, the one crate that has to answer *does this
  image ship one* without being on a machine at all. *Where it is* and *who may
  have written it* are two questions; they are two files now.

### The plan

- Task 10 is **blocked**, with the finding, the ADR and this report named. It
  now also depends on the new task 13.
- **Task 13 — "A sign-in surface, and what starts it"** — written, blocked on
  the ADR being accepted, and marked as the desktop worker's: it is a binary in
  `crates/alo-shell`, which is that lane's. Its acceptance is written out,
  including the refusal paths (*make an account* on a machine with no store, a
  wrong password refused indistinguishably from an unknown name, the opener
  holding nothing else).
- **Task 11 no longer depends on 10.** That was an ordering rather than a need
  — an audit of `docs/features.md` against evidence reads code and reports, not
  a booted image. Left as it was, a blocked 10 and a scheduled 12 would have
  made the plan parse to *nothing executable*, which the loop reports as **the
  workstream being finished**. That would have been the plan's own named
  failure, caused by this change.

## Decisions I took, and why

- **I did not narrow task 10 to fit what I could build.** Its acceptance is
  untouched. Rewriting it to match a smaller deliverable would have moved a
  goalpost and left the VM boot unpromised.
- **I did not build the vocabulary and palette into the image**, which the
  task's prose names. Both are `[v0.5]` in `docs/features.md` — *wallpapers
  shipped with the image*, *the shell in the user's language* — and scope is
  gated to the current release. Shipping them would have been building outside
  v0.01 on a task that could not be finished anyway.
- **I recommended rather than decided.** The ADR is PROPOSED. Option B costs a
  second privileged component, which is an ADR 0018-sized decision and the
  owner's to accept.
- **`place.rs` rather than a second spelling.** The alternative was one line of
  `"/etc/alo/accounts.toml"` in `alo-image`. That is the exact drift that crate
  exists to catch, so the fix went into `alo-accounts`.

## Verification

Run in WSL Ubuntu from this checkout, `CARGO_TARGET_DIR=$HOME/target-claude`,
2026-09-10. Every gate the supervisor runs, run here first.

| Gate | Where | Result |
|---|---|---|
| `cargo fmt --all --check` | `.` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | `.` | clean, zero warnings |
| `cargo test --workspace` | `.` | 162 result lines, all ok; 0 failed |
| `cargo doc --workspace --no-deps` | `.` | clean |
| `cargo fmt --all --check` | `tools/kernel-loop` | clean |
| `cargo clippy --all-targets -- -D warnings` | `tools/kernel-loop` | clean |
| `cargo test` | `tools/kernel-loop` | 48 passed, 0 failed |
| `cargo fmt --all --check` | `crates/alo-bounding-kernel` | clean |
| `cargo clippy --release --target bpfel-unknown-none -Z build-std=core -- -D warnings` | `crates/alo-bounding-kernel` | clean |

The supervisor's own suite matters to this change and was not incidental:
`plan::tests::every_plan_this_repository_drives_holds_only_tasks` reads the
edited plan off the disk and would fail if the new task 13 broke its numbering.

**One environment note, and it was mine.** The first workspace run failed five
`alo-agentd` kernel tests on `cannot make a control group at
/sys/fs/cgroup/init.scope/home — File exists`. That was debris from a run I had
interrupted: the cgroup was empty (`populated 0`) and half-initialised
(`threaded`), and every later run collided with it. Removed with `rmdir`, which
can only succeed on an empty one, and the five passed immediately after. No
other worker's pins or processes were touched, and nothing in this change goes
near that fixture — a fixture whose interrupted run blocks every later one is
worth knowing about and is the kernel lane's to decide on.

Acceptance evidence, each run on its own with `--exact`:

| What it shows | Test |
|---|---|
| An image that ships the accounts a person signs in with is caught | `alo-image` `lib` `checking::tests::an_image_that_ships_an_account_is_caught` |
| The image this repository ships carries none | `alo-image` `lib` `accounts::tests::the_image_this_repository_ships_no_account` |
| A link where the store goes is shipped too | `alo-image` `lib` `accounts::tests::a_link_where_the_store_goes_is_shipped_too` |
| The store and the description are one folder, from two crates | `alo-image` `lib` `accounts::tests::a_sign_in_looks_where_the_description_ships` |
| The path is readable off a machine as well as on one | `alo-accounts` `lib` `place::tests::it_is_beside_what_the_machine_says_about_itself` |

**Not run, and not claimed:** any image build, any VM boot, any local-model
turn, any update or rollback. No *On the machine* box moves and none may be
ticked from this report.

## Limitations

- **The ADR is proposed, not accepted.** Task 10 and task 13 stay blocked until
  somebody accepts or rejects it. That is the intended state.
- **The session transition is unmeasured.** Whether `logind` opens a session
  for a caller that is not `pam_systemd`, on the pinned base, is the first
  measurement the implementation owes, and the ADR says so rather than
  assuming.
- **`alo-image` now depends on `alo-accounts`.** For one path, and by the same
  argument the crate already makes for `alo-keeping` and `alo-entering`. It
  compiles on every host: `alo-accounts` gates its `unix` half properly and
  `place.rs` is outside it. `alo-accounts` keeps a dev-dependency back on
  `alo-image`; cargo permits a cycle through dev-dependencies, and the
  workspace builds and tests clean.

## Proposed shared-document updates

For the integration owner — I did not edit these.

- **`CHANGELOG.md`:** *An ADR on what a person signs in at, and the first check
  that the image ships nobody's account.* alo OS's image can now be held to
  shipping no accounts file: a machine's first surface is *make an account*,
  not a login whose password every holder of the release knows. The decision
  about what draws that surface, checks the password and opens the session is
  written down as ADR 0024 and waits on acceptance.
- **`ROADMAP.md`:** no line moves. The image's *On the machine* box stays
  empty. If a note is wanted, the image bullet may say that the greeter and the
  shell binary are blocked on ADR 0024.
- **`docs/autonomy/QUEUE.md`:** task 10 is blocked on ADR 0024; a new item for
  ADR 0024's acceptance, and one for task 13 behind it, owned by the desktop
  lane.
- **`docs/autonomy/STATE.md`:** reference this report and the ADR.
