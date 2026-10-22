# Contract — the file a machine keeps the level a person chose for running code in

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code". Added 2026-10-22.

This is the file that says which of ADR 0064's three levels the person on this
machine chose for running code: a sealed box the kernel locks down, asking each
time, or full trust. It is written when the person makes that choice, and read
by whatever is about to run code on their behalf. Nothing else in alo OS writes
it, and **there is no value in it that alo OS chooses on somebody's behalf** —
which is the whole of the next section.

Read
`docs/decisions/0064-the-person-chooses-how-code-runs-and-every-protection-they-may-change.md`
first. `crates/alo-capability`'s `how_code_runs.rs` is the three levels and what
each costs; `crates/alo-remembering`'s `the_level.rs` is the shape of this file
and `how_code_runs.rs` is the file itself.

## Not choosing is a state, and it is not the sealed box

`alo_capability::HowCodeRuns` has a default and it is the sealed box. That is
the right answer to *what does this machine do* and the wrong answer to *what
did this person decide*. `docs/features.md` puts it in one line: **the default
is that they have not chosen.**

So a reader of this file has three answers available and must keep them apart:

| | |
|---|---|
| the file is there and says a level | the person chose that level |
| **there is no file** | **the person has not chosen** |
| the file is there and will not read | something is wrong, and no level was read |

A surface that ticked the sealed box for somebody who has never opened the
setting would be showing them a default dressed as their own decision. A reader
that answered the third case with the default would be worse: a file somebody
tampered with, or a disk that would not read, reported as a level nobody picked.
`the_level_remembered` answers `Ok(Some(level))`, `Ok(None)` and `Err(..)` for
exactly these three, and that is why it does not return a bare level.

**A file that exists and omits `chose` is the third case, not the second.** No
file is a person who has not chosen; a file edited into saying nothing is
somebody who had a line and lost it, and they are told rather than quietly
returned to the default.

## Where it is

```
/var/lib/alo/how-code-runs.toml
```

Beside the grants (`grants.toml`), the pairings (`pairings.toml`) and the machine
names, in the folder the image makes for what an agent did on this machine —
`0700 alo alo` in `image/usr/lib/tmpfiles.d/alo.conf`. The path is a constant,
for the grants file's reason: where this lives is nobody's policy, and a second
copy of the answer is a second file for somebody to point somewhere the reader
is not looking.

A file of its own rather than a key in `grants.toml`, for the machine names'
reason: a grants row is a grant — who, what, until when — and a level is not
one. It is a single answer about the whole machine, with its own `format` to
move when ADR 0064 gains a level, and `grants.toml` does not notice.

## What it looks like

```toml
format = 1
chose = "asking-each-time"
```

| Key | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `chose` | The level. Required when the file exists. One of the three words below. |

| `chose` | The level |
|---|---|
| `in-a-sealed-box` | A box the kernel locks down. The default this machine behaves as when nobody has chosen. |
| `asking-each-time` | The person is asked before each run. |
| `at-full-trust` | Code runs without asking. |

A key nobody declared is refused rather than read around.

**It is words and not a number.** A level kept as `0`, `1` or `2` is a level
nobody can read in their own settings, and this file is one a person may edit,
over the single question of how much of their machine an agent may use.

## The rules it is held to

- **A spelling this does not know is refused rather than guessed.** There is no
  nearest match: a file saying `full-trust` where `at-full-trust` was meant is
  refused whole and the refusal names all three words. The two plausible repairs
   — guessing the nearest level, or falling back to the default — are *choosing
  for the person*, and law 5 gives that to nobody.
- **Who may have written it** is the grants file's three rules, asked of the
  open file: not a symbolic link, owned by root or by the login reading it,
  writable by nobody else. The file is written `0600`. **This file is the
  strongest reason that rule exists**: a file somebody else could write into
  saying `at-full-trust` is an escalation to the agent running code on this
  machine without asking, which is the thing ADR 0064 and law 2 are both about.
  A file that fails any of the three is refused; a file that is simply not there
  is a person who has not chosen.
- **Replaced whole or not at all**: written to a sibling, synced, and renamed
  over the real file.
- **No level is ever inferred from another file.** Not from the grants, not from
  the machine description, not from whether the sealed box's enforcement exists.
  An organisation may bound what is possible on a machine it owns (ADR 0016,
  `docs/contracts/machine-description.md`); it may not make this choice, because
  that would be acting as the person, which ADR 0004 forbids.

## What the three protections do not depend on

ADR 0064 keeps three things on at every level — the record, the snapshot taken
before each run, and the egress indicator — and **none of them is in this
file.** There is no key here that turns one off, at any level, and a reader that
wanted one would be looking for a choice law 5 says nobody is offered, because
restricting nothing is what makes those three exempt.

## Versioning

`format` is `1`. Anything that would stop this version reading a file correctly
raises it; anything additive does not. A file with a higher number is refused as
written for an alo OS this is not, **before `chose` is looked at** — a later alo
OS may define a level this one has no word for, and the honest report is *this
file is not for me* rather than a complaint about a word.
