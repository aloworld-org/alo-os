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

## The four laws

1. **Nothing leaves silently.** Every network egress an agent causes
   is visible at the moment it happens and afterwards in a record. On
   a machine sold on sovereignty the indicator is a feature, not a
   diagnostic. With a local model a working day produces **zero**
   inference egress, measured at the network boundary — and we
   publish the measurement rather than the promise.
2. **No verb runs an arbitrary command.** Every capability is an
   enumerated verb with typed, validated arguments. No `exec`, no
   shell, no script the model authored, no "advanced" escape hatch.
   A model that can write code that runs has escaped every other
   control in this repository, so this law is what makes the rest
   true rather than decorative.
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
- A user-readable line in `CHANGELOG.md`, written while the knowledge
  is fresh rather than reconstructed at release time.

**And no rushing.** A date never justifies a shortcut. When something
has to give it is scope — one printer, one certified machine, one
adapter — because a system built in a hurry is a system nobody can
trust, and being trustworthy is the only asset this product has. We
would rather ship less, later, than ship something whose guarantees we
cannot demonstrate.

## Standing rules

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
- **User-facing strings are externalized (i18n) from day one.**
  Hardcoded English is a bug in a European product.
- **Names are for strangers:** files, commit subjects and branches
  describe the subject matter. Release codes live in `ROADMAP.md` and
  commit trailers. Commit subjects follow conventional style —
  `type(scope): descriptive subject`.
- **One agent per working tree.** Concurrent editors on one checkout
  are forbidden; every agent commits with a distinct author so
  authorship is never ambiguous. The canonical checkout lives outside
  any file-sync folder — git and the remote are the only sync.

## Map

- `README.md` — what alo OS is, and what it is not.
- `docs/features.md` — the only list of what gets built.
- `ROADMAP.md` — the only order it gets built in, with exit gates.
- `docs/decisions/` — the ADRs. Start with 0001; it is the one an
  outside contributor must read before touching `alo-agentd`.
- `docs/contracts/` — the agent verbs and the adapter SDK: what other
  people build against.
- `docs/hardware.md` — the certified list, honestly maintained.
- `docs/quirks.md` — where reality and the specification disagree:
  driver behaviour, application automation, firmware.
- `SECURITY.md` — how to report something, and what is in scope.
- `CHANGELOG.md` — what changed, in words a person outside this
  repository can read.
