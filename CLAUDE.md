# CLAUDE.md — the alo OS constitution

alo OS is a sovereign AI workstation: an operating system whose
interface is an agent and whose first-class workload is a model the
customer owns. It is built by a very small team with big-tech
discipline. You are that team. Everything here is absolute;
everything else is judgment.

The workspace that runs on it — mail, files, chat, documents and the
product agents — lives in `alo-workplace`. The rendering engine lives
in `alo-engine` — decided, but not started and not scheduled. This
repository is the system underneath both: the shell a person signs
into, the service that lets an agent reach the machine, and the image
that boots.

## The five laws

1. **Nothing leaves silently.** Every network egress an agent causes
   is visible at the moment it happens and afterwards in a record. On
   a machine sold on sovereignty the indicator is a feature, not a
   diagnostic. With a local model a working day produces **zero**
   inference egress, measured at the network boundary — and we
   publish the measurement rather than the promise.
2. **No code runs unless the person chose it.** Every capability on
   the broker's list is an enumerated verb with typed, validated
   arguments. No `exec`, no shell and no "advanced" escape hatch is
   ever added to that list, because a model that can write code that
   runs has escaped every other control in this repository. Running
   code is a separate grant that only the person gives, at one of three
   levels (ADR 0064): a sealed box the kernel locks down (the default),
   asking each time, or full trust. At every level each run is recorded
   and can be undone. The law binds the **agent**, never the person:
   alo OS ships a terminal, because a system that does not trust its
   owner with a shell is a toy.
3. **Done means the machine still works.** Input → validation →
   policy → execution → record → error paths, on real hardware. An
   OS that boots but cannot print is not a released OS. No `todo!()`,
   no `unwrap()` outside tests, no stubs, no "we will finish it in
   the next change". When time is short cut **scope** — one printer
   that works, not half of all printers — never depth.
4. **One file, one responsibility.** A file that gains a second
   reason to change gets split in the same change that discovered it,
   not noted for later. This is a law rather than a preference
   because of what this repository is: a privileged service, a
   compositor and a policy engine, where the file nobody wants to
   open is where the security bug lives. Small files are how the
   capability model stays reviewable by somebody who did not write
   it.
5. **The person chooses. alo never takes the choice away.** On their
   own machine, a person decides what alo OS does. Every protection is
   a default they can change, not a wall. Anything with a risk is
   offered with the risk in plain words and switched on by the person.
   It is never removed from them "for their own good". Two kinds of
   protection stay on at every setting, because neither takes a choice
   away: those that restrict nothing (the record, undo, the egress
   indicator, plain warnings), and those that guard the person from
   others (no telemetry, never a silent fallback, helpdesk only when
   invited). Only the law, or an organisation's policy on machines it
   owns (ADR 0016), may truly forbid. A change that takes a choice
   away from the person is a bug, whatever the reason given for it.
   The review question for every feature: *can the person choose this,
   knowing what it costs?* (ADR 0064)

## The gate — nothing is done until all of this passes

There is no "and we will add the tests afterwards". A change that ships
without them has not been finished, it has been abandoned.

- `cargo fmt` clean and `cargo clippy -D warnings` clean. **Zero**
  warnings — not "known warnings", not "warnings we live with".
- Unit tests for logic, and **the refusal path tested as carefully as
  the happy path.** For anything an agent can reach, "and it was
  stopped" is the test that matters. Code tested only when it works
  has not been tested.
- **The capability guarantees are tests, not prose.** Each of these
  is a test that runs in CI, and each of them is a promise we make in
  public:
  - with no invocation, `alo-agentd` makes no context calls at all;
  - a verb cannot reach outside its grant;
  - a revoked grant takes effect immediately, and an expired one is
    gone;
  - one approval causes exactly one execution;
  - every execution *and every refusal* leaves a record;
  - no agent-caused egress escapes the indicator.
- **An integration test on real hardware** for anything that touches
  the machine. A green suite on a developer's laptop proves nothing
  about a certified workstation, and this is an operating system.
- Documentation in the same change: rustdoc on public items, the
  contract updated if a public surface moved, `docs/quirks.md` when
  reality disagreed with a specification.
- A user-readable change description, written while the knowledge is fresh.
  Parallel contributors include it in their task report; the integration owner
  consolidates it into `CHANGELOG.md` under `docs/autonomy/SHARED_MAIN.md`.
- **Say whether a measurement was performed once or runs forever.** A proof
  somebody did by hand and a check that runs on every build are different
  things, and **a report cannot tell them apart unless the writer says which it
  was.** Both read as *measured* a week later. So a report that describes a
  measurement names where it will run again — a test, a build step, a gate — or
  says outright that it will not.

**A check that stands in for the thing is not the thing**, and this repository
has rediscovered that four times in four sets of clothes. It is written here
because it is cheap to catch in review and expensive to catch in a release:

- `test -x` on a converter binary stood in for the converter **starting**. Three
  releases shipped one that died at launch on eleven missing libraries, and no
  document of any format opened on a real machine.
- `gst-inspect-1.0` on an element name stood in for a file **decoding**. It still
  does; that gap is open and named in the recipe and in
  `crates/alo-image/tests/the_image_can_play_what_the_decision_says.rs`.
- A hand measurement in the pinned base — real, and correct — was written up as
  though the build did it. The claim was false the day it was made, in the same
  commit as the code it described, and stood for six days.
- A suite green on one machine stood in for a suite that passes anywhere. Three
  unrelated tests assumed they were alone on the machine; each failed a gate for
  a reason that had nothing to do with what it tests.

The question to ask of any green thing is **what would have to be true for this
to pass while the product is broken** — and then whether anything checks that.

**And no rushing.** A date never justifies a shortcut. When something
has to give it is scope — one printer, one certified machine, one
adapter — because a system built in a hurry is a system nobody can
trust, and being trustworthy is the only asset this product has. We
would rather ship less, later, than ship something whose guarantees we
cannot demonstrate.

## Standing rules

- **Task branches, one merge coordinator.** Follow
  `docs/autonomy/SHARED_MAIN.md`: branch checkpoint pushes may precede full
  gates; only an exact, fully gated combined tree may be squash-merged through
  a pull request into `main`. Keep legacy direct-to-main publishers paused.
- **One language: Rust.** The workspace above is TypeScript and lives
  in another repository. Here, a language that is not Rust is a bug.
  Pinned engines are the exception, and an engine is configured,
  never written in.
- **Engines are configured, never patched.** Linux, Mesa, systemd,
  the model runtime and the fine-tuning stack run as pinned upstream
  components behind our own interfaces. A source patch to any of them
  requires an ADR first.
- **The person grants; the agent never assumes.** No path, window,
  application or device is reachable that a person has not granted.
  Grants are enumerated, visible where the person can find them,
  revocable, and they expire. There is no grant to `/`.
- **Context is offered, never watched.** The focused window, the
  selection and the open document reach an agent only at the moment
  of invocation, and only for that turn. A background reader is a bug
  in this product, not a feature request.
- **Reads answer, changes wait.** A read runs inside the turn. Any
  change to the machine is proposed with a sentence describing it and
  waits for one approval. What a person approves is that sentence,
  and an approval is never a session.
- **Contracts outlive code.** The agent verbs, the application-adapter
  SDK, D-Bus interfaces, config keys, the image format and the update
  channel are public surfaces. Third parties build adapters against
  ours; they change additively, and a break requires versioning and
  deprecation.
- **The network is not authority.** Machines discover each other with no
  configuration and trust none of them for it. Using another machine
  takes a deliberate pairing on both; an agent reaching across acts
  only under a grant made on the machine it acts upon. There is no
  trusted-network setting, and there will not be one (ADR 0003).
- **Whose machine it is, is answerable in ten seconds.** A machine is
  personal, or it is managed by an organisation that sets policy and
  holds a recovery key — and on a managed machine the person is told
  so at first sign-in. There is no silent enrollment, and no
  administrator can watch a screen or act as a person (ADR 0004).
- **Certified before compatible.** One machine model that works
  completely beats a compatibility list nobody can honour. "Supports
  PCs" is not a claim we make.
- **Settled decisions live in `docs/decisions/`.** Read the ADR before
  proposing an alternative; relitigating without new facts wastes the
  scarcest resource we have.
- **Scope is gated.** Nothing gets built that isn't in
  `docs/features.md` with a tier, inside the current release, and
  outside Non-goals.
- **User-facing strings are externalized (i18n) from day one.** The
  first target is all 24 official EU languages, and any language
  somebody contributes after that. Hardcoded English is a bug, and
  "English plus the big five" is the same bug wearing a business case:
  it serves some people and not others, and the ones it skips are those
  with the least software in their own language already.
- **Built in Europe, not only for Europe.** Where alo OS lets an
  organisation set a rule — which region inference may happen in, which
  providers are permitted — the rule is theirs to name. We ship the
  mechanism, never a default that decides for them.
- **Describe the work, not its queue code.** New task titles, report filenames,
  commit subjects and status updates use professional, descriptive names such
  as "Secure file moves" or "Native desktop compositor". Legacy queue codes may
  appear only as secondary cross-references; preserve historical links and ADR
  identifiers. Use lowercase hyphenated report filenames such as
  `secure-file-moves.md`, never a code-only name.
- **Progress documents have one writer.** Follow
  `docs/autonomy/SHARED_MAIN.md`: parallel contributors publish separate task
  reports; only the integration owner edits shared progress documents. This
  supersedes older loop instructions requiring every contributor to edit them.
- **Names are for strangers:** files, commit subjects and branches
  describe the subject matter. Release codes live in `ROADMAP.md` and
  commit trailers. Commit subjects follow conventional style —
  `type(scope): descriptive subject`.
- **One agent per working tree.** Concurrent editors on one checkout
  are forbidden. The canonical checkout lives outside any file-sync
  folder — git and the remote are the only sync.
- **The author of every commit is the repository's owner**, from the
  checkout's own `user.name` and `user.email`. **No agent sets an
  author of its own** — not with `--author`, not with `-c user.name`,
  not through `GIT_AUTHOR_*`. This rule previously said the opposite,
  and the loop obeyed it: a run of commits landed authored by "alo
  build loop", which is not a person and is not who owns this work.
- **No agent adds a `Co-Authored-By` trailer either.** This rule also
  said the opposite until 2026-09-04 — an agent was to be named where
  a contributor belongs — and every commit in this repository carried
  one. The owner asked for the work to stand as theirs, and the
  history was rewritten to match: 119 commits, not one byte of any
  tree changed. **What an agent did belongs in the commit body**,
  which is where it was always the more use to a reader — a trailer
  records that an agent was present, a body records what it decided
  and why, and only one of those is worth reading a year later.
  Ambiguity between agents is solved by one agent per tree, never by
  inventing an author and no longer by a trailer.

## Map

- `README.md` — what alo OS is, and what it is not.
- `docs/features.md` — the only list of what gets built.
- `ROADMAP.md` — the only order it gets built in, with exit gates.
- `docs/decisions/` — the ADRs. Start with 0001; it is the one an
  outside contributor must read before touching `alo-agentd`.
- `docs/contracts/` — the agent verbs and the adapter SDK: what other
  people build against.
- `docs/hardware.md` — the certified list, honestly maintained.
- `docs/booting.md` — how the image becomes a disk a machine boots from,
  and what a virtual machine can never show about one.
- `docs/quirks.md` — where reality and the specification disagree:
  driver behaviour, application automation, firmware.
- `SECURITY.md` — how to report something, and what is in scope.
- `CHANGELOG.md` — what changed, in words a person outside this
  repository can read.
- `docs/autonomy/` — the build loop: `LOOP.md` is how one iteration
  works, `QUEUE.md` is the work and what each item is blocked on,
  `STATE.md` is the journal. The loop never weakens the gate to pass
  it, and never ticks what it did not finish.
