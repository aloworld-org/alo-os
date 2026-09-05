# Contract — what a person chose about their own machine

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file that says which model answers a person's questions, which
weights they brought to the machine themselves, and which language they read. It
is written by a settings panel, or typed by the person whose machine it is, and
it is read by whatever puts a question to a model. Nothing else in alo OS writes
it, and there is no value in it that alo OS chooses on somebody's behalf.

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

[[brought]]
id = "my-finetune"
bytes-on-disk = 4700000000
quantisation = "Q4_K_M"
drives-verbs = "reliably"
```

**Everything except `format` is optional, and nothing has a default.** A file
with no `[answers]` is a person who has not chosen what answers their
questions. A file with no `[reading]` is a person who has not said what they
read. A file with no `[[brought]]` is a person who has brought no weights of
their own. None of the three is a mistake, and none is filled in for them.

**There is no address in this file, and there will not be one.** Where a model
runtime on this machine is, is the adapter's own knowledge and nothing else's —
[ADR 0019](../decisions/0019-a-runtime-is-found-not-configured.md) says why, and
says it positively so a later reader does not add the key believing it was an
oversight. A runtime somewhere else is a **provider**, which is a different key
in a different shape and is not here yet.

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

**`brought = "<name>"` must name an entry in `[[brought]]` below**, and a file
where it does not is refused whole, quoting the name back. The two halves
disagree, and honouring either would be the machine deciding which of them the
person meant — most often a name typed twice with one letter different.

**`catalogue = "<name>"` is deliberately not checked against anything.** The
catalogue ships with the release rather than living in this file, and a model
already on somebody's own disk is theirs to ask: alo OS gates what it *offers to
fetch* on a licence, never what somebody may run on hardware they own.

## `[[brought]]` — weights this person put on the machine themselves

An array of tables, one per set of weights, in the order they were brought.
[ADR 0019](../decisions/0019-a-runtime-is-found-not-configured.md) puts the list
here rather than in a store of its own: the weights are the person's, they
fetched them, the licence they accepted is theirs, and this file already holds
*which model answers*. A second store for one owner is how a settings system
becomes six.

| Key | Meaning |
|---|---|
| `id` | What the model runtime on this machine answers to. Required, matched exactly. |
| `bytes-on-disk` | What the weights take on this machine's disk, as the runtime reported it. Required. |
| `quantisation` | The quantisation the runtime reports, where it says. Optional — a runtime does not always say. |
| `drives-verbs` | What a measurement of these weights earned: `"reliably"`, `"sometimes"`, `"rarely"` or `"not-measured"`. Required. |

- **`drives-verbs` has no default and an entry without it does not read.** *Not
  measured* is a thing to state, not a blank to leave: an entry that said nothing
  would read as *probably fine*, and alo OS gives an agent turn only to weights
  a measurement has cleared.
- **Two ids differing in case are two entries.** An id is a name a runtime
  answers to rather than a word a person chose, so it is matched exactly — which
  is the opposite answer from a provider's name and is the same answer as every
  other identity in alo OS.
- **The same id twice is refused**, naming it. alo OS says which model answered a
  question, and with two entries under one name it could not.
- **Weights with no name are refused**, because there would be nothing to ask the
  runtime for.

**alo OS states no licence for anything on this list and does not pretend to
have checked one.** There is no licence key here and there is nowhere to put
one: what somebody brings is theirs, including its terms. The catalogue is where
alo OS states licences, because offering something is what makes a licence ours
to state.

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

**Nothing.** There is a model name, a list of weights on this person's own disk
and a list of language tags in this file, and there is nothing else. A provider's
key lives in the keyring and never in a settings file
(`crates/alo-models/src/provider.rs`), and that is not relaxed here.

## What changes additively

New keys may be added and `format` stays `1` for as long as an older alo OS
reading the file without them would honour the same choice. A new place a
question can be answered — a provider, a paired machine — is a new key under
`[answers]`, and an alo OS that has never heard of it refuses the file rather
than answering somewhere the person did not pick. Anything else — a key removed,
a meaning changed, a default introduced — is a new `format`, and settings with a
number this alo OS does not read are refused.

`[[brought]]` arrived that way and is what the rule looks like in practice: it
is a new key, `format` stays `1`, and an older alo OS meeting it refuses the
file. Going backwards costs a person their settings rather than their choice,
which is the direction this file has always failed in — nothing is honoured
part-way, and they are told.
