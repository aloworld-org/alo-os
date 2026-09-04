# Contract — what a person chose about their own machine

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file that says which model answers a person's questions and which
language they read. It is written by a settings panel, or typed by the person
whose machine it is, and it is read by whatever puts a question to a model.
Nothing else in alo OS writes it, and there is no value in it that alo OS
chooses on somebody's behalf.

`docs/contracts/machine-description.md` is the other settings file on a machine
and it is the **organisation's**: what alo OS is told about the machine, and
what an administrator may bound. This one is the **person's**, and
[ADR 0016](../decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
is why they are two files rather than two sections of one. An organisation sets
a bound — which places a question may be answered at all — and a person makes a
choice inside it. An organisation setting the choice would be acting as the
person, which ADR 0004 forbids.

`crates/alo-choosing` is what reads it.

## Where it is

```
$XDG_CONFIG_HOME/alo/settings.toml
```

and, where `$XDG_CONFIG_HOME` says nothing:

```
$HOME/.config/alo/settings.toml
```

Per-person, because a machine may have several people on it and one person's
model is not another's. It is deliberately not in `/etc`, where an
administrator reading it would be reading somebody's preferences on their own
machine.

`$XDG_CONFIG_HOME` is honoured when it is set **and absolute**, which is the
base directory specification's own rule: a relative value is invalid and is
ignored. That matters more than it reads — a relative path would put a person's
settings wherever the service happened to be started from, and somewhere else
the next time.

A login with **no home directory at all** has nowhere for this file, and alo OS
says so rather than reading somewhere under `/`. A file invented there would
belong to nobody, and on a shared machine it would be somebody else's answer to
*where do my questions go*.

## What a missing file means

**Nothing has been chosen**, which is the ordinary state of a machine nobody
has configured. It is not an error and it is not empty settings with defaults
in them: the machine answers no questions, and it says so.

That is deliberately the opposite of `docs/contracts/record-file.md`, where a
record that is not there is refused rather than read as *nothing happened*. A
record is evidence alo OS itself writes; a settings file is one somebody may
simply never have made.

## The shape

TOML, for the reason `docs/contracts/machine-description.md` is: this file is
typed by a person as often as it is written by a panel, and a comment is how
the person after them finds out why they picked what they picked.

```toml
format = 1

[answers]
catalogue = "mistral-small"

[reading]
languages = ["de", "en"]
```

**Everything except `format` is optional, and nothing has a default.** A file
with no `[answers]` is a person who has not chosen what answers their
questions. A file with no `[reading]` is a person who has not said what they
read. Neither is a mistake, and neither is filled in for them.

**A file that is there and wrong is refused whole**, and nothing in it is
honoured — not the half that parsed. Taking what read and dropping what did not
would be the machine choosing the rest of somebody's settings for them, quietly,
in the release that renamed a key.

**A key nobody declared is refused**, naming it. A newer alo OS may add keys;
what says so is `format`, and a typo is not an addition.

## `format`

| Field | Meaning |
|---|---|
| `format` | Which shape these settings are in. Required. `1` today. |

Settings that say anything but `1` are **refused rather than guessed at**, and
the refusal names both numbers. It is answered before any other value in the
file, so settings written for a later alo OS are refused as such rather than as
whichever of their keys this alo OS happened not to know.

The reason is not tidiness: a newer alo OS may let somebody choose a place this
one cannot honour, and a machine that read the file part-way would answer their
questions somewhere they did not pick while showing them a settings panel that
says otherwise.

## `[answers]` — what answers this person's questions

**One key, and the key is the list.**

| Key | Meaning |
|---|---|
| `catalogue = "<name>"` | A model in the catalogue alo OS ships, named exactly as the catalogue names it. |
| `brought = "<name>"` | Weights the person brought themselves, named exactly as that list names them. |

There are two lists of models on a machine — the catalogue and the weights
somebody added — and neither knows about the other. A model called
`mistral-small` in the catalogue and a file somebody brought under the same name
are two different answers to *what runs my turn*, so the choice records **which
list** as well as which entry.

- **Two keys at once is not a choice** and does not read. Whichever one a reader
  took would be the machine picking between them.
- **A key that is neither is refused**, naming the two that are. A provider and
  a machine in the next room are both places ADR 0008 permits and neither is
  here yet, because this machine keeps no list of either — so such a file fails
  to read rather than reading as a setting that quietly does nothing.
- **A list named with no model** — `catalogue = ""` — is refused rather than
  read as a person who chose nothing. They chose, and what they chose is not a
  model.

The name is kept **exactly as it was written**. A runtime matches the name it
was given, and trimming or lower-casing it would be alo OS quietly asking for a
different model than the one somebody picked.

## `[reading]` — what this person reads

| Key | Meaning |
|---|---|
| `languages` | The languages they read, best first, as tags: `["de", "en"]`. |

Tags are written the way the rest of the world writes them: `de`, `pt-BR`,
`sr-Latn-RS`. A list rather than one language, because a person names their own
second language and alo OS infers nothing from a first. The broader form of each
— `pt` behind `pt-BR` — is worked out by `alo-strings` and is not written here.

**Something that is not a tag refuses the file**, quoting back what was written
where a language belongs. That is the one place in alo OS where a refusal cannot
be in the language the refusal is about: the person's language lives in the file
that did not load, so what they read it in is whatever the machine was already
showing. It is stated rather than solved, because the alternative is a machine
guessing at a language from a file it has just refused to believe.

## When the choice is outside what the organisation permits

**The bound wins, and the person is told.** A choice at a place the
organisation's `SourcePolicy` forbids is refused, in the policy's own words,
naming the place it refused. It is never quietly swapped for a permitted one.

Silent substitution is the failure worth naming because it is the comfortable
one to build: a person picks a hosted provider, policy forbids egress, and the
machine answers anyway using something local. Nothing appears broken, the person
believes they know where their question went, and they are wrong.

On a machine no organisation manages the bound is simply **absent** — not empty,
not a file full of permissions — and the person chooses freely.

## Anything secret

**Nothing.** There is a model name and a list of language tags in this file and
there is nothing else. A provider's key lives in the keyring and never in a
settings file (`crates/alo-models/src/provider.rs`), and that is not relaxed
here.

## What changes additively

New keys may be added and `format` stays `1` for as long as an older alo OS
reading the file without them would honour the same choice. A new place a
question can be answered — a provider, a paired machine — is a new key under
`[answers]`, and an alo OS that has never heard of it refuses the file rather
than answering somewhere the person did not pick. Anything else — a key removed,
a meaning changed, a default introduced — is a new `format`, and settings with a
number this alo OS does not read are refused.
