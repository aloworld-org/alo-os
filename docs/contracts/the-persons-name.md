# Contract — where a person's own name is given to them

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

When a person makes their account on the first-start sheet they type a name —
`anna`. That name is kept in `/etc/alo/accounts.toml`, which is root's and mode
`0600` because it also holds every account's password hash. Their desktop runs
as their own login and must never read that file. So at every sign-in the name
is copied, by number, into a file of its own that **only that person can read**.
[ADR 0135](../decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
is the decision.

**What it is for today:** the strip across the top of alo's own terminal reads
`Local · anna`. Settings and the record may read it later, by this contract.

## Where it is

```
/var/lib/alo-names              0711 root:root
/var/lib/alo-names/<number>     0600 <number>:<their group>
```

`<number>` is the account's login number, written in decimal — `1000`.

`0711` on the folder: every person may pass through to their own file and
nobody but root may list the folder, because a listing would tell one person
who else has an account here. `0600` on the file: the person may read it, and
nobody else but root.

`alo_accounts::THE_NAMES` is the folder and `alo_accounts::where_their_name_is`
is the file, so a Rust reader names neither path itself.

## What is in it

One line of UTF-8: the name, then one line ending (`\n`). Nothing else — no key,
no quoting, no second line.

A reader believes it only if, with one trailing `\n` or `\r\n` removed, it is
**not empty**, is **at most 256 bytes**, and contains **no control character**.
Anything else is read as *no name*. `alo_accounts::as_read` is that rule and
`alo_accounts::their_name` is a reader that applies it, bounded so it never
reads more than a name could be.

**The file is the person's own, so they can write anything into it.** That is
their right — it is their name on their machine — and it is why the rule above
exists: a reader draws a name, never a terminal escape, a second line or a
megabyte.

## Who writes it, and when

**Only `alo-giving-a-name`**, as `alo-giving-a-name@<number>.service`, which
`alo-sessiond` starts at **every** sign-in after checking the number against the
accounts file, and before the session opens. It reads the name from the account
store itself — no name crosses any socket — writes a sibling `.<number>.new`,
gives it to the person, and renames it into place. So a reader sees the old name
or the new one, never half of either, and **a name changed in the account store
reaches the desktop at the next sign-in**.

The unit holds `CAP_CHOWN` and nothing else. Nothing else on the machine may
change a file's owner; `crates/alo-giving-a-folder/tests/only_here_may_give_a_directory_away.rs`
holds the whole repository to that.

## When there is no file

**No file is no name, and it is not an error.** It is the state of an account
that has not signed in since this existed, and of a machine where the helper
could not run (its journal says why). A reader shows no name — the terminal's
strip says `Local` alone.

**A reader never falls back to the login's own name.** The desktop runs as the
account `alo` on every machine, which is nobody's name.

## What a reader may not do

- **Write it.** The person may; a program reading it on their behalf may not.
- **Print it to a log.** The journal is readable by more people than the file
  is. Say *account 1000 has no name*, not what the name is.
- **Send it anywhere.** It is the person's data; a reader that needs it to
  leave the machine needs a grant like anything else.
