# Signing in to a proxy that asks who you are

**Date:** 2026-09-20
**Workstream:** v0.5 software and the web — `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 12
**Contributor:** this development PC's software-and-the-web lane
**Status:** ready for integration

## What a person gets out of this

On a great many company networks there is no other route out, and a good many of
those proxies ask who you are. Until this change alo OS read the name and the
keyring entry an organisation wrote into `[proxy]`, carried them as far as
`alo_proxy::ProxyAddress` — and then **no road in the workspace ever fetched the
password**. Every road reached the company's proxy as somebody with no password
and was refused by it, in the proxy's own words, at the far end of a connection.

Now the four roads alo OS itself takes — installing and updating applications,
the system's own update, a provider's list, and a turn's question — sign in with
the password the machine was given. A machine that has not been given it
**refuses the road in words a person can act on**, saying nothing was sent and
to ask whoever manages the machine. It does not knock on the proxy's door with
nothing to give it, and it does not go round the proxy either.

## The decision, written first

`docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md`, accepted
2026-09-20. The task's own constraint required it before any code: *a store
chosen inside a crate would be a store four roads then have to agree with.*

**Three questions, three answers.**

- **Which store.** The machine's own credentials — `systemd-creds` — read at
  `/run/credentials/<unit>/<entry>`, where the entry is the name
  `password-in-keyring` already gives. Encrypted at rest with a root-only host
  key or the TPM, decrypted by PID 1 at unit start, delivered to the unit as a
  file only its own user may read.
- **Whose.** The machine's, set by whoever administers it: an organisation on a
  machine it manages, the person on their own machine acting as their own
  administrator. ADR 0016's split, unchanged.
- **What a road does when nobody has signed in.** It takes the road. That is the
  property the store was chosen for.

**Why not ADR 0022's store**, and this is the whole argument: a *provider's* key
is the person's and is wanted inside a turn with them signed in; a machine-wide
proxy password is the organisation's and is wanted at `multi-user.target` by the
unit that asks whether there is an update, with `/run/user/<uid>` not yet made.
ADR 0022 records that state as **unavailable**. A machine-wide credential kept
in one person's keyring is also the wrong answer to *whose is it* even where it
would work. ADR 0022 itself names `systemd-creds` and rules it out with the
sentence that selects it here — *it is for credentials an administrator
provisions to a unit* — which is exactly what this is.

**Nothing about ADR 0022 moves.** A provider's key stays in the Secret Service
on the person's bus; neither store reads the other's; no credential moves
between them; and no credential-transfer protocol was added.

## What changed, file by file

### `crates/alo-proxy` — the one door and the reader

- **`src/signing_in.rs`** (new). `alo_proxy::signed_in(way, passwords)` is the
  one door a proxy credential travels through, and it is what makes
  `Carried::with_the_password` keep a single caller. `WhereThePasswordsAre` is
  the trait a store implements — the shape `TheEvaluator` already has in this
  crate. `NotSignedIn` is seven states; `WhatIsWrong` is the three things an
  administrator does about them.
- **`src/provisioned.rs`** (new). `TheMachinesPasswords::given_to(unit)` builds
  `/run/credentials/<unit>` and reads the entry. It **believes nothing it
  finds**: the name must be one thing to look up (no separator, no `..`), what
  is there must be a regular file, nobody but its owner may read it, it must be
  shorter than `LONGEST_PASSWORD`, and what it holds must pass
  `Password::typed`. The doc comment says plainly that systemd's behaviour is
  documentation rather than something this repository measured, and that the
  checks exist so nothing has to be withdrawn later — ADR 0022 had to withdraw a
  paragraph of libsecret's.
- **`src/words.rs`**: three new sentences. Three, not seven, because what a
  person *does* is three things — the machine has not been given the password,
  the password it has is readable by others, or the password it has cannot be
  used. Which of the seven it was is kept for whoever administers the machine
  and never shown. None of them names any machinery; the existing test that
  forbids *keyring*, *environment variable* and the rest runs over them.
- **`src/testing.rs`**: `NeverKept` (a store nothing may ask, so *this road never
  reached for a credential* is a fact rather than an assumption), `Keeping`, and
  a per-test directory.
- **`Cargo.toml`**: `alo-models` as a **dev-dependency**, for the provider
  road's test — see *One thing I decided differently* below.

### The roads

| Road | Where it asks | Unit named |
|---|---|---|
| a turn's question | `crates/alo-agentd/src/the_road_out.rs` | `alo-agentd.service` |
| the system's own update | `crates/alo-looking-once/src/road.rs` | `alo-looking-once.service` |
| installing and application updates | `alo_software::TheRentedTool::taking` | the unit that starts the tool |
| a provider's list | `alo_models::Trying::taking` | the unit that puts the question |

`alo-agentd` and `alo-looking-once` are the two roads with a production
road-builder in the workspace, and both were wired: `TheRoadOut::to` now ends in
`signed_in`, with `NotTaken::NotSignedInToTheProxy` carrying `alo-proxy`'s own
words to the agent, and `alo_looking_once::road::road_out` likewise with
`NoRoad::NotSignedIn`. `alo-software` and `alo-updating` are handed their
`Carried` by whoever starts the program, exactly as task 4 left them; their
tests build it the way a caller does.

**The unit is named, never worked out.** `TheRoadOut::THE_UNIT` and
`alo_looking_once::road::THE_UNIT` are constants, so `$CREDENTIALS_DIRECTORY` is
not read and there is no code path that could — `alo_secrets::TheBus::of`'s rule,
applied to a directory instead of a bus. The constants are also the list of unit
files that have to carry the `LoadCredentialEncrypted=` line.

### The contract

`docs/contracts/machine-description.md` now says where `password-in-keyring` is
looked up, with the two commands an administrator runs. **The key keeps its
spelling**: it is a published contract, and renaming it would break every
description already written against it. Nothing about what `[proxy]` may contain
widened — a password written into the file is refused exactly as it was.

## Acceptance, criterion by criterion

| The acceptance | Where it is held |
|---|---|
| signed in on **every road out** — installing and application updates, the system's own update, a provider's list, a turn's question | `alo-software` `installing_looking_for_updates_and_updating_all_sign_in_to_the_proxy`; `alo-updating` `checking_for_an_update_and_fetching_one_both_sign_in_to_the_proxy`; `alo-proxy` `a_providers_list_is_asked_for_through_a_proxy_this_machine_signed_in_to`; `alo-agentd` `a_machine_whose_proxy_asks_who_it_is_signs_in_on_the_road_a_question_takes` |
| through `Carried::with_the_password` **and nothing else**, so `grep` finds every caller | `alo-proxy` `signing_in::tests::the_password_reaches_a_road_through_this_door_and_no_other`, which reads every `crates/*/src/**.rs` in the workspace and asserts the only two files naming it are `carried.rs` (where it is defined) and `signing_in.rs` (the one door) |
| read from where ADR 0022's successor says it lives, **never from the description** | `alo-proxy` `provisioned::tests::*`; the description's own refusal is untouched and still runs (`alo-agentd` `machine_wide_proxy`) |
| a machine that cannot read it **refuses the road in words**, naming what to do and quoting nothing stored | `alo-agentd` `a_machine_that_was_never_given_the_password_refuses_the_question` and `a_password_anybody_could_read_refuses_the_question_in_its_own_sentence`; `alo-proxy` `signing_in::tests::each_refusal_says_what_to_do_and_quotes_nothing` |
| no record, no log, no indicator line and no `Debug` carries the password | `alo-agentd` `nothing_a_person_reads_and_nothing_written_down_carries_the_password`; `alo-proxy` `signing_in::tests::nothing_formatted_on_this_road_carries_the_credential` |

**Two of the road tests read the credential off the wire.** `alo-agentd`'s and
`alo-proxy`'s provider test each stand a listener up for the company's proxy and
assert what arrived: `CONNECT <the provider> HTTP/1.1` followed by
`Proxy-Authorization: Basic …` decoding to the name and the password. A test
that read a configured value back would pass on a machine that configured a
proxy and then connected somewhere else, which is the failure this whole line of
work exists to prevent. The base64 in the assertion is **worked out in the
test** rather than written down, so a change to how a credential goes on the
wire fails as a difference on the socket.

**The refusals are checked at both doors and in the record**: nobody came to the
proxy, nobody came to the provider, and nothing was written down as having left.

## Two things I decided, and why

**1. The unit's name rather than `$CREDENTIALS_DIRECTORY`.** systemd exports
that variable for exactly this, and reading it would have saved a constant per
crate. It is rejected in the ADR: it would be one environment variable deciding
where alo OS looks for a credential, which is the thing ADR 0022 refused
`DBUS_SESSION_BUS_ADDRESS` for, and `alo-secrets` states the rule better than I
could — *a function that takes a name cannot be pointed somewhere by a
variable*. Naming the unit has a second benefit: the constant sits where
somebody adding a road will see it, and it is the list of unit files that need
the line.

**2. The provider road's test lives in `alo-proxy`, not in `alo-models`.**
`alo-models` belongs to `docs/autonomy/v0-5-the-models-measured-plan.md`, whose
task 9 is *blocked — not pursued, by the owner's decision of 2026-09-15* and so
is not marked done; `tools/kernel-loop/src/who_owns.rs` therefore refuses a lane
that writes in that crate, and the rule is right — two machines in one crate is
what it exists to prevent. So the road is exercised from this side: `alo-models`
is a dev-dependency of `alo-proxy` (`alo-models` already dev-depends on
`alo-proxy` for task 4's test, so the two dev edges are a cycle Cargo permits and
neither crate's real dependencies move), and the test drives the production
`alo_models::Trying` request and reads the wire. The file says so in its own
header. **If that plan's task 9 is ever marked done, moving this file into
`crates/alo-models/tests/` beside task 4's is a one-line change and would be
tidier.**

## What is not done, and is not implied

- **The `LoadCredentialEncrypted=` lines are not in the image.**
  `image/usr/lib/systemd/system/*.service` is the installer plan's, and this
  plan's header says it reads `image/` and never edits it. ADR 0059 and the
  contract name the exact lines. **Until they land, a machine whose proxy asks
  for a name is refused by alo OS, in words, rather than by the proxy** — which
  is strictly better than what it did and is not the finish. Nothing here claims
  otherwise.
- **No acceptance against a real `systemd-creds` on a booted machine.** No
  machine this ran on is a booted alo OS machine. What is measured is the
  reading, the refusals, and the credential on a real socket; what is not
  measured is PID 1 decrypting a credential and handing it to a unit. Stated in
  the ADR as well, so it cannot be read in from the code.
- **A person cannot yet write one from Settings.** ADR 0059 leaves it open
  deliberately — it is a privileged write and belongs at ADR 0049 §3's broker
  door — and it is **task 13**, written into the plan in this change.
- **`Road::SigningIn` and `Road::FetchingAModel`** have no production
  road-builder in this workspace yet, so they have no per-road test here. They
  are not among the four the acceptance names, and `alo_proxy::signed_in` is the
  same one door when they arrive.

## Verification

Run on this development PC, in WSL Ubuntu against the Linux copy of the tree at
`/root/alo-trees/this-machine`, with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`,
as `docs/autonomy/SHARED_MAIN.md` requires. **These are developer checks and not
certified-hardware acceptance.**

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test -p alo-proxy` | 105 + 4 + 4 + 1 doc, 0 failed |
| `cargo test -p alo-agentd` | 527 + 5 + 4 + 21 + 1 + 1 + 1, 0 failed, 34 ignored |
| `cargo test -p alo-looking-once` | 3 + 13, 0 failed |
| `cargo test -p alo-software` | 75 + 12 + 8 + 5 + 5 + 5 + 4 + 2 + 1, 0 failed |
| `cargo test -p alo-updating` | 17 + 10 + 9 + 7 + 6 + 5 + 4 + 4 + 1 + 1, 0 failed, 5 ignored |
| `cargo test -p alo-citing` (a decision was added and cited) | 21 + 10, 0 failed |
| `cargo test -p alo-saying` (three strings were added) | 63 + 4 + 1, 0 failed |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS="-D warnings"` | clean |

**The full workspace suite was deliberately not run here**, on the supervisor's
instruction: it takes the better part of an hour on this machine and the
supervisor runs it after this regardless.

`Cargo.lock` changed — one line, `alo-models` under `alo-proxy`'s dependencies,
from the dev-dependency above. That makes this change workspace-wide by
`SHARED_MAIN.md`'s table, so the integration candidate should run all nine.

## Proposed changelog entry

> **A company proxy that asks who you are is now signed in to.** Every road
> alo OS takes out of a machine — installing and updating applications, the
> system's own update, choosing a provider, and putting a question — uses the
> password the machine was given for its proxy. A machine that has not been
> given one says so, and says what to do about it, instead of quietly being
> turned away by the proxy or quietly going around it.

## Proposed queue and roadmap updates

- `docs/autonomy/QUEUE.md`: task 12 of the software-and-the-web plan done;
  task 13 (*A person's own proxy password, set on the machine that is theirs*)
  written and ready, depending on 12.
- `ROADMAP.md`: v0.5 *corporate proxy support, machine-wide* is closer but not
  finished — the unit lines in `image/` are outstanding and are the installer
  lane's. Do not tick it on this report.
- `docs/autonomy/STATE.md`: reference this report and
  `docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md`.
