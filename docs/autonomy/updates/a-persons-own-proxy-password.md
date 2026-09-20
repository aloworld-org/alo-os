# A person's own proxy password, set on the machine that is theirs

**Date:** 2026-09-20
**Workstream:** v0.5 software and the web — `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 13
**Contributor:** this development PC's software-and-the-web lane
**Status:** ready for integration

## What a person gets out of this

A person on their own machine could already set a proxy in Settings. If that
proxy asked who they were, the field beside it did nothing: the only way to give
the machine the password was to open a terminal and run `systemd-creds` by hand,
which is not a thing anybody outside this repository knows to do. Task 12 made
every road out of alo OS sign in to a proxy that asks — and then refuse, in
words, on every machine that had never been given the credential.

Now the password is set from the same place as the proxy, in the same act. A
person types it once; it is written into the machine's own credentials, where
ADR 0059 put it; it is never shown again and never read back out; and the
settings panel can say *a password is set* and offer to replace it.

And the thing that could have gone wrong does not: **a machine where the
credential cannot be written does not end up with a proxy it cannot sign in
to.** The credential is written first and the proxy only if that succeeded, so a
failure leaves the machine exactly as it was.

## The decision, written first

`docs/decisions/0060-a-persons-own-proxy-password-is-set-with-the-proxy-in-one-act.md`,
accepted 2026-09-20. ADR 0059 named this gap and deliberately left it — *writing
a credential from Settings is not decided here and is not built … a door in the
broker for it is a later task* — and pointed at ADR 0049 §3, which is where the
broker's proxy door lives. So the first question was whether this task was
allowed to build anything at all.

**It was, and here is why, because the plan's own constraint turns on it.** The
constraint says: *if carrying this to the broker needs a change to the broker's
own door, and that door turns out to be another plan's, the task stops at a
decision record.* The broker's door and its closed verb list are
`crates/alo-broker`, which belongs to
`docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` and which still has
unfinished tasks — so a twelfth verb was not available. **Not one line of
`crates/alo-broker` is touched by this change.**

That turned out to be the better design rather than a constraint worked around,
and the plan's own acceptance is what says so: *a machine where the write fails
says so and changes nothing, rather than leaving a proxy set that cannot be
signed in to.* Two verbs cannot promise that. Each would carry its own approval,
and an approval is never a session (ADR 0001 §5), so there is no order of the
two that cannot be interrupted between them. One verb can promise it trivially:
write the credential, and write the proxy file only if that worked.

### The four decisions inside it, and why each went the way it did

**One act, no twelfth verb.** `network.set-proxy` now carries the proxy *and*
the password it signs in with. The verb's argument is unchanged — thirty-two
bytes — and what crosses the door is the digest of the two handed-over files
together.

**One reserved name: `the-proxy-on-this-machine`.** This is the decision I
expected to be a detail and was not. A credential reaches a unit through
`LoadCredentialEncrypted=<name>:/etc/credstore.encrypted/<file>`, which is a
**static line in a unit file**. An organisation writing a machine's description
picks its own name and provisions that credential when it builds the machine, so
the line can be written for it. A person choosing a name in Settings cannot be:
nothing rewrites unit files afterwards, and the credential would be delivered to
no unit at all. So alo OS reserves one name for a person's own machine
(`alo_proxy::THE_PERSONS_PROXY_PASSWORD`), identical on both sides of the
colon, no blank space, one path segment — and **the broker refuses to write a
credential under any other name**. That second half matters independently: it is
what stops a root process writing a file somebody else chose the name of into
the machine's credential store.

**The handed-over password is salted.** The identity that crosses the broker's
door is written into the broker's record, and SHA-256 of what somebody typed is
a dictionary attack away from what they typed. So the bytes handed over are
thirty-two bytes of the kernel's randomness followed by the password, and the
identity is taken over those. Without this, *the password never reaches the
record* would have been false in the one way nobody would have noticed.

**The refusal on a managed machine is `alo-proxy`'s existing sentence, said
before anything is handed over.** The broker already refused a person's proxy
over an organisation's — but it refuses in English into its own service log, and
`alo_changing_network` turned every refusal the broker gives into *the proxy was
not set*. So the person never read ADR 0016's sentence. The road a person's own
change takes now reads the machine's proxy file first and refuses with
`alo_proxy::NotChanged::AnOrganisationSetIt`, which says who can change it. The
useful consequence beyond the wording: **nothing is handed over**, so a password
typed on a managed machine never leaves the process it was typed into. The
broker still refuses independently; this is a better sentence in front of a
check that was already there, never a check moved out of the privileged
component.

## What changed, file by file

### `crates/alo-proxy` — the writer beside task 12's reader

- **`src/provisioning.rs`** (new). `alo_proxy::TheMachinesCredentials` is the
  counterpart of `TheMachinesPasswords`: `written(&WhereThePasswordIs,
  &Password)` runs `systemd-creds encrypt --with-key=auto --name=<entry> - <out>`,
  started directly, environment cleared (`nothing_of_this_machines`, the
  evaluator's own list), **the password on its standard input** and never in an
  argument. It writes beside its place in the store, sets `0600`, checks the mode
  it actually got rather than trusting the tool, and renames over — so a store
  that already held that credential still holds it if anything fails. The store
  is made `0700` and root's if it is not there. `handed_over` and
  `what_was_handed_over` are the salted bytes and the reading back.
  `NotProvisioned` is seven refusals, and `nothing_was_written` is a method on
  them rather than a comment.
- **`src/password.rs`**: `THE_PERSONS_PROXY_PASSWORD`,
  `WhereThePasswordIs::on_this_machine` and `is_this_machines_own`. The module
  doc's claim about `Password::as_str` was **corrected rather than left**: it now
  has two crate-internal readers, the road out and this writer, and the sentence
  names both. It is still `pub(crate)`, so a password still cannot be taken back
  out of the crate.
- **`src/provisioned.rs`**: `one_thing_to_look_up` and `only_its_owners` became
  `pub(crate)`, so the writer applies the reader's own rules rather than a second
  copy of them.
- **`Cargo.toml`**: `getrandom`, for one call — the nonce. The workspace's one
  door to the kernel's randomness, no fallback generator.

### `crates/alo-networks` — where it is handed over

- **`src/proxy_password.rs`** (new). `THE_WANTED_PASSWORD`
  (`/run/alo-broker/wanted/proxy-password`, beside the proxy, in the folder the
  broker makes `0770` in the person's group) and `handed_over_together`, the one
  place the two files become the identity that is approved. For a proxy handed
  over with **no** password it is the proxy's bytes exactly, which is what
  `docs/contracts/machine-proxy-file.md` already publishes and what this change
  may not break — held by its own test.

### `crates/alo-brokerd` — the act

- **`src/handed_over.rs`** (new). *Open it without following a link, look at what
  was actually opened, refuse anything that is not a plain file of the person's,
  never read more than it should be.* One file rather than two copies, because a
  second copy is a second place for one of those four to be left out.
- **`src/proxy.rs`**: reads both handed-over files, digests them together,
  rebuilds the proxy through `alo-proxy`'s checks, refuses an organisation's
  proxy, **writes the credential, and only then the machine's proxy file**. The
  handed-over password is removed on **every** path, refusals included.
  `this_machines_own` refuses a password handed over for a proxy that signs in
  anywhere else — or under two different names on the two schemes, which is a
  machine that would sign in on one and not the other.
- **`src/main.rs`**, the four test files that build a `Proxy`: `Proxy::handed_over`
  now takes five things. Deliberately not a builder with a default: a broker
  built without somewhere to write a credential would quietly write none, and
  the compiler asking every caller is the cheaper answer.

### `crates/alo-changing-network` — the road a person's change takes

- **`src/carrying_out.rs`**: `set_proxy_by_hand(proxy, password, broker, now)`.
  `TheBroker` gained the password's path and the machine's proxy file.
  `whose_it_is` is the managed-machine refusal, before anything is written.
  `hand_over` writes each file whole, and **removes** the password file when
  none is given — which is what keeps the no-password digest exactly the proxy's
  own bytes.
- **`src/refusing.rs`**: `NotChanged::AnOrganisationSetTheProxy`, which delegates
  to `alo_proxy::NotChanged::AnOrganisationSetIt.said` rather than adding a
  second sentence. **No new word was declared anywhere**, so `alo-saying`'s
  vocabulary and its counts are untouched.
- **`src/verbs.rs`**: the *no verb sets a proxy* test gained
  `set_proxy_password`, `set_proxy_credential` and `sign_in_to_proxy`, and now
  also holds the list to three.

### `docs/`

- **`docs/decisions/0060-…md`** (new), accepted.
- **`docs/contracts/machine-proxy-file.md`**: a new section, marked additive,
  describing the second handed-over file, the reserved name, the digest over
  both, and how a settings panel knows a password is set.

## Decisions a reader may want to argue with

**Why `--with-key=auto` is written out** when it is the tool's default: it is the
line that decides a credential is sealed to this machine's TPM where there is one
and to its root-only host key where there is not. ADR 0059 wrote that down; a
default is a poor place to keep it.

**Why `NotProvisioned::TheToolRefused` carries the exit status and not one byte
of what the tool printed.** It costs something real — a machine where
`systemd-creds` fails for a reason of its own says only that it failed. The
alternative is a refusal that could carry whatever the tool decided to echo, in a
crate whose entire subject is a credential. `alo_proxy::NotSignedIn` made the
same trade on the reading side and this follows it.

**Why the store's file name is the entry name verbatim.** ADR 0059's example used
a different spelling on each side of the colon (`the company proxy` /
`the-company-proxy`), which is fine for a name an administrator types once and
wrong for one that has to be computed by whoever writes a unit line. With the
reserved name there is no mapping at all.

**How criterion (f) was read.** *A machine where the write fails says so and
changes nothing, rather than leaving a proxy set that cannot be signed in to* —
read as: the credential is written first and the proxy only on success, and a
failed write leaves the previous credential untouched. That is what is built and
tested. The other reading, that the proxy should be set before the password,
would guarantee the opposite of what the sentence asks for.

## What this leaves owed, named rather than implied

- **The `LoadCredentialEncrypted=` lines in `image/`'s unit files.** The
  installer lane's, as every line in `image/` is. ADR 0060 §2 fixes the name so
  they can be written once and never again:
  `LoadCredentialEncrypted=the-proxy-on-this-machine:/etc/credstore.encrypted/the-proxy-on-this-machine`,
  on the units `alo_proxy::TheMachinesPasswords::given_to` is called for.
  **Until they land, a machine whose person sets a proxy password has the
  credential written and the roads still refuse in words** — better than before
  and not the finish.
- **The settings panel itself** is `alo-shell`'s, which this plan may not touch.
  What a panel needs is here and is a contract: read
  `/etc/alo-proxy/proxy.json`, and `password` being `the-proxy-on-this-machine`
  is *a password is set*.
- **Acceptance on a booted alo OS image**, with the unit lines in place and a
  real company proxy asking for a name. Not claimed. What *is* measured here is
  the writing: the tests run the real `systemd-creds` where the machine running
  them has one, and check that what was written decrypts back to what was handed
  over.

## Verification

Run on this development PC, in WSL Ubuntu (systemd 255) against the Linux copy
of the tree at `/root/alo-trees/this-machine`, with
`CARGO_TARGET_DIR=/root/alo-builds/this-machine`, as
`docs/autonomy/SHARED_MAIN.md` requires. **These are developer checks and not
certified-hardware acceptance.**

| Command | Result |
|---|---|
| `cargo fmt --all --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test -p alo-proxy` | see the table below |
| `cargo test -p alo-networks` | see the table below |
| `cargo test -p alo-brokerd` | see the table below |
| `cargo test -p alo-changing-network` | see the table below |
| `cargo doc --workspace --no-deps`, `RUSTDOCFLAGS="-D warnings"` | clean |

**The full workspace suite was deliberately not run here**, on the supervisor's
standing instruction: it takes the better part of an hour on this machine and the
supervisor runs it afterwards regardless.

`Cargo.lock` changed — `getrandom` under `alo-proxy`'s dependencies. That makes
the change workspace-wide by `SHARED_MAIN.md`'s gate table, so all nine gates
apply to the candidate.

### The real `systemd-creds` was actually run

Two tests encrypt through the machine's own `/usr/bin/systemd-creds` and one of
them decrypts the result back. On a machine with no such tool, or one that
cannot make a host key, both fall through to the refusal path — which is itself
a test that runs everywhere (*a machine with nothing that encrypts writes
nothing*, *a machine that cannot write the credential sets nothing*). On this
machine the credential was written and read back: 163 bytes, `0600`, in a
`0700` store.

### Acceptance, criterion by criterion

| The plan's acceptance | The test |
|---|---|
| a person may give their proxy the password, from the same place they set the proxy, carried by identity | `alo-changing-network` `the_network_changes_only_through_the_broker::a_person_gives_their_own_machines_proxy_the_password_it_asks_for` |
| carried in the bytes-and-digest shape `proxy_file` already uses, and a proxy with no password unchanged | `alo-networks` `lib::proxy_password::tests::a_password_handed_over_is_part_of_what_was_approved`, `…::a_proxy_with_no_password_is_still_exactly_its_own_bytes`, `alo-brokerd` `the_proxy_set_is_the_proxy_handed_over::a_proxy_with_no_password_is_asked_for_under_exactly_its_own_bytes` |
| written where ADR 0059 says and nowhere else, by the tool the base has, handed-over bytes removed | `alo-brokerd` `the_proxy_set_is_the_proxy_handed_over::the_password_is_written_and_then_the_proxy_is`; `alo-proxy` `lib::provisioning::tests::what_this_machine_writes_it_can_read_back`, `…::a_name_that_is_not_this_machines_own_writes_nothing` |
| a managed machine refused in `alo_proxy::NotChanged`'s existing sentence | `alo-changing-network` `lib::refusing::tests::a_managed_machine_is_refused_in_the_sentence_that_names_who_set_it`, and end to end `…::a_managed_machine_refuses_a_persons_proxy_before_anything_is_handed_over` |
| never in a file the person's programs can read, never in a record, journal or `Debug`, never read back; a panel shows *a password is set* | `alo-proxy` `lib::provisioning::tests::nothing_formatted_here_carries_the_password`; `alo-networks` `lib::proxy_password::tests::a_panel_reads_that_a_password_is_set_and_never_reads_the_password` |
| no verb reaches any of this | `alo-changing-network` `lib::verbs::tests::no_verb_configures_a_vpn_sets_a_proxy_or_lists_networks` |
| a machine where the write fails says so and changes nothing | `alo-brokerd` `the_proxy_set_is_the_proxy_handed_over::a_machine_that_cannot_write_the_credential_sets_nothing`; `alo-proxy` `lib::provisioning::tests::a_tool_that_refuses_leaves_what_was_there` |

Refusal paths tested beside the legitimate ones, beyond the table: a password
changed after its approval sets nothing; a handed-over password file that is a
link, or longer than a password, is not read; bytes that are not a password are
refused in each of five ways; a name that would reach outside the store is
refused before anything is looked up; a store this machine makes is nobody's but
its owner's.

## Proposed updates to the shared documents

For the integration owner; **not edited here**, per `SHARED_MAIN.md`.

**`CHANGELOG.md`**, under v0.5:

> A person can now give their own machine's proxy the password it asks for, from
> the same place they set the proxy. It is written into the machine's own
> credentials, never into a file their programs can read, never shown again and
> never read back — a settings panel says a password is set and offers to
> replace it. The proxy and its password are one change: a machine where the
> password cannot be written keeps the proxy it had, rather than ending up with
> one it cannot sign in to. On a machine an organisation manages, a person is
> told who set the proxy instead, and nothing they typed leaves the window.

**`docs/autonomy/QUEUE.md` / `STATE.md`:** software-and-the-web task 13 done,
2026-09-20; task 14 (*The proxy a person set, read from the machine's file by
the roads that install*) written and ready, depending on tasks 4 and 12. ADR
0060 accepted. Outstanding for the installer lane: the
`LoadCredentialEncrypted=` lines, named above.

**`ROADMAP.md`:** no change proposed. *Corporate proxy support, machine-wide and
honoured* is not finished until task 14 wires the roads that install to the
machine's own file, and the unit lines land.
