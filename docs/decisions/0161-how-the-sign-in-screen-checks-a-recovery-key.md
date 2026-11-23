# ADR 0161 — How the sign-in screen checks a recovery key

**Status:** **proposed, 2026-11-21.** Nothing in it is decided until the owner
answers. It was asked for by the coordinator, because checking a recovery key
needs a new privileged surface, and CLAUDE.md says a privileged surface is
decided before it is built.
**Date:** 2026-11-21
**Context:** [ADR 0054](0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md)
(the recovery key, and how it is typed back);
[ADR 0024](0024-what-a-person-signs-in-at.md) (alo-accounts checks the password,
and `alo-sessiond` turns a correct one into a session);
`docs/features.md`, *A forgotten password never loses a person's data*;
`crates/alo-accounts/src/store.rs`, `Accounts::password_set_again`, whose own
rustdoc says the recovery key is "checked before this is reached".

## What it changes for the person

A person who forgets their PIN or password presses *Forgot your PIN?* at
sign-in, types the recovery key they wrote down at install, and chooses a new
PIN or password. Their files are all still there. This ADR decides only one
thing: which program on the machine is allowed to say *yes, that is this
machine's recovery key*.

## The owner's answers this rests on (2026-11-21, relayed by the coordinator)

1. **Encrypted by default, and a person may turn it off.** The installer
   explains the risk in plain words.
2. **The disk's secret is the sign-in secret.** There is one secret, and the
   recovery key is the backup for both.
3. **Too many wrong tries means a growing delay.** After 5 wrong tries the
   person waits 30 seconds, then longer each time. A person is never locked out
   for good, and nothing is ever erased.
4. **A recovery key typed back wrong at install is shown again.** The install
   does not finish until the key is typed back right, and if the person cancels,
   nothing is left half-done.

## What any answer has to do

- **Check a key without keeping one.** ADR 0054 §4 says the key is never stored
  on the disk it recovers.
- **Change the disk's secret, not only the account's.** By answer 2, a new
  password set at sign-in, or in Settings, is also the new disk secret.
  Otherwise the next start asks for a secret the person has just replaced. So
  the same surface that checks the key also re-keys one LUKS keyslot. This is
  the half that is easy to miss.
- **Answer with yes or no, and nothing else.** The sign-in screen runs as the
  `alo-greeter` group. It is nobody in particular, and it must not learn which
  keyslot exists, what the header holds, or why a key failed.
- **Hold the delay where the check is**, not in the screen. A screen can be
  restarted, and a delay the screen keeps is one a restart clears.
- **Work when encryption is off.** By answer 1 a disk may be plain, and then it
  has no LUKS header to check a key against.

## The options

### A. `alo-sessiond` checks the key itself

It already runs as root, holds the account store and sets passwords. It would
gain a way to read the root's LUKS header and re-key it.

- **For:** no new service, and the sign-in screen talks to the same door it
  already talks to.
- **Against:** today `alo-sessiond` holds no capabilities and runs no programs.
  Its unit says it is "the second privileged component", and that it is smaller
  than the first, and `crates/alo-image` holds it to that. Reading a block device
  and running `cryptsetup` would end both claims. It does not work with
  encryption off.

### B. A new, small root service that does only this (recommended)

`alo-recovering.service` would be socket-activated, run as root, and have one
door, reachable only by `alo-sessiond`. It would offer two questions:
*does this key open this machine's root?* and *put this new secret in the
person's keyslot and take the old one out*. Both would be run by the rented
tools, configured and never patched: `cryptsetup open --test-passphrase`, and
`systemd-cryptenroll` with `--wipe-slot`. The growing delay would be counted
there, in `/run`.

- **For:** the privilege is as narrow as the job. `alo-sessiond` stays what its
  unit says it is. The check can be tested from both real sides against a LUKS2
  file in a container, which is how
  `alo-encrypting/tests/the_sequence_against_a_virtual_disk.rs` already works.
- **Against:** a third privileged unit to review. It also does not work with
  encryption off, which needs C's half beside it (see the open questions below).

### C. A hash of the key, kept in the account store at install

When the installer makes the key, it writes an Argon2id hash of it into
`/etc/alo/accounts.toml`. Sign-in then checks the typed key the way it checks a
password, and needs no disk access.

- **For:** it works with encryption off, and it needs nothing new that is
  privileged.
- **Against:** it keeps a verifier of the key on the disk the key recovers. That
  is not the key itself, and with 64 characters from a 16-letter alphabet nobody
  can guess it offline, but it goes against the letter of ADR 0054 §4. It also
  goes stale whenever the disk's recovery keyslot is changed by anything other
  than alo. And it does not answer the re-keying half, so B's second question is
  still needed on an encrypted disk.

### D. The privileged broker gains a verb

- **Rejected:** the broker's list is for what an agent may do (law 2).
  Recovering a person's account is not an agent action, and a verb on that list
  is something an agent could be offered.

## The recommendation

**B on an encrypted disk, and C only on a disk the person chose not to
encrypt**, where there is no header to ask and nothing else to protect. On an
encrypted disk no hash is written, so ADR 0054 §4 holds where it means
something.

## The delay (answer 3), made concrete

- Wrong tries 1 to 5 are answered at once.
- From the sixth, each wrong try is followed by a wait: 30 s, then 60 s, 120 s,
  and so on, doubling up to one hour and never more than that. The cap is the
  lane's proposal. The owner said "longer each time" and gave no ceiling.
- The count is kept per machine, not per name, because the recovery key belongs
  to the machine. It is cleared by a right key.
- **The count lives in `/run`, so a restart clears it.** That is no way round
  the delay: after a restart the disk asks for its secret before any sign-in
  screen exists, and the disk's own tries are counted by
  `systemd-cryptsetup`. Whether that is acceptable is a question for the owner,
  below.

## Open questions for the owner

1. B, as recommended, or another option?
2. With encryption off, is a recovery key still made and written down at
   install, so that *forgot your PIN* works there too (option C's half)? Or does
   a person who turns encryption off also give up the way back?
3. The delay's ceiling (one hour, proposed), and whether a restart may clear the
   count.
4. A forgotten secret is met first at the disk's own prompt, before sign-in.
   The person types the recovery key there, and `systemd-cryptsetup` accepts it.
   Should sign-in then go straight to *choose a new PIN*, without asking for the
   key a second time? That needs a way to know that the disk was opened with the
   recovery key, and nothing reports that today. Measuring whether anything can
   is part of building B.

## What it does not decide

The screens. Those are proposed as pictures for the owner, separately. It also
leaves alone a managed machine's escrow (ADR 0004 and ADR 0054's v1 section),
which this neither builds nor blocks.
