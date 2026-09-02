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

- **Small and complete** beats large and partial. One verb, fully done —
  validation, refusal paths, record, test — is worth more than five sketched.
- **Test the refusal path.** Code that has only been tested when it works has
  not been tested. For anything the agent can reach, the "and it was stopped"
  test is the important one.
- **Documentation in the same change.** Rustdoc on public items; the contract
  updated if a surface moved; `docs/quirks.md` if you learned something about
  hardware or an application that the specification does not say.
- **Say what you verified.** "Tests pass" is weaker than "ran it on the
  certified machine, here is what the record shows".

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

- It builds with no warnings, and `cargo fmt` and `cargo clippy -D warnings` are
  clean.
- Tests pass, including the refusal paths.
- It is in `docs/features.md` with a tier, in the current release.
- The diff has one reason to change. A file that grew a second responsibility
  gets split in the same pull request that discovered it.

## Security

Do not open a public issue for a security problem. See `SECURITY.md`.

## Licence

Contributions are under GPL-3.0-or-later, the licence of this repository. By
opening a pull request you confirm you have the right to contribute the code
under it.
