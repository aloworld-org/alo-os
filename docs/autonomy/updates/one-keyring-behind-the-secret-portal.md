# One keyring behind the Secret portal

**Date:** 2026-09-15
**Workstream:** v0.5 applications, and what they expect —
`docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`, task 3
**Contributor:** Claude (Mac lane worker), for the repository owner
**Status:** ready for integration

## What changed

An application that asks the Secret portal now gets somewhere to keep its
passwords: the person's own keyring, the same one alo OS already keeps a
provider's key in, under the same lock. It only gets in if the person granted
it that, it only ever sees what it kept itself, and revoking the grant stops it
at its next request.

- **`crates/alo-capability`** — a twelfth facility, `Facility::Secrets`
  (`secrets` in the grants file), worded *a place for its own passwords in your
  keyring*. Granted only to applications, like the other eleven.
- **`crates/alo-portals`** — a sixteenth portal, `Portal::Secret`, over that
  facility, with its sentence (`portals.portal.secret`). The acceptance test
  holds the fifteen to the features portal line as before, and the Secret
  portal to its own line, *secret storage — one keyring behind the Secret
  portal*.
- **`crates/alo-secrets`**
  - `application.rs` — `TheKeyring::for_the_application(&Request, &Grants,
    now)`: refuses any portal that is not the Secret portal before the grants
    are read, judges the request with `alo_portals::Request::judged`, and only
    when allowed returns an `ItsOwn`. `ItsOwn::keep`, `kept`, `forget` and
    `the_portals_secret` (the answer to `RetrieveSecret`, made once from
    `getrandom` and the same bytes every time after) each take it by value.
  - `withheld.rs` — `Withheld`, the six ways a request is not answered.
  - `kept_secret.rs` — `KeptSecret`, bytes handed back to the application that
    kept them, with no `Display` and a `Debug` that shows nothing.
  - `store.rs` — shares the provider schema constant, the service and the error
    mapping with `application.rs`; provider lookups are unchanged.
- **`crates/alo-keyring-fixture`** — every bus starts from its own
  configuration, with no `<servicedir>` and no
  `<standard_session_servicedirs/>`. `--session` is gone. `keyring_process()`
  says which process should own `org.freedesktop.secrets`.
- **Docs** — ADR 0040 gains an amendment adding the facility, which the ADR's
  own rule allows because the list grows additively. `docs/contracts/grants-file.md`
  lists `secrets`. `docs/quirks.md`'s 2026-09-13 entry records the fixture fix.
  The plan marks task 3 done.

### Change description (proposed for CHANGELOG.md)

> Applications can keep passwords in the person's keyring through the Secret
> portal, once the person grants them that. Each application sees only what it
> kept, never another application's and never alo OS's own provider keys. A
> locked keyring stays locked, and revoking the grant ends the application's
> access at its next request.

## Decisions

1. **The Secret portal is a sixteenth `Portal` over a twelfth `Facility`.** A
   request for an application's own secrets is not a path and not an installed
   application, which is the situation ADR 0040 was written for. Reusing
   `Reach::Application(own id)` would be option A's pun. ADR 0040 says the
   facility list grows additively, so it was extended, and the ADR gained an
   amendment. That adds to the decision and does not contradict it. The ADR's
   test (`the_decision_this_was_built_from_still_reads_every_portal`) finds the
   new row there.
2. **"A collection the application's name owns" is the set of items filed under
   its identifier inside the person's default collection. It is not a Secret
   Service collection of its own.** A Secret Service collection is also a lock,
   with its own password. One per application would put a second lock on every
   application, which contradicts the plan's *under the same lock*, and making
   one on a real keyring puts a password prompt on screen. Items are filed with
   `xdg:schema = dev.alo.Application` (portal secret:
   `dev.alo.Application.PortalSecret`), `application = <identifier>` and
   `name = <name>`.
3. **Whose secret it is never comes from the caller.** No operation takes an
   application name. The identifier used is the one `Request::judged` allowed,
   so no argument can name another application's secrets.
4. **One judgement, one operation.** Every `ItsOwn` method takes `self`, so each
   keep, read or forget was judged against the grants at that request. A
   revoked grant is therefore refused at the next request. A request already
   allowed before the revocation finishes its one operation, and the test
   shows exactly that.
5. **Revoking does not delete what was kept.** *Gone at the next request* is
   read as the application's access ending, which the test proves. Deleting is
   irreversible, and a folder is not deleted when a grant to it is revoked. A
   person who revokes by mistake and grants again finds the application's
   sign-in where it was. If the owner wants revocation to delete, that is a
   one-line call to `forget` from wherever the revocation is carried out.
6. **The daemon's keys and an application's are kept apart by schema, and every
   search names the schema.** A provider lookup never matches an application's
   item, and no application name reaches a provider key. The test tries both
   directions, using the same name.
7. **Limits:** names up to 255 bytes with no control characters, and secrets up
   to 64 KiB. A keyring holds passwords and keys, not files. The portal secret
   is 64 random bytes. If two first requests race, both read back the earliest
   item, so they get the same bytes.
8. **No words for `Withheld` yet.** This follows `NotStored`: the sentences
   belong with the backend that says and records them (task 5), and
   `Withheld::Refused` already carries `alo-portals`' words. No new English
   reaches a person from this task apart from the two vocabulary strings above,
   which are collected into the machine's one vocabulary.
9. **Item labels hold no English**: `<identifier>/<name>`, or the identifier
   alone for the portal secret, so a keyring manager shows nothing that needs
   translating.

## Acceptance, and the test for each clause

| Clause | Test |
|---|---|
| A Secret portal request is a grant naming the application; a secret is kept in the same keyring, in the application's collection; stored as one application, it cannot be read as another | `alo-secrets` `one_keyring_behind_the_secret_portal::a_secret_kept_as_one_application_cannot_be_read_as_another` |
| Refused before the keyring: nothing granted, the wrong grant, an expired grant, another portal; nothing filed | `…::a_request_the_grants_do_not_cover_never_reaches_the_keyring` |
| Name, size and locked-keyring refusals; a locked keyring stays locked | `…::the_keyring_refuses_what_it_should_and_never_unlocks_itself` |
| The daemon's keys and an application's are separated by what is asked, not by hope | `…::the_daemons_keys_and_an_applications_never_answer_for_each_other` |
| The fixture is the one Secret Service on its bus (the 2026-09-13 race, closed for its own fixture) | `…::the_fixture_is_the_one_secret_service_on_its_bus` |
| Revocable with the application's grant, gone at the next request | `…::revoking_the_grant_ends_the_reach_at_the_next_request` |
| The Secret portal is on the closed list, held to its features line | `alo-portals` `a_portal_request_is_a_grant::the_portals_are_the_closed_list_the_promise_names` |
| The facility is named and worded once | `alo-capability` `lib` `facility::tests::every_facility_is_named_and_said_once` |

**The one-keyring test was checked against the old fixture.** With the fixture
briefly put back to `dbus-daemon --session`, the test failed:
*"the fixture's bus can start a second Secret Service: [… "org.freedesktop.secrets" …]"*.
With the fix restored, it passes. So the test detects the race and does not
depend on the VM's `dpkg-divert`.

## Verification

Run on the Mac lane's Lima VM (`alo`, Ubuntu 24.04 aarch64, as root), with
`CARGO_TARGET_DIR=/root/alo-builds/alo-os-a96435785b5e7d7d`:

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets -- -D warnings` (whole workspace): clean.
- `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps -p alo-secrets -p alo-portals -p alo-capability -p alo-keyring-fixture`: clean.
- `cargo test -p alo-secrets`: all pass, including 6 new integration tests and
  3 new unit tests. The 3 `a_session_that_really_ended` tests are ignored, as
  they already were.
- `cargo test -p alo-capability -p alo-portals -p alo-keyring-fixture -p alo-granted`: all pass.
- `cargo test -p alo-remembering -p alo-saying`: all pass. Neither crate was
  edited; they were run because the grants file and the vocabulary read the
  lists that changed.

**Not run:** the full workspace suite, which is left to the supervisor as
instructed. `alo-agentd`'s tests use the keyring fixture, and every one of them
now gets a bus started from a configuration. The refusal tests already used that
path, so no change in behaviour is expected, but it has not been run here.
**Not physical acceptance:** a virtual machine's keyring is development
evidence, not a certified workstation.

## Limitations

- **No D-Bus portal yet.** `org.freedesktop.portal.Secret` is served by task 5.
  This task decides what that portal answers.
- **ADR 0022's limitation still applies.** Anything running as the person can
  talk to their keyring and could file an item under these attributes itself.
  What is guaranteed is what this door hands out.
- **Not measured:** a real gnome-keyring on a signed-in desktop, as opposed to
  the fixture's own.
- The VM's `dpkg-divert` of the machine keyring's activation file is no longer
  needed by the fixture. It was left in place because removing it is the
  machine owner's call.

## Proposed updates to shared documents

- **CHANGELOG.md:** the change description above.
- **QUEUE.md / STATE.md:** applications plan task 3 done. Tasks 4 (what opens
  what) and 5 (the D-Bus portal backend) remain. Task 5 can now serve `Secret`
  from `TheKeyring::for_the_application`.
- **ROADMAP.md:** nothing moves. This is v0.5 screenless work under ADR 0028.
