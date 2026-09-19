# An organisation's proxy, read from the machine's description

- Date: 2026-09-18
- Workstream: v0.5 software and the web (`docs/autonomy/v0-5-software-and-the-web-plan.md`, task 9)
- Contributor: Claude Code
- Status: **blocked — the code and its tests are complete and unpublished; the
  three gates for `alo-agentd` have not been run.** The machine's one shared
  build cache was held by another lane's nine-gate runs, back to back, for the
  whole of this worker's window. Nothing is handed off, because work that has not
  gated is not finished. The exact state, and how to finish it in minutes, is
  under *Verification*.

## What a person outside this repository needs to know

An organisation that manages a machine can now state the proxy that machine
reaches the network through, in the one file that already describes the machine,
`/etc/alo/agentd.toml`:

```toml
format = 4

[proxy]
goes-through = "an-address"
http = "http://proxy.example.com:8080"
https = "http://proxy.example.com:8080"
except = ["intranet.example.com", ".example.test"]
sign-in-as = "anna"
password-in-keyring = "the company proxy"
```

A great many company networks have no other route out, and on a machine an
organisation manages the organisation is who knows the route. So the proxy that
comes off that file is the organisation's: a person who tries to change it is
told *an organisation set this*, in words naming who set it, rather than shown a
setting that quietly does nothing. A person who writes the identical section into
a description they own owns it, and may change it — a restrictive value is never,
on its own, evidence that somebody else wrote it.

A description with **no** `[proxy]` section is a machine **nobody set a proxy
on**, which is not the same as a machine told to go straight out. *Straight out*
is `goes-through = "nothing"`, and it is something somebody wrote down.

**A password never goes in this file.** The section names the keyring entry the
password is kept under, and the two ways a password would otherwise land in
`/etc` — a `password = "…"` line, and a credential pasted into an address as
`http://anna:hunter2@proxy.example.com:8080` — are each refused by name, in
sentences that never repeat what was written.

An alo OS too old to honour the section refuses the description and does not
start, rather than taking every road out straight onto a network where going
around the proxy is the thing the proxy exists to prevent. A section that is
present and does not hold — an unknown way out, an address that is not one, a key
the way out would ignore — also stops the service rather than becoming *no
proxy*.

## What changed

| | |
|---|---|
| `crates/alo-agentd/src/machine_wide_proxy.rs` (new) | `[proxy]` as typed, and the one door from it to `alo_proxy::Kept`, attributed by who owns the file |
| `crates/alo-agentd/src/describing.rs` | the optional section on the description; `THE_FORMAT` is `4`, `ALSO_READ` is `[1, 2, 3]`; `PROXY_SINCE` names the shape it arrived in |
| `crates/alo-agentd/src/described.rs` | `Bounds::proxy` and `Described::proxy`, absent as `Option::None` |
| `crates/alo-agentd/src/refusing.rs` | eleven `NotDescribed` variants for this section, including `AProxyPasswordInTheFile`, which is the one refusal in the file that never repeats what it read |
| `crates/alo-agentd/src/changing_the_proxy_under_the_description.rs` (new, test only) | the acceptance: a person's change refused in the organisation's words, beside the same change with no section — and the setting put to every road out |
| `crates/alo-agentd/tests/what_a_machine_says_about_itself.rs` | the section read off a real disk, attributed by the file's real owner; its absence in every shape; its refusals |
| `crates/alo-agentd/src/lib.rs`, `Cargo.toml`, `Cargo.lock` | registration; `alo-proxy` as a dependency of the daemon |
| `docs/contracts/machine-description.md` | the `[proxy]` section, its refusals, and its `format` rule |
| `docs/autonomy/v0-5-software-and-the-web-plan.md` | task 9 marked done; task 11 written |

**Nothing in `alo-proxy` was edited.** Which way a road out then goes, what a
proxy address may be, what an exception matches, where a password lives and the
words a person is refused a change in are all that crate's; this task only
produces the value it already takes. The one consequence worth naming is that a
proxy address in a description is built through `ProxyAddress::checked` rather
than by a `Deserialize` derive — see *Decisions*.

## Decisions

**Which way out is written, not worked out.** `goes-through` is `"nothing"`,
`"an-address"` or `"a-configuration"`, and it is required. The alternative —
inferring the shape from which keys are present — makes a section where
commenting out one line silently moves the machine's traffic somewhere else, and
this is a file people edit by hand under pressure. For the same reason **a key
the way out would ignore is refused** rather than dropped: `configuration` beside
`"an-address"`, or `http` beside `"nothing"`, is a line somebody believes is
sending their traffic through a proxy and is not. That is `describing.rs`'s
existing rule about a `region` written beside a bound with no use for one.

**It needs `format = 4`, not a reuse of `3`.** The plan's rule is `[questions]`'s
and `[applications]`', one shape later again: an older service must refuse a
description whose proxy it cannot honour. `deny_unknown_fields` alone would
refuse the section as a typo, which sends whoever is on call to the wrong line.
`[questions]` and `[applications]` are read in a `4` exactly as they were, and a
description with none of the three sections is the same machine under all four
numbers.

**Absence is `Option::None`, and not `TheProxy::None`.** The two take the same
roads today and are not the same fact. Writing *straight out* on a person's
behalf would be this service answering a question about their machine that only
they can answer — and the moment a settings surface shows *this machine goes
straight out* for a machine nobody configured, the distinction stops being
academic. `alo_software::Bound::Nobodys` is the same distinction one section
above; the shape differs because `alo-proxy` has no `Nobodys` and this task may
not add one.

**The section's spelling belongs to `alo-agentd`, not to `alo-proxy`.**
`alo_proxy::ProxyAddress` derives its own `Deserialize` over the fields it holds,
so a description that named that type directly would accept a host nobody checked
and would put a `password` field one line away from a credential in `/etc`.
Everything here goes through that crate's constructors instead — which is the
same decision `describing.rs` already made about `alo_models::SourcePolicy`, with
a sharper reason.

**`password` is declared in order to be refused.** Without the key, a
`password = "…"` line would be answered as *a key nobody declared*, the same
sentence a typo gets, and whoever read it would go looking for a spelling
mistake. It is not a spelling mistake: it is a credential in `/etc`, and it is
worth its own sentence naming `password-in-keyring` as the place it belongs. It
is typed `toml::Value` rather than `String` so that `password = 1234` is the same
refusal — what matters is that somebody wrote a password into `/etc`, not which
shape it arrived in.

**A pasted credential is refused before the address is taken apart.** `@` in an
address is checked first, so the refusal names the file and the key rather than
describing a value that was never built; `alo_proxy::ProxyAddress` refuses the
same shape underneath, and both roads map onto one sentence. The same check
covers `sign-in-as` and `configuration`, which are the other two places a paste
lands. **No refusal for this section repeats what it read**, because a password
in a refusal is a password in a service log.

**Half a sign-in is refused.** `sign-in-as` without `password-in-keyring`, or the
reverse, is a setting that would fail on the first road out — `signing_in` takes
both — so it fails here, where somebody can still read the file and fix it.

**One sign-in for the section, not one per scheme.** A proxy that wants a name
wants it on both schemes; two sign-ins in one section would be two answers to one
question.

**An exception list is checked entry by entry and then whole.** `Exceptions::of`
refuses the list, which is right for a settings panel showing it back, and says
nothing about which line of a file was wrong; a file is edited by line, so the
refusal names the entry. The list is never half-honoured.

**`"an-address"` naming neither scheme is refused.** It would send everything
straight out under a setting that says otherwise, which is the one reading of the
section nobody could have meant. Naming only one scheme is allowed — *straight
out for the other* is a configuration company networks really have.

**The way out is matched before the keys beside it**, so a misspelt
`goes-through` is answered as the typo it is rather than as whichever key turned
out to be missing.

## Acceptance criteria and their tests

**These tests exist in the change and have not been executed** — see
*Verification*. They are listed so that whoever runs the gates knows what to
expect and what the handoff's evidence block should name.

| Criterion | Test |
|---|---|
| an optional section stating the proxy in `alo_proxy::Setting`'s three shapes, documented, with the `format` rule — an older shape carrying it is refused | `describing::tests::a_proxy_in_a_shape_that_could_not_carry_one_is_refused`; `what_a_machine_says_about_itself::a_proxy_in_an_older_shape_is_refused_off_the_disk`; `machine_wide_proxy::tests::an_automatic_configuration_is_the_address_it_is_at` |
| read into `alo_proxy::Kept` with who set it decided by who owns the file, exactly as task 8 | `describing::tests::the_proxy_an_administrator_wrote_is_an_organisations`; `describing::tests::the_same_proxy_the_person_wrote_names_no_organisation`; `what_a_machine_says_about_itself::a_proxy_on_a_disk_is_read_and_attributed_to_whoever_wrote_it` |
| its absence is a proxy nobody set, never *straight out* written on a person's behalf | `describing::tests::a_machine_with_no_proxy_section_has_none_set`; `what_a_machine_says_about_itself::a_description_with_no_proxy_on_a_disk_has_none_set`; `machine_wide_proxy::tests::nothing_is_a_way_out_somebody_wrote` |
| a password is never in the file — the section names the keyring entry, and a password written inline is refused by name | `machine_wide_proxy::tests::the_sign_in_names_the_keyring_and_never_a_password`; `machine_wide_proxy::tests::a_password_written_into_the_file_is_refused_by_name`; `changing_the_proxy_under_the_description::a_password_in_the_description_stops_the_service_and_is_never_repeated`; `what_a_machine_says_about_itself::a_password_in_the_description_stops_the_machine_off_the_disk` |
| a person's own proxy refused in words on a machine whose root-owned description sets one, beside the same change accepted with no section | `changing_the_proxy_under_the_description::a_persons_change_is_refused_on_a_machine_whose_description_sets_the_proxy` (and `…the_same_section_in_the_persons_own_description_is_their_own`) |
| a section that does not hold stops the service, as `[questions]` and `[applications]` do | `describing::tests::a_proxy_section_that_does_not_hold_is_refused`; `machine_wide_proxy::tests::a_key_the_way_out_would_ignore_is_refused`; `machine_wide_proxy::tests::a_way_out_this_service_does_not_know_is_refused_first`; `what_a_machine_says_about_itself::a_proxy_that_cannot_be_read_stops_the_machine` |

The acceptance test reads the description through the production
`describing::read` as root wrote it, takes `Described::proxy()` exactly as it
comes back, makes the person's change through `alo_proxy::Kept::changed_by_the_person`
— the one door a settings panel has — and renders the refusal in the machine's
whole vocabulary from `starting::what_this_machine_says`, checking that the
sentence names the organisation, is not a bug string and has nothing left
unfilled. It also checks that the refused change left the machine's proxy exactly
as the description stated it.

Because a setting that is read and not honoured would be a line in `/etc` doing
nothing, that file additionally puts the setting the description produced to
`alo_proxy::the_way` for every `Road`: the address reaches all seven roads out,
the excepted host still goes straight out, `"nothing"` is straight out, and a
machine told to ask an automatic configuration it cannot evaluate **refuses the
road** rather than going around the rule. Those are `alo-proxy`'s decisions, and
this is the first place they are asked of a setting that came off a disk.

## Verification

**Read this section before treating anything below as evidence.** Three of the
checks this task owes have not been run, and they are named here rather than
implied.

### What was run

- **The supervisor's own tests** — `cargo test` in `tools/kernel-loop`, on the
  Windows host: **151 passed, 0 failed** (17.97 s). This is one of the nine
  gates, and it is the one that reads this change's documentation: it holds the
  plan checks (every plan's task, status and `**Done, <date>**` marks) and the
  ADR citation check against the real `docs/` tree. The plan edit — task 9 marked
  done, task 11 written — passes them.
- **`Cargo.lock` regenerated** with `cargo metadata`, which resolves without
  building. One line: `alo-proxy` under `alo-agentd`. It had been missed, and is
  described under *Recovered from an unfinished session*.
- **A line-by-line review** of the whole change against the plan's acceptance,
  `docs/contracts/machine-description.md`, and `alo-proxy`'s actual public API —
  every type, variant, method and trait signature the new code and its tests
  name was checked to exist as named (`Kept`, `TheProxy`, `ProxyAddress::checked`
  / `signing_in` / `password`, `Exceptions::of` / `let_through`,
  `ConfigurationAddress::checked`, `NotAnAddress::CarriesAPassword`,
  `NotAConfiguration::CarriesAPassword`, `WhereThePasswordIs::named`, `NotAName`,
  `the_way`, `Way::Through`, `Road::EVERY`, `TheEvaluator::asked`,
  `NotEvaluated::NothingEvaluatesIt`, `NotChanged::AnOrganisationSetIt`). The
  exception semantics the acceptance test relies on were read out of
  `crates/alo-proxy/src/exceptions.rs` rather than assumed. That is review, not a
  gate, and it is offered as nothing more.

### What was **not** run, and why

- `cargo fmt --all`
- `cargo clippy -p alo-agentd --all-targets -- -D warnings`
- `cargo test -p alo-agentd`

All three need Linux: `alo-agentd` is `#[cfg(target_os = "linux")]` in its
entirety, so a green run on the Windows host would have compiled none of this and
would be worth nothing. On this machine they therefore run in WSL against
`/root/alo-trees/this-machine` with `CARGO_TARGET_DIR=/root/alo-builds/this-machine`
— the one shared tree and the one shared build cache that
`docs/autonomy/SHARED_MAIN.md` requires every lane to take turns at.

**Another lane held both for this worker's entire window.** Observed directly: a
`cargo test --workspace` already 29 minutes long when this worker was ready to
gate, a second nine-gate sequence beginning minutes after it ended, and a third
`cargo test --workspace` starting after that. One attempted sync was destroyed
mid-flight by the other lane's own `rsync --delete` onto the shared tree. The
machine has four cores, and the shared cache is 43 GB, so the two ways around it
were both refused deliberately:

- **A second build directory** — forbidden by `SHARED_MAIN.md` ("Never create
  another target directory"), and in any case a cold build of this crate's
  dependency graph on four already-contended cores is far longer than the window
  it was meant to save.
- **Building this tree into the shared cache concurrently** — this is exactly the
  cross-contamination that broke `main` on 2026-09-17 (a crate compiled from
  another lane's copy). It would have corrupted the other lane's gate results as
  well as this one's, so it was not done.

Waiting was therefore the only correct option, and the window did not come.

### How to finish this in minutes

A queued run is armed on this machine and will take the shared tree and cache by
itself as soon as they are genuinely free — it waits for **120 seconds of
continuous idle**, so a gap between another lane's gates cannot be mistaken for
the end of its run, then rsyncs this checkout onto the Linux side exactly as
`tools/kernel-loop/src/gates.rs` does and runs the three gates in order:

- script `/tmp/gate3.sh`, log `/tmp/gate3.log` (in WSL, as root).
- The log ends with `GATE1 EXIT=`, `GATE2 EXIT=`, `GATE3 EXIT=`, a
  `TREE STILL MINE:` line that proves no other lane overwrote the tree during the
  run, and `ALLDONE`.

The diff reaches only `crates/alo-agentd/**` and `Cargo.lock`, so against a warm
cache the three gates rebuild `alo-agentd` and its test targets and nothing else.
Each evidence test below must then also be run on its own with `--exact`, which
is the supervisor's own requirement and has not been done either.

**No handoff was written.** The gates are the statement that the work is
finished, and they have not spoken.

## Limitations

- **The daemon reads the proxy and hands it to nothing.** `alo_proxy::the_way`
  answers for the setting in this task's own test, and no caller in `alo-agentd`
  carries it to the road a turn's question actually takes (`alo-asking`, through
  `crate::doing` and `crate::questioned`). On a company network with no other
  route out, that is a machine that reads its organisation's proxy off the disk
  and then asks a provider directly. This is written up as **task 11** in the
  plan — *The proxy a machine was told about, carried to the question a turn
  puts* — with its own acceptance, because carrying it is a change to what a turn
  does rather than to what a description says, and the task's own constraint
  forbids this one from re-deciding which way a road goes.
- **No settings surface shows it.** `Described::proxy()` is the value such a
  surface would show and refuse a change through; the surface is the shell's, and
  this plan may touch nothing in `crates/alo-shell`.
- **`alo-image` reads only format 1.** The image ships an unmanaged machine with
  format 1 and no sections, which is correct and unchanged; a built image
  carrying a proxy would need that reader taught `2`, `3` and `4` first, as the
  contract already says of `[questions]`.
- **Not run on a real company network.** Every proxy here is an address in a
  test. What a machine does against a real corporate proxy — authentication
  round trips, a WPAD server, a TLS-intercepting middlebox — is not shown by
  anything in this change, and belongs with task 11's acceptance on hardware.

### Recovered from an unfinished session, and reviewed line by line

The implementation was left uncommitted in this checkout by an earlier session
that stopped before publishing anything: there was no report, no handoff and no
branch. Per `SHARED_MAIN.md` (*for existing unfinished work, inspect and preserve
it before creating its branch*), it was read line by line rather than
reimplemented, and one real defect was found and fixed: **`Cargo.lock` had never
been regenerated for the new `alo-proxy` dependency**, so the committed tree would
have carried a lock file disagreeing with `alo-agentd/Cargo.toml`. It was
regenerated with `cargo metadata` — one added line — and is among the files this
task publishes. Everything else in the tree was verified against the plan's
acceptance, the contract and `alo-proxy`'s real API before the gates were run.

## The handoff this task did not write

Written out so the next worker repeats no analysis — **and it is not a handoff**:
it must not be copied into `.kernel-loop/handoff.toml` until `/tmp/gate3.log`
shows all three gates at `EXIT=0`, `TREE STILL MINE: yes`, and each evidence test
has been run on its own with `--exact`.

```text
task = Task 9 — An organisation's proxy, read from the machine's description
report = docs/autonomy/updates/an-organisations-proxy-from-the-machine-description.md
subject = feat(agentd): a machine's description states the proxy an organisation set, and a password never goes in it
evidence =
  . alo-agentd lib describing::tests::a_proxy_in_a_shape_that_could_not_carry_one_is_refused
  . alo-agentd what_a_machine_says_about_itself a_proxy_on_a_disk_is_read_and_attributed_to_whoever_wrote_it
  . alo-agentd lib describing::tests::a_machine_with_no_proxy_section_has_none_set
  . alo-agentd lib machine_wide_proxy::tests::a_password_written_into_the_file_is_refused_by_name
  . alo-agentd lib changing_the_proxy_under_the_description::a_persons_change_is_refused_on_a_machine_whose_description_sets_the_proxy
  . alo-agentd lib changing_the_proxy_under_the_description::the_proxy_the_description_states_reaches_every_road_out
  . alo-agentd what_a_machine_says_about_itself a_proxy_that_cannot_be_read_stops_the_machine
files =
  Cargo.lock
  crates/alo-agentd/Cargo.toml
  crates/alo-agentd/src/changing_the_proxy_under_the_description.rs
  crates/alo-agentd/src/described.rs
  crates/alo-agentd/src/describing.rs
  crates/alo-agentd/src/lib.rs
  crates/alo-agentd/src/machine_wide_proxy.rs
  crates/alo-agentd/src/refusing.rs
  crates/alo-agentd/tests/what_a_machine_says_about_itself.rs
  docs/autonomy/updates/an-organisations-proxy-from-the-machine-description.md
  docs/autonomy/v0-5-software-and-the-web-plan.md
  docs/contracts/machine-description.md
```

Every file the evidence names is in that list, which is what
`tools/kernel-loop/src/evidence.rs` requires of it.

## Proposed updates for the integration owner

**Tick nothing from this report yet.** The change descriptions below are ready to
consolidate once the gates have run; until then this report is a record of
unpublished work, not permission to move a release gate.


- `CHANGELOG.md`: *A machine an organisation manages can now state the proxy it
  reaches the network through, in its machine description — none, an address per
  scheme with exceptions, or an automatic configuration address. On such a machine
  the proxy is the organisation's and a person is told so rather than shown a
  setting that does nothing; on a personal machine it is the person's. A proxy's
  password never goes in the file: the description names the keyring entry it is
  kept under, and a password written into the file is refused by name without
  being repeated. A machine whose description names no proxy is one nobody set a
  proxy on, which is not the same as one told to go straight out. An alo OS too
  old to honour the section refuses to start rather than reaching the network a
  way the organisation forbade.*
- `ROADMAP.md` v0.5 *Software*: an organisation's proxy is read from the machine
  description (code and tests); carrying it to the road a turn's question takes is
  still outstanding and is the plan's task 11.
- `docs/autonomy/QUEUE.md`: task 9 done; task 11 (*The proxy a machine was told
  about, carried to the question a turn puts*) written and ready, depending on 4
  and 9 — the gap this task found and could not close inside its own acceptance.
