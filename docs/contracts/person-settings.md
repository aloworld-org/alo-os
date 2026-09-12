# Contract — what a person chose about their own machine

**Status:** contract. Additive changes only; a break requires versioning and a
deprecation period. See `CLAUDE.md`, "Contracts outlive code".

This is the file that says which model answers a person's questions, which
weights they brought to the machine themselves, which language they read, and
whether they have been asked at setup and answered. It is written by a settings
panel or by setup, or typed by the person whose machine it is, and
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

`crates/alo-choosing` is what reads it, and since 2026-09-11 what writes it:
`alo_choosing::Settings::at` is the way in and `alo_choosing::Choosing` is the
way out. `crates/alo-setting-up` is the first-time flow and writes through that
same way out and no other. See "Writing it" below.

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
format = 3

[answers]
catalogue = "mistral-small"

[setup]
answered = true

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
their own. A file with no `[setup]` is a person nobody has asked. None of the
four is a mistake, and none is filled in for them.

**There is no address for a runtime on this machine, and there will not be
one.** Where it is, is the adapter's own knowledge and nothing else's —
[ADR 0019](../decisions/0019-a-runtime-is-found-not-configured.md) says why, and
says it positively so a later reader does not add the key believing it was an
oversight. A runtime somewhere else is a **provider**, which is a different key
in a different shape, and since format 2 it is here: `[[provider]]` below.

**There is no credential in this file, and there is nowhere to put one.** A
provider's key lives in a keyring under a name **derived** from the provider's
own name, so the file has no field to paste one into, and a file that invents
one is refused naming the key. See `[[provider]]`.

**A file that is there and wrong is refused whole**, and nothing in it is
honoured — not the half that parsed. Taking what read and dropping what did not
would be the machine choosing the rest of somebody's settings for them, quietly,
in the release that renamed a key.

**A key nobody declared is refused**, naming it. A newer alo OS may add keys;
what says so is `format`, and a typo is not an addition.

## `format`

| Field | Meaning |
|---|---|
| `format` | Which shape these settings are in. Required. `3` today; `2` and `1` are still read. |

**`2` since providers and `3` since `[setup]`**, and **both older shapes are read
exactly as they always were** — a machine configured before either existed keeps
working, nothing rewrites its file and nobody is asked to. That is expand, then
migrate, then contract, and nothing here is the contract yet.

A file that says `1` and contains a provider — chosen or listed — is **refused**,
naming the number it needs. So is a file that says `1` or `2` and contains
`[setup]`. Their keys parse either way, and honouring them would be this machine
believing whichever half of a disagreement it preferred — and in `[setup]`'s case
it would tell the machine that somebody had answered a question this alo OS could
not have put to them, so setup would never be shown to a person who has never
seen it.

Settings that say a shape this alo OS does not read are **refused rather than
guessed at**, and the refusal names both numbers. It is answered before any other value in the
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
| `provider = { name = "<provider>", model = "<model>" }` | A provider from `[[provider]]` below, and the model to ask it for. Format 2. |

These are the three choices `docs/features.md` names — **local models**, **your
own API provider**, and **alo**, which is the second one because
[ADR 0014](../decisions/0014-alos-own-model-is-a-provider-like-any-other.md)
makes alo's service one more provider with no special case anywhere. They are
**model-source choices and not privacy levels**: where a question is answered
follows from the choice and is not the choice.

The provider form carries **two** names where the others carry one, and they are
different pairs: on this machine it is *which list* and *which entry*; for a
provider it is *which provider* and *which of its models*.

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

## `[[provider]]` — the providers this person added

Format 2. An array of tables, so a person who has added none simply has no
`[[provider]]` in their file.

| Key | Meaning |
|---|---|
| `name` | What they call it, and what an answer says it came from. Matched case-insensitively against `[answers] provider.name`; two providers of one name is refused, because *answered by Mistral* would not say which. |
| `endpoint` | Where it is. `https://` unless it is on this machine — a key over plain `http://` to anywhere else is refused, and "it is only our internal network" is how that gets shipped. |
| `region` | Where it runs, **as stated by whoever added it**. Optional; absent is *unknown*. Never inferred from the address: `api.example.fr` is not evidence of anything, and a guess here would hand somebody a reassuring label while putting them in breach. |
| `needs-a-key` | Whether it is asked for a credential. Optional, and **absent means yes**, because almost every hosted API needs one. A compatible service that takes none says `false`, and then nothing is looked up and nothing is sent. |

**There is no `key`, and that is the protection rather than an omission.** The
single most reliable way for a credential to end up in a text file in somebody's
home directory is for the file to have a field called `key`. So it has none: the
keyring name is derived as `provider/<name>`, and a file that invents a `key` is
refused naming it — the person is told, rather than left with a credential on
their disk that alo OS quietly read.

```toml
format = 3

[answers]
provider = { name = "Mistral", model = "mistral-small-latest" }

[[provider]]
name = "Mistral"
endpoint = "https://api.mistral.ai"
region = "the EU"
```

**`[answers] provider.name` must name an entry in `[[provider]]`**, and a file
whose two halves disagree is refused whole — the same rule `brought` is held to.

**A provider choice is never answered on this machine.** Not if a model of the
same name is in a list here, and not if the provider cannot be reached. What
happens then is a refusal naming the reason; what does not happen is another
place answering in its stead.

**A provider that needs a key cannot be asked on this machine yet.** alo OS holds
a reference to where a key lives and there is no store behind that reference,
so such a choice is read, kept, and refused at the moment of asking — never sent
without its key.

## `[setup]` — whether this person has been asked, and answered

Format 3. One key, and it records **one bit**: the question was put to this
person and they answered it.

| Key | Meaning |
|---|---|
| `answered` | Whether setup was put to this person and answered by them. `true`, or absent. `false` says what absent says and is read as such — there is no reading of it under which anybody was asked. |

**It is not a second copy of the choice.** What was answered is `[answers]`, and
nothing here can disagree with it: `[setup] answered = true` with no `[answers]`
is [ADR 0009](../decisions/0009-a-good-computer-without-the-agent.md)'s fourth
choice — *no model, no provider, no agent* — and `[setup] answered = true` beside
an `[answers]` is a person who chose a source. There is no rule keeping the two
in step because there is nothing to keep in step.

**It exists because two states of this file would otherwise be one file.** A
machine nobody has configured and a machine whose owner said *not at all* both
have nothing answering questions. They are different machines: one is waiting to
be asked and the other is finished. Without this key, setup would be shown again
to every person who declined it, which is ADR 0009's *no nagging* broken by the
one mechanism guaranteed to meet all of them.

**Nothing but the person writes it.** It is absent on a machine nobody has
configured, absent in every file alo OS writes for somebody who has not answered,
and set only by `alo_setting_up::SettingUp::answer` — through
`alo_choosing::Choosing::setting_up`, which is the door
[ADR 0016](../decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
keeps for them. An image that shipped with this set would be
[ADR 0025](../decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)'s
rejected Option B with the mechanism moved somewhere nobody would look for it.

**Changing what answers your questions afterwards does not change it.** A person
who chose at setup and later clears the setting in Settings has answered setup
and has nothing answering questions; a person who typed their own settings file
and never saw setup has something answering questions and has not been asked.
Both are true sentences about a machine, and both are expressible here.

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

## Writing it

A settings panel does not compose this file as text. `alo_choosing::Choosing`
takes a path, holds what the file at it says, and has one door per thing
ADR 0016 gives the person: what answers their questions, weights they brought, a
provider they added, a provider they changed, and the languages they read. Every
door takes a value some crate has already checked — there is no door that takes
text, and none that takes a fragment of this file.

**A provider's address is judged again at the write, whichever door it came
through.** `alo_models::Provider` has public fields, so a surface can hold one
that `Provider::checked` never made — most often a working provider whose
address somebody edited to `http://` on their own network. The writer asks
`alo-models`' rule again before a byte is written: an address that is not
`https://` is refused unless it is a service on this machine (`127.0.0.1`,
`::1`, `localhost` — the one exception, on any scheme), and the person reads
that crate's own sentence, *use https, or a service on this machine*. An
address on the machine's own network is not an exception; *it is only our
internal network* is the sentence the rule is written against. There is no
list of permitted hostnames and no environment variable that turns it off. A
file written before this rule is read exactly as it always was and is never
rewritten behind the person; the refusal is at the next write.

**Changing a provider replaces it where it stands.** It is matched by name, the
way the list matches — case does not count — and keeps its place in the file,
so the order providers were added in is still the order of `[[provider]]`. A
provider the list does not have is refused rather than added, because *change*
and *add* are two different things a person did.

**Whole or not at all.** The change is applied to a copy of the settings, the
copy is written to a sibling file and renamed over the real one, and only then
does it become what the machine will read. A change refused therefore leaves
this file byte for byte as it was, and the choice made before it is still in
force. What a person is told says exactly that, and it is a different sentence
from the one said about a file that would not read: *nothing in your settings has
been changed*, rather than *nothing in the file has been used*.

**A file that is not there is written.** The first choice somebody makes on a
machine is made when neither this file nor the directory around it exists, so
the directory is created (`0700` on a machine with modes) and the file goes down
`0600`. That is the opposite of `/var/lib/alo`, which belongs to the image and is
never created by anything that writes into it.

**A file that is there and does not read is never written over.** Opening
settings that do not hold is refused, naming the file — because a surface that
read a typo as *nothing chosen* and then saved would take away the keystroke
that was about to fix it.

**What is written is the `format` this alo OS writes**, which is `3`. A file
saying `1` or `2` is read exactly as it always was and nothing rewrites it unasked; a
file this alo OS writes says the shape this alo OS writes, because a writer
choosing among past shapes would grow one branch per format for ever. Going
backwards after a change costs a person their settings rather than their choice,
which is the direction this file has always failed in.

**Nothing is written that cannot be read back as the same settings.** The change
is serialised, parsed again by this file's own reader, and refused unless what
comes back is what went in. Two things a caller can hold and this file cannot
say are refused by it rather than dropped: the list of model names a provider
offers, and a credential kept under a name other than the derived
`provider/<name>`. A change reported as made and afterwards described
differently by the machine that made it is the defect that check exists for.

**Nothing is chosen for anybody.** Opening a person's settings writes nothing —
not a file, not a directory, not a `format` line. The first byte is written by
the first choice somebody makes.

**A running daemon is not told, because it does not need to be.** `alo-agentd`
reads this file once a turn, at the first question of that turn, so a change
written here is in force for the next question anybody asks. There is no knock
and no message: that is the opposite answer from `/var/lib/alo/grants.toml`,
which a daemon holds from start-up and is told about through
`alo_protocol::FromAPerson::Granted`.

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
is a new key, `format` stayed `1`, and an older alo OS meeting it refuses the
file. Going backwards costs a person their settings rather than their choice,
which is the direction this file has always failed in — nothing is honoured
part-way, and they are told.

**`[[provider]]` and `[setup]` are what the other half looks like.** Each is a
new key that changes what an older alo OS would *do*, so each took a number:
`2` and `3`. The test is not whether the key is new, it is whether an alo OS
that ignored it would still honour the same choice. Ignoring `[[provider]]`
would answer somebody's question in a place they did not pick. Ignoring
`[setup]` would tell a machine that a person who declined an agent had never
been asked, and put setup in front of them again at every sign-in. Neither is a
key an older release may quietly not know about, so neither is additive in the
sense above — and both older shapes are still read, which is what the rule is
really protecting.
