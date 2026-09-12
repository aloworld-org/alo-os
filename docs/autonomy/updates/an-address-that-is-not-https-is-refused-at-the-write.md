# An address that is not https is refused at the write, unless it is a service on this machine

**Date:** 2026-09-12
**Workstream:** v0.5, lane B — providers and models, task 1
**Contributor:** Claude (`C:\dev\alo-os-b`, kernel-loop worker)
**Status:** ready for integration

## What was wrong

`docs/features.md`, v0.5: *an address that is not https is refused rather than
warned about, unless it is a service on this machine — "it is only our internal
network" is how a key ends up on the wire in clear.*

The plan's premise — *today a provider's address is validated for shape and not
for scheme* — is half true, and the report says which half. `alo_models::Provider::checked`
has refused `http://` to anything but loopback since providers existed
(`ProviderError::InsecureEndpoint`, `crates/alo-models/src/provider.rs`, with
`address.rs` deciding what loopback is). What was missing was on the **write**
side, in `alo-choosing`:

- `alo_models::Provider` has public fields. A settings surface that builds one
  by hand, or copies a checked one and edits its address, holds a value nothing
  has judged — and `Choosing::adding` took it on the strength of a type whose
  name says *checked*.
- Such a provider was refused all the same, but by the writer's round trip
  (`crate::writing` reads its own text back and finds the reader will not take
  it). That surfaces as `NotWritten::NotExpressible`, whose sentence is *this
  alo OS could not write that choice* — a defect in alo OS. The person who typed
  `http://` in front of an address on their own network read about a fault in
  the machine and never the sentence telling them what to change.
- Nothing in `alo-choosing` tested any of it, in either direction.

## What changed

- `crates/alo-choosing/src/holding.rs` — new. `every_provider_holds` asks
  `alo_models::Provider::checked` again of every provider the settings would
  write. It names no scheme, no host and no exception; what an address is stays
  `alo-models`' one answer.
- `crates/alo-choosing/src/writing.rs` — `written()` asks it before serialising,
  and refuses as `NotWritten::NotAProvider` carrying `alo-models`' own reason.
  `written()` is the one function every door of `Choosing` goes through on the
  way to the disk, so there is no second path to the file.
- `crates/alo-choosing/src/choosing.rs` — a `changing(Provider)` door: the
  provider of that name, replaced where it stands. The acceptance names *adding
  or changing*, and the crate had no way to change one.
- `crates/alo-choosing/src/unwritten.rs` — `NotWritten::NothingToChange`, for a
  change naming a provider the list does not have. Refused rather than added.
- `crates/alo-choosing/src/words.rs` — one string, `choosing.change.nothing-to-change`.
  The https refusal itself adds **no** string: it is `alo-models`' sentence
  (`models.provider.insecure-endpoint`, *use https, or a service on this
  machine*), which `alo-saying` already collects.
- `crates/alo-choosing/src/lib.rs` — the module, and a paragraph in the crate's
  argument.
- `crates/alo-choosing/tests/an_address_that_is_not_https.rs` — new. One test
  per clause of the acceptance, on a real disk, against the vocabulary the
  whole machine loads.
- `crates/alo-choosing/tests/a_persons_choice_reaches_the_machine.rs` — the
  word count it pins, 15 → 16.
- `docs/contracts/person-settings.md` — *Writing it* gained the rule and the
  changing door, additively.
- `docs/autonomy/v0-5-lane-b-plan.md` — task 1 marked done. Task 2 was already
  written after it.

### User-readable change description

Adding a provider whose address is not `https://`, or changing one to such an
address, is now refused before anything is written — your settings file is byte
for byte what it was — and the refusal tells you what to do: use https, or a
service on this machine. A service at `127.0.0.1`, `::1` or `localhost` is the
one exception, on any scheme, because nothing travels a wire to reach it. An
address on your own network is not an exception. A provider can now be changed
in place, keeping its position in your file.

## Decisions, and why

**The rule lives at the writer, not at the doors.** The acceptance asks for
*the one place a provider is written*, so no second path can bypass it. A check
at `Choosing::adding` alone would hold for that door and be forgotten at the
next one; `writing::written` is the funnel every present and future door uses,
and the check is there. The doors document it; the writer enforces it.

**`alo-models`' rule is asked again, not copied.** `holding.rs` calls
`Provider::checked` rather than testing `starts_with("https://")` and a loopback
list of its own. Two answers to *what is this machine* would be the shape
`address.rs` was rewritten to remove (prefix matching that let
`http://127.0.0.1.attacker.example` pass). The integration test reads
`holding.rs` and fails if a scheme, a host, `starts_with`, `contains` or an
environment read ever appears in its code — which is also how *no allow-list
and no environment variable* is a test rather than a promise.

**The refusal is `NotAProvider`, not a new variant.** It already carries
`ProviderError` and already says it in `alo-models`' words. Rewording it here
would be two accounts of one moment, which `unwritten.rs` argues against.

**A `changing` door, with `NothingToChange`.** The plan says *adding or
changing*; the crate could only add. The door replaces by name (case-insensitive,
as `Providers::get` matches) and in place, so `[[provider]]` keeps the order
things were added in. A change naming a provider the list lacks is refused
rather than upserted: *change* and *add* are two different things a person did,
and a surface that added under a *change* button would be deciding what they
meant. That needs one sentence of its own, hence the new string.

**The reader is unchanged.** The constraint says a file written before this
rule is read as it always was. The reader has built every provider through
`Provider::checked` since format 2, so a file with `http://192.168.1.10` in it
was already refused whole on the way in, with `NotSet::NotAProvider`. It still
is, the file is never rewritten, and a test pins both. The refusal this task
adds is at the next write.

## Verification

Run inside WSL Ubuntu against this checkout, `CARGO_TARGET_DIR` the
supervisor's own directory for it (`/root/alo-builds/alo-os-b-72aa4fda7f7d8fc1`),
after touching changed files (the `/mnt/c` mtime quirk):

| Command | Result |
|---|---|
| `cargo fmt --all` | clean, no diff |
| `cargo clippy --all-targets -- -D warnings` (whole workspace) | clean |
| `cargo clippy -p alo-choosing --all-targets -- -D warnings` | clean |
| `RUSTDOCFLAGS="-D warnings" cargo doc -p alo-choosing --no-deps` | clean |
| `cargo test -p alo-choosing` | 125 unit + 37 integration tests pass |
| each evidence test with `--exact` | 1 passed, each |

The full workspace suite was not run here, per the task's instruction; the
supervisor runs it.

**Hardware:** nothing here touches a device. The tests write a real settings
file on the filesystem they run on; that is not the certified machine.

### Evidence, one line per acceptance criterion

All in `crates/alo-choosing/tests/an_address_that_is_not_https.rs`:

| Criterion | Test |
|---|---|
| Adding a non-https address is refused before anything is written; file byte for byte; sentence collected by `alo-saying` | `adding_an_address_that_is_not_https_is_refused_and_the_file_is_byte_for_byte_what_it_was` |
| Changing to one is refused the same way | `changing_to_an_address_that_is_not_https_is_refused_and_the_file_is_byte_for_byte_what_it_was` |
| Loopback on any scheme is the one exception | `a_service_on_this_machine_is_accepted_on_any_scheme` |
| The machine's own LAN is not an exception | `an_address_on_this_machines_own_network_is_not_an_exception` |
| Applied in the one place a provider is written | `no_door_writes_a_provider_the_writer_has_not_judged` |
| No allow-list, no environment variable | `nothing_here_lists_a_hostname_or_reads_the_environment` |
| A file written before is read as it always was, never rewritten | `a_file_written_before_this_rule_is_read_as_it_always_was_and_never_rewritten` |

## Remaining limitations

- `alo_models::Provider` still has public fields, which is why the writer has
  to judge again at all. Making them private with accessors would remove the
  by-hand path entirely; it is a change to a public type third parties may
  build against, so it is proposed as its own task rather than done inside this
  one.
- A proxy on `127.0.0.1` that forwards off the machine is still this machine to
  every type here — `docs/quirks.md`'s existing entry, caught at the network
  boundary rather than by an address check.
- `Choosing` still has no door to remove a provider. Not in this task's scope;
  the writer's whole-file shape already makes removal safe when a surface needs
  it.

## Proposed shared-document updates

**CHANGELOG.md**, under Unreleased / Settings:

> A provider whose address is not `https://` is refused when it is added or
> changed — before anything is written, with your settings file byte for byte as
> it was — unless it is a service on this machine (`127.0.0.1`, `::1`,
> `localhost`). An address on your own network is not an exception. A provider
> can now be changed in place.

**QUEUE.md / STATE.md:** lane B task 1 done; task 2 (*test a provider before
saving it*) is next and depends on it.
