# Contributing to alo OS

alo OS is small, opinionated and early. Contributions are genuinely welcome —
and the fastest way to have one merged is to know what the project has already
decided.

## Before you write code

**Read these three, in order.** They are short, and they answer most review
comments before they happen.

1. `CLAUDE.md` — the three laws and the standing rules.
2. `docs/decisions/0001-the-capability-model.md` — what an agent may reach and
   under whose authority. **Required** if you are touching anything the agent
   can reach.
3. `docs/features.md` — the scope gate. Nothing gets built that isn't listed
   with a tier, in the current release.

Then check `docs/decisions/` for the rest. If you want to argue with a settled
decision, argue with the ADR and bring new facts — relitigating without them
wastes the scarcest thing this project has.

## The rules that get patches rejected

- **A verb that runs an arbitrary command.** No `exec`, no shell, no field that
  reaches an interpreter, no adapter that accepts a model-written script. This
  is the one rule with no exceptions and no "just for development" version.
- **A change that doesn't wait for approval.** Reads answer inside the turn;
  anything that changes the machine is proposed and waits for one tap.
- **Reach without a grant.** No path, application or device an agent can touch
  that a person did not deliberately grant.
- **A background reader.** Context arrives on invocation, never by watching.
- **Egress that doesn't fire the indicator.** And no telemetry, ever.
- **A language that isn't Rust.** Pinned engines are the exception, and they are
  configured, never written in.
- **`unwrap()` outside tests, `todo!()`, or a stub.** Done means the full path
  works, on real hardware.

## What makes a good contribution

**The gate is in `CLAUDE.md`** and it is not negotiable: zero warnings, the
refusal path tested as carefully as the happy path, the capability guarantees
as tests rather than prose, real hardware for anything touching the machine,
documentation in the same change, and a changelog line. There is no "tests in
a follow-up" — a change that ships without them has not been finished.

Beyond the gate:

- **Small and complete** beats large and partial. One verb, fully done —
  validation, refusal paths, record, test — is worth more than five sketched.
- **Take the time.** Nothing here is on a deadline that justifies a shortcut.
  If a change is getting away from you, cut its scope rather than its depth,
  and say in the pull request what you left out.
- **Say what you verified.** "Tests pass" is weaker than "ran it on the
  certified machine, and here is what the record shows".

## Adapters

Writing an adapter for an application is the most useful contribution available,
and it does not require touching the system. Read
`docs/contracts/app-adapters.md` — especially the rule that an adapter exposes
typed verbs and never accepts code.

## Commits and branches

Conventional subjects: `type(scope): descriptive subject`. Names describe the
subject matter — release codes live in `ROADMAP.md` and commit trailers, not in
branch names.

Commit with your own author identity. One agent, human or otherwise, per working
tree: concurrent editors on one checkout produce work nobody can attribute.

## Before you open a pull request

- Every line of the gate in `CLAUDE.md` passes.
- It is in `docs/features.md` with a tier, in the current release.
- **One file, one responsibility** (law 4). A file that grew a second reason to
  change gets split in the same pull request that discovered it — in this
  repository the file nobody wants to open is where the security bug lives.
- `CHANGELOG.md` has a line a person outside this repository can read.

## Security

Do not open a public issue for a security problem. See `SECURITY.md`.

## Licence

Contributions are under GPL-3.0-or-later, the licence of this repository. By
opening a pull request you confirm you have the right to contribute the code
under it.
