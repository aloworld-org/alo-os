# ADR 0136 — A person's name reaches their desktop through a file only they can read

**Status:** accepted — the owner chose this road on 2026-10-28 from four put to
them in plain words. It adds a second holder of `CAP_CHOWN` under
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)'s
rule, and a second unit for `alo-sessiond` to start under
[ADR 0121](0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)'s
bound.
**Date:** 2026-10-28
**Context:** [ADR 0024](0024-what-a-person-signs-in-at.md) (what `alo-sessiond`
is), [ADR 0118](0118-more-than-one-person-on-a-machine.md),
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md),
[ADR 0121](0121-the-thing-that-opens-a-session-makes-the-persons-folder.md),
`docs/design/the-terminal-window.md`, `docs/contracts/the-persons-name.md`

## The question in one line

**How does the name a person typed when they made their account reach their
desktop**, so the terminal's strip reads `Local · anna`?

## Why it cannot today

The owner decided on 2026-10-28 that the strip's name is the signed-in
person's. It was built without one, because:

- the name is in `/etc/alo/accounts.toml`, **root's and `0600`**, because the
  same file holds every account's password hash;
- the sign-in screen (`alo-compositor`) and `alo-sessiond` are the only things
  that read it, and both run as root **with every capability removed** — so
  neither can give a file to another account;
- the desktop runs as the person's login, `alo`, on every machine, and must
  never be able to read a hash;
- nothing personal travels from sign-in to desktop today. The keyboard is not a
  road: the sign-in screen reads a machine-wide file and the desktop uses the
  `ALO_KEYBOARD` its unit names.

## The decision

**At every sign-in, a small privileged unit copies that one name, by number,
into a file only that person can read.**

```text
/var/lib/alo-names              0711 root:root
/var/lib/alo-names/<number>     0600, owned by <number>
```

- `alo-giving-a-name@<number>.service` runs `/usr/libexec/alo-giving-a-name`
  with `CAP_CHOWN` and nothing else, `ProtectSystem=strict`, and
  `StateDirectory=alo-names`. It is `alo-giving-a-folder@.service`'s shape on
  purpose.
- `alo-sessiond` starts it **after** the account is checked and nobody is
  signed in, **after** the folder, and **before** the session opens, because the
  session is what starts the desktop that reads it. A failure goes to the
  journal and **refuses nobody**: the cost is a word in a strip.
- The helper reads the name from the store itself
  (`Accounts::name_of`, the question the store already answers for the sign-in
  screen). **No name crosses a socket**, so `alo-sessiond` still never learns
  who is signing in.
- It writes a sibling, gives it to the person, and renames it into place, so a
  reader sees the old name or the new one and never half. Because it runs at
  every sign-in, **a changed name is picked up at the next one.**
- The desktop reads it once as the session starts
  (`TheDesktop::whose_desktop_this_is`). **No file is no name**: the strip says
  `Local` alone, and **never the login's own name**, which is `alo` everywhere.

The file's shape and who may do what with it are
`docs/contracts/the-persons-name.md`.

## What was not taken, and why

The four roads were put to the owner as options 1 to 4; this is option 1.

**2. Make the sign-in screen a member of the person's group**, and have it write
a file `0640 root:<their group>`. Fewer parts — no new unit. Not taken because
it widens the screen a person types their password into: it could then read
anything the person has opened to their own group, on every machine, for a
label. ADR 0119 measured that screen at 113,587 lines; it is the worst place on
the machine to add a reach.

**3. Let the desktop ask a small root service for its own name**, answering only
for the caller's own number. Clean in principle, but it is a **new long-running
privileged service and a new public surface** — a socket and a protocol a third
party could build against — to serve a value that changes only when an account
is renamed. A file written at sign-in is the same answer with nothing listening.

**4. A file everybody on the machine can read**, the way ordinary Linux keeps
names in `/etc/passwd`. Simplest of all. Not taken because the owner asked for
the name to be **readable by the person alone**, and a world-readable file is
readable by the agent's own account too.

## What this changes about earlier records

- **ADR 0119 said one unit holds `CAP_CHOWN`. Now two do**, each small and each
  a decision. Its rule — *a capability belongs to the smallest thing that can
  hold it* — is why this is a second small unit rather than the folder helper
  growing a second job: that helper's header promises *it reads nothing but its
  two arguments*, and reading the account store would have broken it.
  `crates/alo-giving-a-folder/tests/only_here_may_give_a_directory_away.rs` now
  names both crates, each held to exactly one owner-changing line; a third
  needs a third record.
- **ADR 0121 bounded `alo-sessiond` to starting one unit.** It now starts two,
  under the same bound: each a constant of this repository instanced by a
  checked `u32`, three read-or-start methods, nothing else.
  `crates/alo-sessiond/tests/this_component_only_starts_its_own_units.rs` holds
  the count at two.

## What is measured, and what is not

**Measured in the gate, on every build:** the helper writes `0600` and the
person's ownership, keeps the folder `0711`, replaces a stale sibling, picks up
a changed name, and refuses a missing store or a number nobody has, leaving no
file (`crates/alo-giving-a-name/src/giving_tests.rs`, as root in CI); the
reader's rules (`crates/alo-accounts/src/the_name_given_tests.rs`); the order
and the *refuses nobody* in `alo-sessiond`; the units on the image; and the
strip drawing a name, no name, another script, a right-to-left name and an
over-long one (`crates/alo-shell`).

**Not measured:** a booted machine signing in and drawing `Local · anna`. That
needs an image with this change in it, and is owed.
