# ADR 0121 — The thing that opens a session makes the person's folder

**Status:** accepted — it amends
[ADR 0024](0024-what-a-person-signs-in-at.md)'s charter for `alo-sessiond` by
one sentence, and it is what finally calls
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md)'s
helper
**Date:** 2026-10-17
**Context:** [ADR 0045](0045-what-undoing-rewinds-to.md) term 8 (a privileged
step at first sign-in), [ADR 0024](0024-what-a-person-signs-in-at.md) (what
`alo-sessiond` is and is not),
[ADR 0118](0118-more-than-one-person-on-a-machine.md) (more than one person),
[ADR 0119](0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md),
[ADR 0120](0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md),
`crates/alo-sessiond`, `crates/alo-brokerd/src/units.rs`,
`docs/contracts/kept-undo-folder.md`

## The decision in one line

`alo-sessiond` asks systemd to start `alo-giving-a-folder@<account>.service`
when it opens a session, **after** it has checked that the account exists and
that nobody is signed in, and **a failure does not refuse the sign-in.**

## What this is for

ADR 0119 built the privileged step ADR 0045 term 8 asked for, and ADR 0120 put
it where root can run it. **Nothing started it.** The helper shipped on the
image and no line of code on the machine ever asked for it, which means the
folder was still absent at runtime on every machine — the same *built and never
called* state three other components were found in on 2026-10-16.

## Why the thing that opens sessions, and not the thing that makes accounts

The obvious caller is account creation, and it was the first design. It is
wrong, and the reason was found by reading the code rather than by reasoning:

```rust
// crates/alo-greeting/src/greeting.rs:208
if self.accounts.how_many() != 0 {
    return Greeted::Refused(crate::words::ALREADY_HAS_AN_ACCOUNT.key());
}
```

**`made_an_account` is the first-account road and refuses to be anything else.**
So an account road reaches nobody who already has an account — which is every
person on every machine that has been switched on once, including the one this
repository is developed against. A machine would have to be reinstalled to get
a folder.

Covering those would need a **second** caller: something at boot that walks
`/etc/passwd` and filters it, because `Accounts`' public surface is `none`,
`created`, `password_set_again`, `signs_in`, `how_many`, `numbers` and
`name_of` — **no enumerator**, deliberately, and `numbers`' own rustdoc says it
"is not an enumeration door".

**Everybody signs in; not everybody is created.** So the sign-in road is one
call site where the account road is two, it needs no new surface on `Accounts`,
and it reaches a person whose account is five years old. That is the whole
argument.

Three smaller reasons, in the order they weigh:

- **The number is already in hand and already checked.** `Opening::heard` asks
  `an_account_numbered` one step above, so no second answer to *is this a
  person* is introduced. **This is a correctness argument and not a safety one**,
  and it was first written here as though it were safety — a folder made for a
  number that is not a person's is a `0700` directory nobody can use inside a
  `0711 root:root` folder, which is untidy and inert. The Mac lane claimed the
  safety reading and the desktop lane pointed out that it contradicted the same
  lane's own measurement; the weaker, true version is what stands.
- **The port pattern is already there.** `Opening<A: TheAccounts, L: Logind>`
  takes its machine as traits one method wide, with the Linux halves in their
  own modules and the refusals tested on any host. A third is the shape the file
  already has.
- **The blast radius.** `Greeting<K: Knocking>` carries its door as a type
  parameter, so a port on it changes every construction site in `alo-shell`,
  `alo-leaving`, `alo-locking`, `alo-saying`, `alo-entering` and
  `alo-notifying`. `Opening::at` has one real caller and sixteen in tests.

## Why not `alo-brokerd`, which already starts units

Raised by the desktop lane, and it is the strongest alternative:
`alo-brokerd/src/units.rs` already asks systemd to start units, with a closed
enum of unit names and a refusal for every method that is not read-or-start. The
mechanism and the discipline both exist there, and using them would leave ADR
0024 untouched.

**Rejected on where it would be written down.** `alo-brokerd`'s surface is
*verbs the agent asks and the person approves* — its own table reads *"which
verbs this machine carries out, and the refusal for every other, by name"*. And
`docs/contracts/kept-undo-folder.md` says of this folder:

> **Nothing an agent can reach is in it.** There is no verb over this folder and
> there is not going to be one (ADR 0045's seventh term): an agent that can
> forget an undo can erase the evidence of what it did.

A third variant beside `Carriers` would not be a verb on the day it landed. The
risk is not that day; it is the lane that opens that file next year to add a
verb and finds the undo folder already named in it. **The record is the cheaper
thing to amend.**

## The amendment to ADR 0024, exactly

`crates/alo-sessiond/src/lib.rs` said:

> **No path reaches it.** It is handed nothing to open, write, or run.

*run* is now untrue and the sentence has been rewritten rather than quietly
left. What replaces it is bounded by four things, each a test:

1. **One unit**, named by `THE_UNIT`, a constant of this repository.
2. **Instanced by a `u32`**, so the only text in a unit name is that constant —
   `CLAUDE.md` law 2, and
   `tests/this_component_only_starts_one_unit.rs` reads the source for
   `format!("{THE_UNIT}@{person}.service")` and for the parameter's type.
3. **Three methods**, `StartUnit`, `GetUnit` and `Get`, in `EVERY_METHOD`, with
   the caller refusing anything not on that list at run time. The same test
   reads the file for the twelve systemd verbs that stop, kill, mask, reload,
   enable or write, and finds none — with a control proving the filter can find
   one.
4. **No new capability, and that is measured rather than asserted.**
   `alo-sessiond.service` still says `CapabilityBoundingSet=` and
   `AmbientCapabilities=`, both empty. What authorises starting a unit is the
   caller's **uid**, not a capability — the same thing ADR 0024 rests on for
   `CreateSession`. A program calling this crate's own port was run under
   exactly those settings:

   ```
   User=root  CapabilityBoundingSet=  AmbientCapabilities=  NoNewPrivileges=yes
   Result=success
   OK: the port says account 1000 has their folder
   /var/lib/alo-undo/1000  mode=700  owner=alo
   ```

   **And the control refuses.** The same program as the person, not root:

   ```
   ERR: systemd would not start alo-giving-a-folder@1000.service for account 1000:
        org.freedesktop.DBus.Error.InteractiveAuthorizationRequired
   /var/lib/alo-undo/1000: No such file or directory
   ```

   So *uid and not capability* is a measurement with both answers, and the error
   name this crate decides on is the one that really arrives.

**What it still does not do** is unchanged: it does not authenticate, it does
not draw, no password or name reaches it, and it is handed nothing to open or
write.

## A failure does not refuse the sign-in, and that is the decision

The folder is the only step in `Opening::heard` whose failure is not a refusal.
Three reasons, and the third decides:

1. **It is not what they came for.** A person turned away from their own machine
   over a directory for a feature nothing yet writes to is a machine refusing
   somebody for a reason they have never heard of.
2. **There is already an honest answer for a missing folder.**
   `alo-letting-go` answers *not yet on this machine*, and ADR 0045's sixth term
   makes that a real answer rather than an error. A person whose folder is
   missing is in exactly the state of a person on a machine installed before the
   filesystem was decided.
3. **`CLAUDE.md` law 5.** Refusing would take the machine away from its owner to
   protect a feature, and *a change that takes a choice away from the person is
   a bug, whatever the reason given for it.*

So it goes to the journal, and `opening.rs`'s
`a_folder_that_was_not_made_refuses_nobody` is what holds it — which exists
because the obvious way to write that call site, a `?` or a match returning a
refusal, is one line from being the opposite. **Mutation-proved**: adding
`return Answered::Refused(NOT_OPENED.key())` to the failure road fails that one
test and no other.

## Where in the order, and why there

```
1. is this the sign-in surface?        refuses
2. is there anybody with that number?  refuses
3. is somebody signed in already?      refuses
4. their folder                        does not refuse      <- new
5. open the session
```

**After 3**, so a knock nobody asked for causes no privileged act —
`a_second_knock_asks_for_no_second_folder` and `a_refused_knock_asks_for_no_folder`
are what hold that, the second over both a caller who may not knock and a number
nobody here has. **Before 5**, because the session is what writes into the
folder, and a session opened first would have a window in which it could not.

It is also **asked for exactly once per knock that gets that far**, which is
`CLAUDE.md`'s *one approval causes exactly one execution* in the shape this
component has it: `the_folder_is_asked_for_once_for_one_knock` counts the calls.

## Waited for, not fired and forgotten

`StartUnit` answers as soon as the job is enqueued, which says nothing about
whether a directory exists. The unit is `Type=oneshot`, so there is a result to
read, and `made_for` polls `ActiveState` and then `Result` until the unit is at
rest.

**Bounded by ten seconds, and the bound is about the person rather than the
work.** The unit makes two directories and changes one owner. Ten seconds is
three orders of magnitude more than that needs, and running out of it is not an
error about the folder — it is a decision not to keep somebody standing at a
sign-in screen. Running out is reported as a failure and, like every other
failure here, signs them in.

## Consequences

- **`machine.rs` was split in the same change.** Adding systemd's manager to the
  file that holds `logind` took it to 475 lines with two reasons to change;
  `CLAUDE.md` law 4 says split in the change that discovered it, so
  `making_a_folder.rs` is its own file. *It was renamed
  `crates/alo-sessiond/src/starting_their_units.rs` on 2026-10-28, when
  [ADR 0135](0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
  gave it a second unit to start in the same shape.*
- **`alo-sessiond` gains no dependency.** It already carries `zbus` for
  `logind`, and the unit's name is read off the image rather than imported from
  `alo-giving-a-folder` — a constant compared with another crate's constant
  passes while both are wrong, and the image's directory is the artifact a
  machine is built from.
- **No new word.** Nothing new is said to a person, because nothing new is
  refused, so the vocabulary is untouched and there is no snapshot to
  regenerate.
- **The folder is still not written to.** Nothing takes a bracket —
  `alo-turn`'s, unchanged — so what this makes true is that the folder exists
  for every person who signs in, which is the precondition the bracket was
  blocked on rather than the bracket.
- **It has not been measured over a folder that already has something in it.**
  The remover's own road is unreachable on a non-`btrfs` machine (ADR 0045 term
  6), so on the machine this was written on `alo-letting-go` returns at its first
  question. That gap is named in the task report and a `btrfs` machine is
  booked.
