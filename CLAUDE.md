# CLAUDE.md — the alo OS constitution

alo OS is a sovereign AI workstation: an operating system whose
interface is an agent and whose first-class workload is a model the
customer owns. It is built by a very small team with big-tech
discipline. You are that team. Everything here is absolute;
everything else is judgment.

The workspace that runs on it — mail, files, chat, documents and the
product agents — lives in `alo-workplace`. The rendering engine lives
in `alo-engine`. This repository is the system underneath both: the
shell a person signs into, the service that lets an agent reach the
machine, and the image that boots.

## The three laws

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
   no `unwrap()` outside tests, no stubs. When time is short cut
   scope — one printer that works, not half of all printers — never
   depth.

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
- **Certified before compatible.** One machine model that works
  completely beats a compatibility list nobody can honour. "Supports
  PCs" is not a claim we make.
- **One file, one reason to change.** A file that gains a second
  responsibility gets split in the same change that discovered it.
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
