# A real keyring answers

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-secrets`)
- Contributor: Claude Code
- Task: An isolated real Secret Service fixture, and retrieval against it
- Status: **fixture built and isolated; real encrypted retrieval proved.** Three
  of the six continuation items are open, named below.

## The packages

Coordinated first: the other checkout was **mid-build**, so nothing was installed
until `pgrep cargo` reported it idle.

**The transaction was inspected before anything happened.** The default
resolution pulled `libpam-gnome-keyring` — a PAM module that changes login
configuration — so it was **not used**. With `--no-install-recommends`:

**24 installs, 0 removals, and no `libpam-gnome-keyring`.** Verified afterwards:
nothing in `/etc/pam.d` references gnome-keyring, no shared service was
restarted, and no Windows configuration was touched.

What the packages themselves did, reported because it is a configuration change
even though it is not mine: they registered D-Bus activation files
(`org.freedesktop.secrets.service`) and enabled **user** systemd units for
`gcr-ssh-agent`. Neither takes effect without a user session, and this machine
has none. One package, `gtk-update-icon-cache`, was upgraded rather than newly
installed.

### Reproducible prerequisites

```
apt-get install --no-install-recommends gnome-keyring dbus-daemon
# and, from the earlier step, libsecret-1-dev — kept, though the binding no
# longer uses it
```

## How the fixture isolates itself

Inspected **before** launching, from `gnome-keyring-daemon --help`:

- **Its own bus** — `dbus-daemon --session --address=unix:path=…` in a temporary
  directory.
- **No service activation** — the bus is given `XDG_DATA_DIRS` pointing at an
  *empty* directory, so it cannot start anything on demand. Without that,
  `org.freedesktop.secrets.service` would let the bus launch a keyring **with the
  bus's own environment**, outside every isolation below, and a test could pass
  against the wrong store.
- **Its own storage** — `XDG_DATA_HOME` in that directory, so no existing
  keyring is read, written or unlocked.
- **Its own control directory**, `0700` as the daemon requires.
- **`--components=secrets`** only, and **`--unlock`** reading a synthetic
  password from stdin. `--start` is incompatible with `--unlock` — measured, not
  supposed.
- **Never `--replace`**, which is the flag that would make this the machine's
  normal keyring.

Cleanup kills only the two processes it started and removes only the directory it
made. **The packages stay installed.**

### Ready means usable, not answering

The first version waited until the Secret Service answered on the bus. That is
not the same state: the daemon takes its name **before** it has finished making
and unlocking the login keyring, so a connection can succeed while
`get_default_collection` still says `NoResult` — which is what two of three tests
hit.

Readiness is now the state the fixture actually promises — **a collection to put
a secret in**. Five consecutive runs, three tests each, all green, and a full
workspace gate pass.

**Neither failure in this task has a proven cause, and the fix does not depend on
one.** The build machine's disk reached 100% while this work was going on, and a
keyring daemon that cannot write its keyring files offers no collection either.

- The **earlier** `NoResult` is equally explained by a start-up race and by the
  disk. This note picks neither.
- The **later** failure — `never offered a collection`, alongside the linker
  reporting `No space left on device` — is **consistent with disk exhaustion**.
  That is not the same as proven, and it is not recorded as proven.

Waiting for a usable collection rather than a service that merely answers is
right on its own terms, whichever it was: a fixture that is only sometimes ready
produces a suite that fails for reasons having nothing to do with its subject,
and that is how a real refusal later gets waved through as "the flaky keyring
test". If it recurs with space available, that is when it gets investigated.

## What is proved

`crates/alo-secrets` — **12 tests, all passing**.

| Test | What it proves |
|---|---|
| `a_key_that_is_there_is_handed_over` | **real encrypted retrieval** — a key stored by one client is read back by ours over an `EncryptionType::Dh` session against a real `gnome-keyring-daemon`. `Plain` would have worked and was not used |
| `a_reference_nothing_was_filed_under_is_missing` | **`Missing`** against a real store that is open and simply has no such entry |
| `an_item_that_is_not_ours_is_missing_rather_than_borrowed` | an item another application filed under **our reference but its own schema** is not read as ours |
| `the_intended_bus_is_reached_and_an_environment_decoy_is_not` | routing, with the decoy proved live |
| the eight in `bus` and `refusing` | the socket-shaped `Unavailable` cases, and the four states told apart |

## A bug the fixture found in shipped code

`can_be_said_as_an_address` (published in `66d2e15`) refused only `,` and `;` —
the two characters that change an address's *meaning*. It let through every
character that makes an address **invalid**. The fixture hit it immediately:

```
Failed to start message bus: In D-Bus address, character '(' should have been escaped
```

A path containing `(` would have been a bus this crate called usable and no
client could open. The check is now the specification's optionally-escaped set —
`A-Z a-z 0-9 _ - / . \` — with everything else refused rather than escaped,
because escaping would be a second spelling of the same path.

`TheKeyring::opened` now also **asks the service for its collections** before
answering, so a store that cannot answer is refused there rather than at the
first key somebody wanted.

## Open, and precisely why

**1. A real bus with no Secret Service on it.** The test is withdrawn, not
silently dropped, and the file says so where it was. The fixture cannot yet stop
*only* the keyring: signalling the child leaves something still serving
`org.freedesktop.secrets`, so the test asserted a state it had not produced. **A
test whose fixture does not reach the state it names proves nothing.** The
`get_all_collections` check above is the code for it and **that path is
unproven**.

**2. Locked and denied.** Both need the fixture to lock a collection and to make
the service refuse a caller. Neither is done, and neither is faked — an injected
transport failure is not a real-store refusal, and the difference is the whole
value of having a real store.

**3. Items 3 to 6** — daemon wiring, authenticated HTTPS through `alo-agentd`,
connection lifetime and concurrency, and the no-request/no-disclosure/no-fallback
proofs at that level — are not started. They sit on top of retrieval, which now
works.

## What is preserved

Redaction; refusal without fallback; agent and daemon identity separation;
`/run/alo/<uid>/agentd.sock` and every check in `place.rs`, untouched. **No access
control was weakened and no cryptography was invented**: the session is the
library's own DH, the fixture's password unlocks only its own temporary keyring,
and `alo-agentd` still refuses a keyed provider with nothing sent.

## Coordination

Image packaging and sign-in remain the desktop worker's; this fixture completes
neither. Their checkout is untouched.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible. **ROADMAP.md** — no tick, no tier moved.

**docs/autonomy/QUEUE.md** — the credential-store item is progressing; retrieval
works, three states remain.

**docs/autonomy/STATE.md** — three facts. An isolated real Secret Service fixture
exists — private bus, activation disabled, temporary storage, synthetic password,
never `--replace` — with prerequisites recorded. **Real encrypted retrieval is
proved** against `gnome-keyring-daemon`, along with `Missing` and schema
isolation. And the fixture found a real bug in the published address check, which
refused only the characters that change an address's meaning and admitted every
one that makes it invalid.
