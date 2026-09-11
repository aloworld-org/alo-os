# ADR 0024 — What a person signs in at, and what starts it

**Status:** **ACCEPTED, 2026-09-11 — Option B**, after the measurement this ADR
said it owed was taken and answered in its favour. See *The measurement, taken*
at the end. Accepted under a standing delegation from the owner; everything
below is left exactly as it was argued while unaccepted, so the reasoning can be
checked against the outcome rather than rewritten to match it.
**Date:** 2026-09-10
**Proposed by:** the v0.01 delivery workstream
**Context:** [ADR 0002](0002-the-shell-is-native.md) (the shell is native),
[ADR 0004](0004-the-organisations-machine.md) (whose machine it is),
[ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (the base is
rented and the image is a container),
[ADR 0017](0017-the-agents-door-is-ours-and-not-in-the-session.md),
[ADR 0018](0018-the-boundary-is-loaded-by-a-loader-not-by-the-agent.md) (one
privileged component), `crates/alo-accounts`, `crates/alo-entering`,
`crates/alo-shell`, `image/`, `docs/features.md` (*Sign-in with an alo identity,
and a local account that needs no tenant*; *Boots on one certified machine,
firmware to sign-in*)

## The question in one line

**What draws the sign-in, what checks the password, and what turns a correct
password into the session `alo-agentd.service` is bound to?** Three questions
with one answer, because whoever draws it is running when nobody is signed in,
and that is a position in the system rather than a screen.

## What is true today, verified rather than remembered

- **`crates/alo-accounts` authenticates and nothing calls it.** The store, the
  Argon2id hash, the evenly-timed refusal and the session that cannot disagree
  with the machine description are all built and tested. It is a library.
- **`crates/alo-shell` has no binary.** No `src/main.rs`, no `[[bin]]`: the
  compositor is a library that nothing starts. There is nothing for an image to
  install as *the shell*, and nothing to boot to.
- **`alo-agentd.service` is already wired to a session it cannot cause.** It is
  `WantedBy=`, `BindsTo=` and `After=` `user@1000.service`, so it starts when
  the person's own systemd manager starts and stops when it stops. Nothing on
  the image starts that manager, because nothing signs anybody in.
- **The image boots to a text console.** `image/` installs two daemons, two
  units, two directories, two logins and one description. There is no greeter,
  no display manager, and no `graphical.target` wiring in it.
- **The machine ships with no accounts**, and as of this change that is a test
  (`crates/alo-image`, `AnAccountShippedWithTheImage`) rather than an accident
  of what happens to be in the tree.

So the gap is not a missing screen. **It is that nobody has decided what runs
before anybody is signed in**, and every remaining piece of phase 7 — which unit
the image enables, what `graphical.target` pulls in, whether `alo-accounts` is
the authenticator or a development artefact — is downstream of that one
decision.

## What alo OS may not do, whichever option is taken

These are constraints rather than options, and they knock out otherwise
reasonable designs before the comparison starts.

- **No rented name reaches a person** (`docs/features.md`, and
  `alo-saying`'s `rented` check in CI). Whatever is underneath, a person signs
  in to alo OS.
- **Every string is externalised** (`CLAUDE.md`). A sign-in surface says at
  least *your password was not right* and *make an account*, in 24 languages.
  A rented surface says it in its own vocabulary, which alo OS neither
  translates nor owns.
- **No unsafe block** (`unsafe_code = "forbid"`, workspace-wide).
- **One privileged component** (ADR 0018). A second is not forbidden, but it is
  an ADR rather than a line in a unit file, and it is the thing this decision
  costs.
- **The person's system password is not the alo account.** `alo` is created by
  `sysusers` with no password set; if a design lets a Unix password sign
  somebody in, then `alo-accounts` is a decoration over `/etc/shadow` and the
  local account that needs no tenant was never built.

## The options

### Option A — rent a greeter, and let PAM be the authenticator

A small rented greeter daemon runs an unprivileged greeter process and opens a
session through PAM on success.

- **What it buys:** the session transition is somebody else's solved problem —
  the one genuinely hard part, and the part alo OS has no measurement of.
- **What it costs:** PAM decides who signs in. `alo-accounts` then either dies,
  or survives as a PAM module — which is a C ABI, which is `extern "C"`, which
  is `unsafe`, which the workspace forbids. **Taking this option means asking
  for that exemption**, and the exemption is on the authentication path, which
  is the worst place in the system to put the first one.
- It also puts a rented component's vocabulary in front of the person at the
  first screen they ever see, which is the one screen alo OS cannot afford to
  have somebody else's words on.

### Option B — alo OS's own sign-in surface, and PAM only for the transition

The compositor starts at boot as a system service on a virtual terminal, draws
the sign-in from `alo-accounts` and `alo-appearance`, and on a correct password
asks a small privileged opener to start the person's session. `alo-accounts` is
the authenticator; PAM, if the measurement says a logind session cannot be
opened any other way, is **configured underneath the transition and never
consulted about who the person is**.

- **What it buys:** alo OS owns the first screen — its words, its palette, its
  accessibility, its 24 languages — and the account model that was built is the
  account model that runs. Nothing is rented in front of a person.
- **What it costs:** a second privileged component, and it must be tiny and
  enumerated the way ADR 0018's loader is: it takes a uid that has already been
  authenticated and asks `systemd-logind` to open a session for it, and it can
  do nothing else. **Its capability set belongs in the change that builds it,
  as a test in `crates/alo-image`, beside the loader's.**
- **What is not yet measured:** whether `logind` will open a session for a
  caller that is not `pam_systemd`, and on the pinned base rather than in
  general. That is the first thing the implementation measures, and
  `docs/quirks.md` is where the answer goes. **This ADR does not assume the
  answer**; if the only supported road is a PAM stack that authenticates
  trivially and exists to make the session, that is Option B with PAM as an
  engine, which is what *engines are configured, never patched* means.

### Option C — rent a display manager whole

Rejected, and named so nobody proposes it as new. It brings a second toolkit, a
second settings surface, a second set of translations, and a login screen alo
OS neither designs nor owns, on a product whose case is that the machine is the
owner's. ADR 0002 settled that the shell is native; the screen a person meets
first is not the place to make an exception.

## The recommendation

**Option B.** The first screen is a product decision before it is a technical
one, and it is the one screen every person sees on every machine. The cost is
real and it is nameable: a second privileged component, small enough to read in
one sitting, held to `crates/alo-image`'s checks the way the loader is.

With it, two things that are not separable from it:

- **First boot asks a person to make an account rather than to sign in.** The
  image ships no store, which is now a test, so the first surface a machine
  ever shows is *make an account*. A machine that shipped one would ship a
  password every holder of the image knows.
- **The person's Unix login stays passwordless and unable to sign in**, so that
  the alo account is the only way in and cannot be walked around at a console.

## Consequences if it is accepted

- `crates/alo-shell` gains a binary, and it is the desktop lane's — this is the
  decision that unblocks that work rather than a licence to start it here.
- A new privileged component, with an ADR-grade capability set and its own
  entry in `crates/alo-image`'s checks.
- The image gains: the shell, the opener, their units, and `graphical.target`
  wiring. `crates/alo-image` gains a check per promise, each with the twin that
  breaks one line, which is how everything else in that file is held.
- `docs/contracts/machine-description.md` is unaffected: who the person is is
  already a number in it, and this decides who may become that person.

## Consequences if it is rejected in favour of Option A

- An exemption from `unsafe_code = "forbid"` on the authentication path, argued
  and recorded before any code is written.
- `crates/alo-accounts` is either retired or rewritten as a PAM module, and the
  local account that needs no tenant becomes a Unix account with an alo name on
  it.

## What this does not decide

- **The look of the sign-in surface.** That is the desktop lane's, and it is
  drawing rather than deciding.
- **Sign-in with an alo identity** — the tenant half of `docs/features.md`'s
  line. This is the local account only, which is the half v0.01's exit gate
  needs.
- **More than one person on a machine.** `docs/features.md` puts that at v1,
  and nothing here forecloses it: the store already holds accounts rather than
  an account.

## The measurement, taken

**2026-09-11.** This ADR said it did not know whether `logind` would open a
session for a caller that is not `pam_systemd`, and that finding out was the
first thing the implementation owed rather than something to guess. It was
measured before the decision was taken.

Asked of `org.freedesktop.login1.Manager.CreateSession` directly:

| Caller | What logind answered |
|---|---|
| root, well-formed arguments | `Invalid leader PID` |
| uid 1000, the same call | `Access denied` |

**`Invalid leader PID` is the finding.** It is not a refusal to let the caller
in — it is logind having *accepted* the call and gone on to check its contents,
then rejecting the leader PID offered (deliberately an implausible one, since
what was wanted was the authorisation answer and not a real session). The method
is on the interface, there is no policy rule against it, and the boundary that
does exist is **privilege**, which the second row shows: the same call from an
ordinary person is denied outright.

So `logind` will open a session for a caller that is not `pam_systemd`, provided
that caller is privileged. **Option B is buildable**, and the expensive
alternative this ADR priced — `alo-accounts` becoming a PAM module, which is a C
ABI, which is `unsafe`, which the workspace forbids — is not necessary. The
exemption named as Option A's price does not have to be paid.

**What it confirms rather than removes** is Option B's own price, which this ADR
already stated honestly: the sign-in surface must be **privileged**. That is a
second privileged component beside ADR 0018's loader, and it is held to the same
terms — small enough to read in one sitting, and checked by `crates/alo-image`
the way the loader is.

**Taken again on the pinned base, 2026-09-11.** The paragraph below said this
check was owed before any box was ticked, and it has been taken: Fedora 42,
systemd 257, running the alo OS image built from `quay.io/fedora/fedora-bootc:42`
under `podman --systemd=always`. The answer holds — root is refused only on the
leader PID, uid 1000 is refused outright — so **Option B is buildable on the
machine alo OS actually ships.** Two differences that are not differences in the
answer are recorded in `docs/quirks.md`: the sentence for the root case is
reworded between systemd 257 and 259, and an unprivileged caller is sometimes
turned away by the bus policy rather than by `logind`, under the same D-Bus error
name. `crates/alo-sessiond` decides on the name and never on the wording.

**Where it was measured, and where it was not.** Ubuntu 26.04, systemd 259,
under WSL2 — the development machine. The image ships a Fedora-derived base with
its own `logind`. This is the same upstream D-Bus interface and the same
authorisation model, so the answer is expected to hold; it is **not** measured on
the pinned base, and the implementation owes that check on the image before any
box is ticked. Recording it as measured here would be the guessing this
paragraph exists to prevent.
