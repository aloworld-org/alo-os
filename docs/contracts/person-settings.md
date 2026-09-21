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

### The folder, and the files beside this one

`$XDG_CONFIG_HOME/alo/` (or `$HOME/.config/alo/`) is **the person's folder**,
and `settings.toml` is one file in it.
[ADR 0038](../decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md)
puts a person's other settings beside it, one file per crate that owns the
shape. `alo_choosing::where_the_folder_is` works the folder out by exactly the
rule above, and whoever starts a session hands each of those crates the path it
keeps at, so none of them reads an environment.

Every file in the folder other than this one is kept by one rule, held in type
by `crates/alo-kept`: it begins `format = N` with a number of its own; it holds
only what the person changed; no file means they changed nothing; a file that is
there and wrong — not TOML, no `format`, another format, a key not on the list,
or a value the shape refuses — is refused whole and nothing in it is honoured;
and it is written whole to `<file>.new`, read back off the disk as the same
value, and only then renamed over the old one. Which files, and their keys, are
sections of this contract as those crates gain them.

Since 2026-09-21 there are **ten**, each read and written by the crate that
declares its shape, at a path that crate is handed, and by nobody else:

| File | Kept by | `format` | Keys besides `format` |
|---|---|---|---|
| `appearance.toml` | `alo_appearance::keeping` | `1` | `background`, `displays`, `lock`, `following`, `text`, `accent` |
| `dock.toml` | `alo_dock::keeping` | `1` | `edge` |
| `shortcuts.toml` | `alo_shortcuts::keeping` | `1` | `changed` — one `[[changed]]` table per action, with `action` and, unless the person wants no shortcut for it, `chord` |
| `what-opens-what.toml` | `alo_applications::keeping` | `1` | `kinds` — a table from a kind of file to the identifier of the application the person chose to open it |
| `sleeping.toml` | `alo_sleeping::keeping` | `1` | `keep-awake`, `lid` |
| `displays.toml` | `alo_displays::keeping` | `1` | `arrangements` — one `[[arrangements]]` table per set of screens, each holding its own `screens`; `night-light` |
| `leaving.toml` | `alo_leaving::keeping` | `1` | `reopen`; `was-open` — one `[[was-open]]` table per window that was open |
| `notifying.toml` | `alo_notifying::keeping` | `1` | `do-not-disturb`, `quiet-hours` |
| `keyboards.toml` | `alo_keyboards::keeping` | `1` | `layouts`, `compose`, `input-methods` |
| `gestures.toml` | `alo_desktops::gesture_files` | `1` | `scroll`, `pinch`, `three-finger-swipe`, `four-finger-swipe`, `scrolling` |

**That number is true because a test reads it.** The sentence above said *four*
from 2026-09-15 until 2026-09-21 while the folder grew to ten, and it was the
count rather than the omission that did the damage: somebody writing a backup
tool, a migration or an organisation's provisioning from a contract that names
four files writes something that silently drops the rest of what a person
chose, and has no reason to look. So the count is no longer prose. Each of
`sleeping.toml`, `displays.toml`, `leaving.toml` and `notifying.toml` is held
to this table by its own crate's `tests/the_contract_describes_this_file.rs`,
and each of those four reads the number in the sentence above against the
number of rows in the table. A row added without the number moving, or the
number moved without a row, fails in the change that does it.

**Eight of the ten have a section of their own below.** `keyboards.toml` and
`gestures.toml` are in the table because they exist and a reader needs to know
they do; their sections, and the tests that would hold them, are owed by the
crates that keep them. Those two rows are the only ones nothing yet checks —
read them as a pointer to `alo_keyboards::keeping` and
`alo_desktops::gesture_files`, where the two shapes are actually declared.

A file that did not read is answered, by each crate's `keeping::at_sign_in`,
with what the release ships and the refusal beside it — naming the file, and the
key when a key was what was wrong. Nothing watches these files: a hand edit is
read at the next sign-in. Each section below —
[`appearance.toml`](#appearancetoml--how-this-persons-machine-looks),
[`dock.toml`](#docktoml--which-edge-the-dock-is-on),
[`shortcuts.toml`](#shortcutstoml--the-shortcuts-this-person-changed),
[`what-opens-what.toml`](#what-opens-whattoml--which-application-opens-each-kind-of-file),
[`sleeping.toml`](#sleepingtoml--what-closing-the-lid-does-and-what-keeps-this-machine-awake),
[`displays.toml`](#displaystoml--where-this-persons-screens-are-and-when-they-warm),
[`leaving.toml`](#leavingtoml--whether-what-was-open-opens-again) and
[`notifying.toml`](#notifyingtoml--when-notifications-are-held) — gives its
keys, its `format`, what a missing file means and what a file that does not
read is told, each held to the crate that keeps it by a test in that crate
(`tests/the_contract_describes_this_file.rs`).

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

[brought.measured]
machine = "Apple M3, 8 GB unified memory"
date = "2026-09-14"
runtime = "Ollama 0.34.0"
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
one is refused whole, naming the file. See `[[provider]]`.

**A file that is there and wrong is refused whole**, and nothing in it is
honoured — not the half that parsed. Taking what read and dropping what did not
would be the machine choosing the rest of somebody's settings for them, quietly,
in the release that renamed a key.

**A key nobody declared is refused**, and the person is told which file —
`choosing.settings.not-understood` — **without the key being repeated back**.
That is the one place this file answers differently from the files beside it
(below), and it is deliberate: a key nobody declared is exactly where a pasted
credential lands, TOML accepts one as a bare name, and a refusal that quoted it
would carry it into every log line and support bundle that formatted the
refusal. What survives, for whoever reads the log, is the parser's sentence
with every quoted run that is not one of this format's own words replaced by
`…`, and the line and column it stopped at (`crates/alo-choosing/src/unreadable.rs`).
A newer alo OS may add keys; what says so is `format`, and a typo is not an
addition.

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
| `machine = "<identity>"` | A machine this person is paired with, by the identity its pairing names, whose own model answers. Additive; format 3 unchanged. See below. |

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
- **A key that is none of the four is refused**, naming the ones that are, so
  a file naming a place this alo OS does not know fails to read rather than
  reading as a setting that quietly does nothing.
- **A list named with no model** — `catalogue = ""` — is refused rather than
  read as a person who chose nothing. They chose, and what they chose is not a
  model. `machine = ""` is refused the same way.

### `machine = "<identity>"` — a machine down the corridor

*A machine without a GPU discovers the one with it, and the agents just work.*
The value is the other machine's **identity** — thirty-two lowercase
hexadecimal characters, the string its pairing names — and never an address or
a name: an address is measured by discovery at the moment of asking and is
never typed or kept (ADR 0003), and a name is what the person here called it,
which can change without the choice changing.

```toml
format = 3

[answers]
machine = "aaaabbbbccccddddeeeeffff00001111"
```

- **It carries no model.** The machine down the corridor puts the question to
  the model **its** person chose (`docs/contracts/local-network-wire.md`, *the
  question path*), and names that model in its answer. Which model runs there
  is that person's setting (ADR 0008), not this one's.
- **It is refused at choosing** when no pairing permits asking that machine's
  models: `alo_choosing::Choosing::answered_by_a_paired_machine` asks the
  pairings as they stand (`alo_choosing::WhoMayBeAsked`, answered by whoever
  holds them) and writes nothing otherwise — *your settings would say the
  machine … answers your questions, and no pairing with it lets it answer them
  at the moment, so nothing in your settings has been changed*.
  `alo_choosing::AMachine::permitted` is the only way to make the choice.
  A person's shell reaches it through the daemon: `choose-machine-to-answer`
  on the person's door (`docs/contracts/daemon-protocol.md`) is answered with
  the daemon's own pairings as the `WhoMayBeAsked`, and that request is the one
  place `alo-agentd` writes this file — only because the person's shell asked.
- **It is not checked when the file is read.** A pairing ends or is revoked and
  the file outlives it; a settings file is not wrong because an expiry passed
  overnight. What is refused instead is **the question**, at every question, in
  words and with nothing sent.
- **An identity nobody is paired with reads**, exactly as it was written, and
  is refused at the question as paired with nothing — which is what it is.
- **It is a place, so an organisation's bound applies**:
  `SourcePolicy::ThisMachineOnly` refuses it, and `InTheBuilding` and a region
  permit it (a paired machine stays in the building and is in the customer's
  region by definition). The bound is asked before anything leaves.
- **It is never a fallback, in either direction.** A question for a paired
  machine that is not answered is not put to anything on this machine or to a
  provider, and a question for either of those is never put to a paired
  machine.
- **Additive, and the format number did not move** — `[[brought]]`'s
  precedent: an alo OS from before the key refuses the whole file as *a key I
  have not heard of* rather than answering anywhere else, so no release honours
  a different choice.

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
| `measured` | **Where and when that grade was earned**: a table of `machine` (with the memory it had, in `GB` or `GiB`), `date` (`YYYY-MM-DD`) and `runtime` (a name and a version), and optionally `drove` and `of` (the counts, both or neither), `loaded_bytes` and `on_the_gpu_bytes` (the residency, both or neither) and `instructions` (the SHA-256 of the instructions the model was shown, in sixty-four lowercase hexadecimal characters — [ADR 0034](../decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)). Written beside every grade alo OS records, never beside `"not-measured"`. Optional in a file. |
| `file` | The file on this machine the person pointed at, where they pointed at one rather than picking from what a runtime reports. Optional, and absent in every entry written before it existed. When alo OS writes it, `bytes-on-disk` beside it is what the disk said about that file at the moment they pointed — measured, never typed — and `id` is the file's own name. |

- **`drives-verbs` has no default and an entry without it does not read.** *Not
  measured* is a thing to state, not a blank to leave: an entry that said nothing
  would read as *probably fine*, and alo OS gives an agent turn only to weights
  a measurement has cleared.
- **`measured` is additive and the format number did not move for it.** A file
  written before it existed reads exactly as it did. What changed is what a
  grade does: **only a grade with a `measured` table that says something
  checkable gives the weights agent turns.** A grade without one is read, kept
  and shown — with a sentence saying it does not say which machine measured it —
  and is not a measurement, so it is not the reason somebody's files are handed
  to a model. alo OS writes a grade only from a measurement that finished, and
  always with the machine beside it; it never travels anywhere but this file.
- **`instructions` is additive too.** A grade without it reads exactly as it
  did; a grade with one that is not a digest is read, kept and shown like any
  other `measured` table that says something uncheckable, and does not give the
  weights agent turns. alo OS writes it beside every
  grade it measures from 2026-09-14, because two grades under different
  instructions are two measurements.
- **Two ids differing in case are two entries.** An id is a name a runtime
  answers to rather than a word a person chose, so it is matched exactly — which
  is the opposite answer from a provider's name and is the same answer as every
  other identity in alo OS.
- **The same id twice is refused**, naming it. alo OS says which model answered a
  question, and with two entries under one name it could not.
- **Weights with no name are refused**, because there would be nothing to ask the
  runtime for.
- **`file` is additive and the format number did not move for it.** An alo OS
  from before the key would ask the runtime for the same `id`; nothing about the
  choice changes. It is not measured again on the way in: a drive that is not
  mounted this morning is not a settings file that is wrong.
- **Pointing at a file that is not there, or at a folder, is refused at the
  write** — naming the path, and nothing is added. The catalogue is not consulted
  on the way: a file whose name matches a catalogue entry is still the person's
  own, on this list, and *the catalogue recommends; it does not gate*.

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
| `region` | Where it runs, **as stated by whoever added it**. Optional; absent is *unknown*. Never inferred from the address: `api.example.fr` is not evidence of anything, and a guess here would hand somebody a reassuring label while putting them in breach. Unknown is a value of its own and never satisfies a bound naming a region: such a bound refuses the provider with a sentence saying it has not said where it runs — never that it runs elsewhere — and a machine with no bound is unaffected. |
| `needs-a-key` | Whether it is asked for a credential. Optional, and **absent means yes**, because almost every hosted API needs one. A compatible service that takes none says `false`, and then nothing is looked up and nothing is sent. |

**There is no `key`, and that is the protection rather than an omission.** The
single most reliable way for a credential to end up in a text file in somebody's
home directory is for the file to have a field called `key`. So it has none: the
keyring name is derived as `provider/<name>`, and a file that invents a `key` is
refused whole, naming the file and never the value — the person is told, rather
than left with a credential on their disk that alo OS quietly read.

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

**A provider can be tested before it is saved, and the writer does not know
it.** `alo_asking::Vetting` makes one request for the provider's model list with
the key the person has just typed — before the key goes to a keyring and before
the address goes to this file — on the egress indicator and under the
organisation's rule, and comes back with one of exactly three things: it
answered, it refused the key, or no working provider could be reached at that
address. A provider that answered is saved through `Choosing::adding` as it
always was; one that did not has written nothing, because nothing in the
testing crate can reach this file, and is saved through the same door only if
the person says so — a provider that is down today is not a wrong provider. A
provider `alo-asking` does not know how to reach is not guessed at: the honest
answer is *this cannot be tested from here*, and the save proceeds exactly as
it does today, including this file's own judgement of the address. The test is
one request, never retried, and waits no longer than a person sits at a
dialogue.

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

## `appearance.toml` — how this person's machine looks

Kept by `alo_appearance::keeping`, in the person's folder beside this file, at
the path the crate is handed. It holds `alo_appearance::Changes` — what the
person changed about the background, the lock screen, light and dark, the size
of the text and the accent — and nothing the release ships, so a release that
improves a wallpaper reaches every machine whose owner never changed theirs.

### Keys

Besides `format`, and each of them optional: a key that is not there is a
setting the person has not changed.

| Key | Meaning |
|---|---|
| `background` | What is behind the windows on every display the person has not singled out. |
| `displays` | The displays the person singled out, oldest first, each with its own background. |
| `lock` | What the lock screen shows. |
| `following` | What decides light and dark: one of the two always, or the clock. |
| `text` | How big the text is, as a whole-number percentage. |
| `accent` | Which of the five accents the shell follows. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.
It is this file's own number: `dock.toml` moving to another says nothing about
this one.

### What alo OS writes

Every setting changed, exactly as `alo_appearance::keeping::keep` writes it:

```toml
format = 1

accent = "Moss"
displays = [["HDMI-1", { Colour = "#102A43" }], ["eDP-1", { Picture = { fitting = "Fill", of = { File = "/home/ada/harbour.jpg" } } }]]
text = 150

[background.Rotating]
fitting = "Fit"
folder = "/home/ada/Pictures"

[background.Rotating.every]
nanos = 0
secs = 600

[following.TheClock.dark_from]
hour = 18
minute = 0

[following.TheClock.light_from]
hour = 7
minute = 30

[lock.Its.Picture]
fitting = "Fill"

[lock.Its.Picture.of]
Shipped = "alo"
```

A file typed by hand need not be laid out that way — TOML's inline tables and
its `[section]` tables say the same thing — and this reads too:

```toml
format = 1
accent = "Rose"
text = 125
lock = "TheDesktop"

[following]
Always = "Dark"
```

### Values

The names inside a value are the crate's own and are matched exactly, capitals
included: `"Dark"` reads and `"dark"` does not.

- **A background** is one of three, named by what it is:
  - `{ Picture = { of = …, fitting = … } }` — `of` is `{ Shipped = "<name>" }`,
    a wallpaper that came with alo OS asked for by name and never by a path, or
    `{ File = "<whole path>" }`, the person's own picture from the top of the
    disk. A relative path is refused, because where it leads would depend on
    where the shell was started.
  - `{ Rotating = { folder = "<whole path>", every = { secs = …, nanos = … }, fitting = … } }`
    — a folder of pictures taking turns. `every` is how long each stays up, and
    anything under sixty seconds is refused: quicker than that is a flicker.
  - `{ Colour = "#RRGGBB" }` — a hash and six hexadecimal digits, as `#102A43`.
- **`fitting`** is how a picture meets the edges of a screen: `"Fill"`,
  `"Fit"`, `"Stretch"`, `"Centre"` or `"Tile"`.
- **`displays`** is a list of pairs, `["<display name>", <background>]`. The
  name is whatever the shell calls the screen, matched exactly — an empty name,
  or one that begins or ends with a space, is refused. A display that is renamed
  by a driver loses its exception and shows `background`, which is the right way
  round to fail. **A display named twice is read as its last entry.**
- **`lock`** is `"TheDesktop"` — the same background as the desktop — or
  `{ Its = <background> }`.
- **`following`** is `{ Always = "Light" }`, `{ Always = "Dark" }`, or
  `{ TheClock = { dark_from = { hour = …, minute = … }, light_from = { hour = …, minute = … } } }`.
  An hour runs from 0 to 23 and a minute from 0 to 59, on a twenty-four hour
  clock whatever the region writes; the two times must differ.
- **`text`** is from 75 to 300. 200 is what EN 301 549 requires a machine to
  reach.
- **`accent`** is `"Verdigris"`, `"Indigo"`, `"Violet"`, `"Moss"` or `"Rose"`.
  Terracotta is how the machine says alo is present or acting (ADR 0010), and a
  file that asks for it as an accent does not read.

A table inside a value has exactly its own keys: a picture, a folder, a
schedule or a time of day with a key that is not its own does not read.

### What a missing file means

**The person has changed nothing**, and the machine looks the way the release
ships it. It is not an error, and nothing is written to make one: the first byte
is written by the first change somebody makes. A person who puts everything back
is kept as `format = 1` and nothing else.

### A file that does not read

**Refused whole, and nothing in it is honoured** — not the accent on the line
above the typo. `alo_appearance::keeping::at_sign_in` answers with the release's
appearance and the refusal beside it, for Settings to say in that section. Every
sentence names the file, and each is in the vocabulary with a note for its
translator:

| What was wrong | What the person reads |
|---|---|
| The disk would not give the file up — a permission, or a folder where the file should be | `appearance.kept.not-read` |
| Not text, no `format`, or a value this shape does not take | `appearance.kept.not-understood` |
| Not TOML, from a line on | `appearance.kept.not-understood-at`, naming the line |
| Another `format` | `appearance.kept.another-format` |
| A key at the top of the file that is not on the list | `appearance.kept.unknown-key`, naming the key |

```toml refused
format = 1
accent = "Rose"
wallpaper = "harbour"
```

Refused — `appearance.kept.unknown-key`, naming `wallpaper` — and the machine
draws the release's accent rather than rose. Only a key **at the top of the
file** is named: a wrong key inside a value is a value this shape does not take.

```toml refused
format = 1

[lock.Its.Picture]
fitting = "Fill"
size = "large"

[lock.Its.Picture.of]
Shipped = "alo"
```

Refused — `appearance.kept.not-understood`.

```toml refused
format = 1
accent = "Terracotta"
```

Refused — `appearance.kept.not-understood`.

```toml refused
format = 1
text = 50
```

Refused — `appearance.kept.not-understood`.

```toml refused
format = 1
accent =
```

Refused — `appearance.kept.not-understood-at`, naming line 2.

```toml refused
format = 2
accent = "Rose"
```

Refused — `appearance.kept.another-format` — before its keys are judged, so a
file a later alo OS wrote is not reported as a typo.

A refusal quotes nothing the file says except the key at the top of it: a line
is a number, and a value is never repeated back.

### Writing it

`alo_appearance::keeping::keep` writes the whole of the person's changes: to
`appearance.toml.new` beside the file, read back off the disk as the same
changes, and only then renamed over the old one — created `0600` in a folder
created `0700` where the machine has modes. A change that would not read back
is refused before the file is touched (`appearance.kept.not-expressible`), and
a disk that will not take it leaves the file as it was
(`appearance.kept.not-written`); both say nothing has been changed.

**A file that does not read is not written over.** ADR 0038 says Settings
does not write over a file that did not read unless the person puts that
section back as shipped, so a hand edit with one typo in it is never lost to
the next click. `keep` asks the file as it is **at the moment of the write** —
after the new text is staged and read back, immediately before the rename —
and when it is there and does not read, for any of the reasons in the table
above, the change is refused (`appearance.kept.not-replaced`): the file is left
byte for byte as it was, the sentence names it and tells the person to correct
it or put appearance back as shipped, and
`alo_appearance::FileNotWritten::did_not_read` says what is wrong with it. It is
never asked of what was read at sign-in: a file mended in an editor since then
takes the next change. A file that is not there, or that reads, is written as
above.

**Putting appearance back as shipped is the one door that replaces such a
file.** `alo_appearance::keeping::put_back_as_shipped` takes no changes and
writes `format = 1` alone, whatever is at the path — the same whole write, read
back before it counts — so afterwards the file reads as a person who has
changed nothing. It is the person's deliberate act in Settings; nothing else,
and nothing an agent can reach, writes over a file that did not read.

## `dock.toml` — which edge the dock is on

Kept by `alo_dock::keeping`, beside `appearance.toml`, at the path the crate is
handed. It holds `alo_dock::Changes`.

### Keys

Besides `format`, and optional:

| Key | Meaning |
|---|---|
| `edge` | Which edge of the screen the dock sits on: `"Bottom"`, `"Left"`, `"Right"` or `"Top"`. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.

### What alo OS writes

```toml
format = 1

edge = "Left"
```

### What a missing file means

**The person has changed nothing**: the dock is where the release puts it. Not
an error, and nothing is written until the person moves it.

### A file that does not read

**Refused whole**, and the dock is where the release puts it;
`alo_dock::keeping::at_sign_in` answers with the release's dock and the refusal
beside it. The sentences name the file, and are the same five reasons
`appearance.toml` has, under `dock.kept.`: `not-read`, `not-understood`,
`not-understood-at` naming the line, `another-format`, and `unknown-key` naming
the key.

```toml refused
format = 1
edge = "Left"
size = 48
```

Refused — `dock.kept.unknown-key`, naming `size` — and the dock is not moved to
the left.

```toml refused
format = 1
edge = "left"
```

Refused — `dock.kept.not-understood`. The edge's name is matched exactly.

### Writing it

As `appearance.toml`: whole, read back before it counts, and renamed over the
old file (`dock.kept.not-expressible`, `dock.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`dock.kept.not-replaced`) with the file byte for
byte as it was. `alo_dock::keeping::put_back_as_shipped` is the one door that
replaces such a file, and it writes `format = 1` alone.

## `shortcuts.toml` — the shortcuts this person changed

Kept by `alo_shortcuts::keeping`, beside the other two, at the path the crate is
handed. It holds `alo_shortcuts::Changes`: one entry per action the person
moved or cleared, and nothing for an action still on the release's shortcut.

### Keys

Besides `format`, and optional:

| Key | Meaning |
|---|---|
| `changed` | One `[[changed]]` table per action the person changed, in the order they changed them. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.

### What alo OS writes

A person who moved the agent, cleared the launcher and moved the next window:

```toml
format = 1

[[changed]]
action = "TheAgent"

[changed.chord]
key = "Space"
modifiers = ["Ctrl", "Alt"]

[[changed]]
action = "Launcher"

[[changed]]
action = "NextWindow"

[changed.chord]
key = "J"
modifiers = ["Super"]
```

### Values

Each `[[changed]]` has exactly two keys:

- **`action`** — `"TheAgent"`, `"Launcher"`, `"CloseWindow"`,
  `"MinimiseWindow"`, `"MaximiseWindow"`, `"SnapLeft"`, `"SnapRight"`,
  `"NextWindow"`, `"PreviousWindow"`, `"NextApplication"` or
  `"PreviousApplication"`. **An action named twice is read as its last entry.**
- **`chord`** — `{ modifiers = [...], key = "..." }`, or no `chord` at all,
  which is the person wanting **no** shortcut for that action rather than the
  release's.
  - `modifiers` is a list of `"Super"`, `"Ctrl"`, `"Alt"` and `"Shift"`, in any
    order; the same one twice is the same chord.
  - `key` is `"A"` to `"Z"`, `"Digit0"` to `"Digit9"`, `"F1"` to `"F12"`,
    `"Comma"`, `"Period"`, `"Slash"`, `"Minus"`, `"Equals"`, `"Space"`,
    `"Tab"`, `"Enter"`, `"Escape"`, `"Backspace"`, `"Delete"`, `"Insert"`,
    `"Home"`, `"End"`, `"PageUp"`, `"PageDown"`, `"Left"`, `"Right"`, `"Up"`,
    `"Down"` or `"Print"`.
  - A chord with **nothing held**, with **only Shift** held, or that is
    **Ctrl+C, Ctrl+X or Ctrl+V** — which the clipboard is worked by — does not
    read.

**A clash does not refuse the file.** A chord the person gave to an action that
the release, or another of their own entries, also has is read, and is reported
by `alo_shortcuts::Shortcuts::clashes` where the person can see it: their
binding beats one the release shipped, and two of their own on one chord fire
nothing.

### What a missing file means

**The person has changed nothing**: every shortcut is the release's. Not an
error, and nothing is written until they change one.

### A file that does not read

**Refused whole**, and every shortcut is the one the release ships;
`alo_shortcuts::keeping::at_sign_in` answers with the release's shortcuts and
the refusal beside it. The sentences name the file, and are the same five
reasons, under `shortcuts.kept.`: `not-read`, `not-understood`,
`not-understood-at` naming the line, `another-format`, and `unknown-key` naming
the key.

```toml refused
format = 1
bindings = []
```

Refused — `shortcuts.kept.unknown-key`, naming `bindings`.

```toml refused
format = 1

[[changed]]
action = "CloseWindow"

[changed.chord]
key = "C"
modifiers = ["Ctrl"]
```

Refused — `shortcuts.kept.not-understood` — so Ctrl+C is not taken from the
clipboard, and nothing else in the file is honoured either.

```toml refused
format = 1

[[changed]]
action = "Launch"
```

Refused — `shortcuts.kept.not-understood`. An action's name is matched exactly.

### Writing it

As `appearance.toml`: whole, read back before it counts, and renamed over the
old file (`shortcuts.kept.not-expressible`, `shortcuts.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`shortcuts.kept.not-replaced`) with the file byte
for byte as it was — so a person's hand-moved shortcut with a typo in it is not
lost to the next shortcut they change in Settings.
`alo_shortcuts::keeping::put_back_as_shipped` is the one door that replaces
such a file, and it writes `format = 1` alone.

## `what-opens-what.toml` — which application opens each kind of file

Kept by `alo_applications::keeping`, beside the other three, at the path the
crate is handed. It holds `alo_applications::Chosen`: the application the person
chose for each kind of file they chose one for, and nothing for a kind they did
not. `docs/features.md`, v0.5: *file associations — what opens what, changeable
by a person.*

A kind with no entry opens in the first installed application that declares it
in its desktop entry; an entry wins over every declaration. What answers *what
opens this file* is `alo_applications::WhatOpensWhat`, and it says which of the
two it was. **Only a person's choice is written here**: no application declares
itself into this file, no agent verb names it, and the open-with portal
(`alo_portals::open_with`) reads it and never writes it.

### Keys

Besides `format`, and optional:

| Key | Meaning |
|---|---|
| `kinds` | A table from a kind of file to the identifier of the application chosen to open it. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.

### What alo OS writes

A person who chose Okular for PDF documents and Papers for PNG images:

```toml
format = 1

[kinds]
pdf = "org.kde.okular"
png-image = "org.gnome.Papers"
```

### Values

Each key of `[kinds]` is a kind of file, as `alo_applications::spelled` writes
it — the kind a file is read as from its own bytes, never its extension:
`pdf`, `word-document`, `excel-workbook`, `powerpoint-presentation`,
`opendocument-text`, `opendocument-spreadsheet`, `opendocument-presentation`,
`older-word-document`, `older-excel-workbook`, `older-powerpoint-presentation`,
`rich-text`, `text`, `text-in-an-older-character-set`, `png-image`,
`jpeg-image`, `gif-image`, `webp-image`, `pages-document`, `heic-photo`, `autocad-drawing`,
`zip-archive`,
`matroska-video`,
`webm-video`, `mp4-video`, `mp4-audio`, `avi-video`, `ogg-media`, `mp3-audio`,
`wave-audio` or `flac-audio`. A kind is matched exactly. A program, an empty file and bytes of no kind alo OS recognises are
not kinds, and nothing is ever chosen to open them.

Each value is an application's identifier, as this machine knows it — never the
name it calls itself, because two applications can share a name and no two
share an identifier. An identifier has no spaces, no control characters and no
folder separators in it. **An application chosen and since uninstalled is kept**
and used again if it is reinstalled; until then the kind opens in what the
applications declare, and the answer says that the choice is not installed.

### What a missing file means

**The person has chosen nothing**: every kind opens in the first installed
application that declares it, and a kind no application declares is opened by
nothing — said as a sentence (`applications.opens.nothing`), never handed to a
text editor. Not an error, and nothing is written until the person chooses.

### A file that does not read

**Refused whole**, and every kind opens in what the applications declare;
`alo_applications::keeping::at_sign_in` answers with no choices and the refusal
beside it. The sentences name the file, and are the same five reasons, under
`applications.kept.`: `not-read`, `not-understood`, `not-understood-at` naming
the line, `another-format`, and `unknown-key` naming the key.

```toml refused
format = 1
default = "org.gnome.TextEditor"

[kinds]
pdf = "org.kde.okular"
```

Refused — `applications.kept.unknown-key`, naming `default` — so there is no
application that opens everything, and Okular is not used for PDFs either.

```toml refused
format = 1

[kinds]
docx = "org.libreoffice.LibreOffice.writer"
```

Refused — `applications.kept.not-understood`. A kind is named as alo OS reads
it, and an extension is not a kind.

```toml refused
format = 1

[kinds]
pdf = "/usr/bin/okular"
```

Refused — `applications.kept.not-understood`. An application is named by its
identifier, never by a program's path.

### Writing it

As `appearance.toml`: whole, read back before it counts, and renamed over the
old file (`applications.kept.not-expressible`, `applications.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`applications.kept.not-replaced`) with the file
byte for byte as it was.
`alo_applications::keeping::put_back_as_shipped` is the one door that replaces
such a file, and it writes `format = 1` alone.

## `sleeping.toml` — what closing the lid does, and what keeps this machine awake

Kept by `alo_sleeping::keeping`, beside `appearance.toml`, at the path the
crate is handed. It holds `alo_sleeping::Changes`: what the person changed
about sleep, and nothing the release ships.

### Keys

Besides `format`, and each of them optional: a key that is not there is a
setting the person has not changed.

| Key | Meaning |
|---|---|
| `keep-awake` | Whether the machine stays awake when nobody is using it. |
| `lid` | What closing the lid does. |

How long *nobody is using it* is, battery thresholds and power profiles are not
in this file and are not that crate's.

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.
It is this file's own number: `appearance.toml` moving to another says nothing
about this one.

### What alo OS writes

Both settings changed, exactly as `alo_sleeping::keeping::keep` writes it:

```toml
format = 1

keep-awake = true
lid = "stays-awake-with-a-display"
```

### Values

The names a value takes are matched exactly, case included: `"sleeps"` reads
and `"Sleeps"` does not.

- **`keep-awake`** is `true` or `false`. What alo OS ships is `false`: nothing
  keeps the machine awake on its own.
- **`lid`** is `"sleeps"` — closing the lid always sleeps the machine, which is
  what alo OS ships — or `"stays-awake-with-a-display"`, which sleeps the
  machine unless another display is attached at the moment the lid closes.

### What a missing file means

**The person has changed nothing**: closing the lid sleeps the machine, and
nothing keeps it awake. Not an error, and nothing is written until the person
changes one of the two.

### A file that does not read

**Refused whole, and nothing in it is honoured** — not the lid on the line
above the typo. `alo_sleeping::keeping::at_sign_in` answers with the release's
sleep settings and the refusal beside it, for Settings to say in that section.
Every sentence names the file, and each is in the vocabulary with a note for
its translator:

| What was wrong | What the person reads |
|---|---|
| The disk would not give the file up — a permission, or a folder where the file should be | `sleeping.kept.not-read` |
| Not text, no `format`, or a value this shape does not take | `sleeping.kept.not-understood` |
| Not TOML, from a line on | `sleeping.kept.not-understood-at`, naming the line |
| Another `format` | `sleeping.kept.another-format` |
| A key at the top of the file that is not on the list | `sleeping.kept.unknown-key`, naming the key |

```toml refused
format = 1
lid = "stays-awake-with-a-display"
never-sleep = true
```

Refused — `sleeping.kept.unknown-key`, naming `never-sleep` — and closing the
lid sleeps the machine, because the good key beside it is not honoured either.

```toml refused
format = 1
lid = "Sleeps"
```

Refused — `sleeping.kept.not-understood`. The name is matched exactly.

```toml refused
format = 1
keep-awake = "yes"
```

Refused — `sleeping.kept.not-understood`. It is a `true` or a `false`, not a
word for one.

```toml refused
format = 1
lid =
```

Refused — `sleeping.kept.not-understood-at`, naming line 2.

```toml refused
format = 2
lid = "sleeps"
```

Refused — `sleeping.kept.another-format` — before its keys are judged, so a
file a later alo OS wrote is not reported as a typo.

### Writing it

As `appearance.toml`: the whole of the person's changes to `sleeping.toml.new`
beside the file, read back off the disk as the same changes, and only then
renamed over the old one. A change that would not read back is refused before
the file is touched (`sleeping.kept.not-expressible`), and a disk that will not
take it leaves the file as it was (`sleeping.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`sleeping.kept.not-replaced`) with the file byte
for byte as it was, `alo_sleeping::FileNotWritten::did_not_read` saying what is
wrong with it. `alo_sleeping::keeping::put_back_as_shipped` is the one door
that replaces such a file, and it writes `format = 1` alone.

## `displays.toml` — where this person's screens are, and when they warm

Kept by `alo_displays::keeping`, beside `appearance.toml`, at the path the
crate is handed. It holds `alo_displays::Changes`: every arrangement the person
has made, one per set of screens, and night light.

alo OS ships **no** arrangement — it has never seen anybody's screens — so here
the person's changes are the whole of it rather than a difference over a
default.

### Keys

Besides `format`, and each of them optional:

| Key | Meaning |
|---|---|
| `arrangements` | One `[[arrangements]]` table per set of screens, in the order they were last remembered in, oldest first. |
| `night-light` | How warm the screens are drawn of an evening, and when. One for the machine, not one per set of screens. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.
It is this file's own number: `appearance.toml` moving to another says nothing
about this one.

### What alo OS writes

Two sets of screens arranged and night light asked for, exactly as
`alo_displays::keeping::keep` writes it:

```toml
format = 1

[[arrangements]]

[[arrangements.screens]]
at = [0, 0]
main = true
scale = 200
socket = "eDP-1"

[[arrangements.screens]]
at = [-2560, 0]
make = "Example"
model = "U2723QE"
scale = 150
serial = "G4H2K7"

[[arrangements]]

[[arrangements.screens]]
at = [1920, 240]
scale = 200
socket = "eDP-1"

[[arrangements.screens]]
at = [0, 0]
main = true
make = "Example"
model = "P2419H"
scale = 100

[night-light]
warmth = 3400

[night-light.between]
from = "21:30"
to = "07:00"
```

A file typed by hand need not be laid out that way — TOML's inline tables say
the same thing — and night light taken from the sun rather than from a clock
reads too:

```toml
format = 1

[night-light]
warmth = 2700

[night-light.sunset-at]
latitude = 51.5
longitude = -0.12
```

### Values

- **An arrangement** is one `[[arrangements]]` table holding one
  `[[arrangements.screens]]` row per screen of that set. It is remembered under
  the set of screens it names, which is the whole of why docking at the office
  restores the office: the office's screens are a different set from home's,
  and nothing asked where the machine is. **A set written down twice means what
  the file says last**, rather than one of the two at random.
- **A screen row names its screen in one of two ways, and never both.**
  - `make` and `model`, with `serial` where the screen reports one — what the
    screen says about itself, which is what survives being unplugged, carried
    to another desk and plugged into a different socket.
  - `socket` — the name the compositor calls the socket, for a screen that says
    nothing about itself, which a great many built-in laptop panels do not. It
    is **weaker**, and the machine says so to the person: another screen in the
    same socket is set up as though it were this one.

  A row with neither, with both, or with a `make` and no `model`, is half a
  screen and does not read. Every name is matched exactly; an empty one, or one
  that begins or ends with a space, is refused.
- **`at`** is the top left corner of that screen, `[across, down]`, two whole
  numbers in the pixels the arrangement is laid out in. A screen to the left of
  the main one has a negative first number, and one above it a negative second.
- **`scale`** is how large everything on that screen is drawn, in per cent,
  from 100 to 300. No size is written down beside it: how much room a screen
  takes is read off the pixels it reports at the moment it is plugged in, and a
  screen takes *less* room the larger it draws.
- **`main`** is `true` on the one screen a new window opens on, and left out of
  every other row. An arrangement with none, or with two, does not read — one
  screen or a shrug, and a shrug is not an answer to *where does this window
  go*.
- **`night-light`** is a warmth, and at most one way of saying when.
  - `warmth` is the colour temperature the screens are drawn at while night
    light is on, in kelvin, from 2000 to 6500. It is **required**: a
    `night-light` table without one does not read. 6500 is the white a screen
    is defined against, and warms nothing.
  - `between = { from = "21:30", to = "07:00" }` — every day, from one time to
    another on a twenty-four hour clock, written `HH:MM` whatever the region
    writes. A stretch that runs through midnight is written exactly that way,
    and the two times must differ.
  - `sunset-at = { latitude = 51.5, longitude = -0.12 }` — from sunset to
    sunrise where the person says they are, in degrees: north and east
    positive, south and west negative, a latitude no further than 90 and a
    longitude no further than 180. **Typed, never looked up**: nothing in alo
    OS arrives at a whereabouts any other way.
  - Neither of the two means night light is set and never on.
  - **Both does not read.** A file that asks for a clock and for the sun is a
    file that has not said when.

  A table inside `night-light` has exactly its own keys: one it does not know
  refuses the whole file.

An `[[arrangements]]` table, and each `[[arrangements.screens]]` row inside it,
has exactly its own keys: one alo OS does not know refuses the whole file.
There is no key for how bright a screen is, what a person calls the desk it
stands on, which resolution it was running at or which of two identical
monitors this one is, and a table that has one does not read rather than being
read past. A row is where a screen is described, which is exactly the place
something would be tempted to grow a field, and a row read past is room to put
one nobody agreed to.

### What a missing file means

**The person has arranged nothing and has never asked for night light**: every
set of screens is laid out as though the machine had never seen it, and nothing
is warmed. Not an error, and nothing is written until somebody moves a screen.

### A file that does not read

**Refused whole, and nothing in it is honoured** — not the arrangement above
the one with two main screens in it. `alo_displays::keeping::at_sign_in`
answers with a machine that has arranged nothing and the refusal beside it, for
Settings to say in that section. Every sentence names the file, and each is in
the vocabulary with a note for its translator:

| What was wrong | What the person reads |
|---|---|
| The disk would not give the file up — a permission, or a folder where the file should be | `displays.kept.not-read` |
| Not text, no `format`, or a value this shape does not take | `displays.kept.not-understood` |
| Not TOML, from a line on | `displays.kept.not-understood-at`, naming the line |
| Another `format` | `displays.kept.another-format` |
| A key at the top of the file that is not on the list | `displays.kept.unknown-key`, naming the key |

```toml refused
format = 1
brightness = 50
```

Refused — `displays.kept.unknown-key`, naming `brightness`. How bright a screen
is, is not something this file holds.

```toml refused
format = 1

[[arrangements]]

[[arrangements.screens]]
socket = "eDP-1"
at = [0, 0]
scale = 100
main = true

[[arrangements.screens]]
socket = "DP-2"
at = [1920, 0]
scale = 100
main = true
```

Refused — `displays.kept.not-understood`. Two main screens is not an
arrangement, and neither is none.

```toml refused
format = 1

[[arrangements]]

[[arrangements.screens]]
make = "Example"
at = [0, 0]
scale = 100
main = true
```

Refused — `displays.kept.not-understood`. A `make` with no `model` is half a
screen, and it would match a different monitor tomorrow.

```toml refused
format = 1

[[arrangements]]
desk = "the office"

[[arrangements.screens]]
socket = "eDP-1"
at = [0, 0]
scale = 100
main = true
```

Refused — `displays.kept.not-understood`. An arrangement is its screens and
nothing else; where the machine happens to be standing is not a key this file
has, and nothing asks it.

```toml refused
format = 1

[[arrangements]]

[[arrangements.screens]]
socket = "eDP-1"
at = [0, 0]
scale = 100
main = true
brightness = 50
```

Refused — `displays.kept.not-understood`. How bright a screen is, is not
something this file holds — inside a screen's row no more than at the top of
the file.

```toml refused
format = 1

[night-light]
warmth = 3400

[night-light.between]
from = "21:30"
to = "07:00"

[night-light.sunset-at]
latitude = 51.5
longitude = -0.12
```

Refused — `displays.kept.not-understood`. A clock and the sun are two answers
to one question.

```toml refused
format = 1

[night-light]
warmth =
```

Refused — `displays.kept.not-understood-at`, naming line 4.

```toml refused
format = 2

[night-light]
warmth = 3400
```

Refused — `displays.kept.another-format` — before its keys are judged, so a
file a later alo OS wrote is not reported as a typo.

### Writing it

As `appearance.toml`: the whole of the person's arrangements to
`displays.toml.new` beside the file, read back off the disk as the same
arrangements, and only then renamed over the old one
(`displays.kept.not-expressible`, `displays.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`displays.kept.not-replaced`) with the file byte
for byte as it was. `alo_displays::keeping::put_back_as_shipped` is the one
door that replaces such a file, and it writes `format = 1` alone.

## `leaving.toml` — whether what was open opens again

Kept by `alo_leaving::keeping`, beside `appearance.toml`, at the path the crate
is handed. It holds `alo_leaving::Changes`: one setting, and the list that
exists only because the person turned that setting on.

### Keys

Besides `format`, and both of them optional:

| Key | Meaning |
|---|---|
| `reopen` | Whether this person's applications open again when they sign in. |
| `was-open` | One `[[was-open]]` table per window that was open when they last logged out, oldest first. |

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.
It is this file's own number: `appearance.toml` moving to another says nothing
about this one.

### What alo OS writes

The setting on and three windows written down at a log-out, exactly as
`alo_leaving::keeping::at_sign_out` writes it:

```toml
format = 1

reopen = true

[[was-open]]
application = "org.example.Files"
on = "eDP-1"
split = "the-whole-screen"

[[was-open]]
application = "org.example.Mail"
on = "DP-2"
split = "left-half"

[[was-open]]
application = "org.example.Editor"
on = "DP-2"
split = "right-half"
```

### Values

- **`reopen`** is `true` or `false`. What alo OS ships is `false`, and that is
  a decision rather than a default nobody thought about: the list below is
  written only for somebody who asked for it, so a machine nobody configured
  keeps no record of what anybody had open.
- **`was-open`** is one `[[was-open]]` table per window, in the order the
  windows were opened. Two windows of one application are two tables, because
  *reopen what was open* is a promise about what was on which screen. Each
  table has three keys and no fourth:
  - `application` — the identifier this machine knows the application by, as
    `org.example.Editor`, which is what a grant is made over. Never the name a
    packager wrote: two of those can claim the same one, and a file that named
    *Mail* would reopen whichever *Mail* answered to it next. No spaces, no
    control characters, no `/` and no `\`.
  - `on` — the name the shell knows that screen by, matched exactly. An empty
    name, or one that begins or ends with a space, is refused.
  - `split` — what the window had of that screen: `"the-whole-screen"`,
    `"left-half"`, `"right-half"`, `"top-half"`, `"bottom-half"`,
    `"top-left-quarter"`, `"top-right-quarter"`, `"bottom-left-quarter"`,
    `"bottom-right-quarter"` or `"part"`.
- **A fourth key refuses the whole file**, and that is the clause to read this
  section for. There is no key for a title, a document, a path, a URL, a window
  identifier or a size, and a `[[was-open]]` table that has one does not read
  rather than being read past. A window title is the name of the document
  somebody was writing or the subject of the mail they were reading; written
  into a file it is a record of what a person did yesterday, sitting in their
  own folder, and on a machine with an agent on it it is context nobody offered
  ([ADR 0001](../decisions/0001-the-capability-model.md)).
- **The list is written at one moment, and that moment is a log-out.**
  `alo_leaving::keeping::at_sign_out` is the only door in that crate that
  writes one, and it can only be called by a log-out that asked every
  application to close first. A machine that kept the file up to date as
  windows opened and closed would be a background reader of what somebody is
  doing. The cost is honest and worth stating: a session that ends without a
  log-out — a power cut, a crash — leaves nothing to reopen.
- **And only for somebody who asked.** The choice is read from the file at the
  moment of the write: with `reopen` absent or `false`, no list is written and
  any list that was there is taken out. A hand-written list under a `reopen`
  that is off reads, and nothing is reopened from it —
  `alo_leaving::restoring::at_sign_in` reopens a list only when the choice is
  on, and the next log-out removes it.

### What a missing file means

**The person has changed nothing and nothing was written down**: nothing is
reopened at the next sign-in, and the machine holds no record of what anybody
had open. Not an error. A log-out on a machine whose owner never asked for the
setting writes no file at all.

### A file that does not read

**Refused whole, and nothing in it is honoured** — including a list that is
well formed inside a file that is not. `alo_leaving::keeping::at_sign_in`
answers with the release's settings and the refusal beside it, and
`alo_leaving::restoring::at_sign_in` reopens nothing. Every sentence names the
file, and each is in the vocabulary with a note for its translator:

| What was wrong | What the person reads |
|---|---|
| The disk would not give the file up — a permission, or a folder where the file should be | `leaving.kept.not-read` |
| Not text, no `format`, or a value this shape does not take | `leaving.kept.not-understood` |
| Not TOML, from a line on | `leaving.kept.not-understood-at`, naming the line |
| Another `format` | `leaving.kept.another-format` |
| A key at the top of the file that is not on the list | `leaving.kept.unknown-key`, naming the key |

```toml refused
format = 1
restore = true
```

Refused — `leaving.kept.unknown-key`, naming `restore`. The key is `reopen`.

```toml refused
format = 1
reopen = true

[[was-open]]
application = "org.example.Editor"
on = "eDP-1"
split = "the-whole-screen"
title = "march.odt"
```

Refused — `leaving.kept.not-understood`, and nothing is reopened. There is no
key for a title and there will not be one.

```toml refused
format = 1
reopen = true

[[was-open]]
application = "org.example.Editor"
on = "eDP-1"
split = "diagonal"
```

Refused — `leaving.kept.not-understood`. The splits are the ten named above.

```toml refused
format = 1
reopen = true

[[was-open]]
application = "org.example.Editor"
on = ""
split = "the-whole-screen"
```

Refused — `leaving.kept.not-understood`. A screen with no name is not a screen
anything could be reopened on.

```toml refused
format = 1
reopen =
```

Refused — `leaving.kept.not-understood-at`, naming line 2.

```toml refused
format = 2
reopen = true
```

Refused — `leaving.kept.another-format` — before its keys are judged, so a file
a later alo OS wrote is not reported as a typo.

### Writing it

As `appearance.toml`: whole, read back before it counts, and renamed over the
old file (`leaving.kept.not-expressible`, `leaving.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change — or a log-out — over
one that is there and does not read is refused (`leaving.kept.not-replaced`)
with the file byte for byte as it was.
`alo_leaving::keeping::put_back_as_shipped` is the one door that replaces such
a file, and it writes `format = 1` alone: nothing reopened, and no list kept.

## `notifying.toml` — when notifications are held

Kept by `alo_notifying::keeping`, beside `appearance.toml`, at the path the
crate is handed. It holds `alo_notifying::Changes`: what the person changed
about notifications, and nothing the release ships.

### Keys

Besides `format`, and both of them optional:

| Key | Meaning |
|---|---|
| `do-not-disturb` | Whether notifications are held rather than shown. |
| `quiet-hours` | The stretch of the clock during which they are held anyway. |

**No notification is in this file, and none ever will be.** There is no key for
a title, a body, a sender or what somebody missed. Those live in the session's
own memory (`alo_notifying::Missed`), reach no disk, and are gone when the
person signs out.

### `format`

`format = 1`, the first line of the file, and the only shape this alo OS reads.
It is this file's own number: `appearance.toml` moving to another says nothing
about this one.

### What alo OS writes

Both settings changed, exactly as `alo_notifying::keeping::keep` writes it:

```toml
format = 1

do-not-disturb = true

[quiet-hours.begin]
hour = 22
minute = 30

[quiet-hours.end]
hour = 7
minute = 0
```

A file typed by hand need not be laid out that way, and this reads too:

```toml
format = 1
do-not-disturb = false
quiet-hours = { begin = { hour = 23, minute = 0 }, end = { hour = 6, minute = 30 } }
```

### Values

- **`do-not-disturb`** is `true` or `false`. What alo OS ships is `false`:
  notifications are shown. A machine that arrives silent is a machine whose
  owner finds out a week later that nothing was ever reaching them.
- **`quiet-hours`** is `begin` and `end`, each a time of day written
  `{ hour = …, minute = … }` on a twenty-four hour clock whatever the region
  writes — an hour from 0 to 23 and a minute from 0 to 59. The two must differ,
  because one time given twice says nothing about any minute of the day. A
  stretch that runs through midnight is written exactly that way: `begin` in
  the evening and `end` in the morning. The minute it begins is inside it and
  the minute it ends is outside, so two stretches laid end to end never both
  hold the same minute. Each table has exactly its own keys.
- **What happens while the screen is being shared or recorded is not a
  setting.** Notifications are held then whatever this file says, decided
  before the file is consulted (`alo_notifying::Quiet::now`), and there is
  nothing here to turn it off with.

### What a missing file means

**The person has changed nothing**: notifications are shown, and no hours are
set aside. Not an error, and nothing is written until the person changes one of
the two.

### A file that does not read

**Refused whole, and nothing in it is honoured** — not the quiet hours on the
line above the typo. `alo_notifying::keeping::at_sign_in` answers with the
release's settings and the refusal beside it, for Settings to say in that
section. Every sentence names the file, and each is in the vocabulary with a
note for its translator:

| What was wrong | What the person reads |
|---|---|
| The disk would not give the file up — a permission, or a folder where the file should be | `notifying.kept.not-read` |
| Not text, no `format`, or a value this shape does not take | `notifying.kept.not-understood` |
| Not TOML, from a line on | `notifying.kept.not-understood-at`, naming the line |
| Another `format` | `notifying.kept.another-format` |
| A key at the top of the file that is not on the list | `notifying.kept.unknown-key`, naming the key |

```toml refused
format = 1
do-not-disturb = true
missed = ["a message from anna"]
```

Refused — `notifying.kept.unknown-key`, naming `missed`. What somebody missed
is not kept in a file, so there is no key for it to arrive by.

```toml refused
format = 1

[quiet-hours]
begin = { hour = 22, minute = 30 }
end = { hour = 7, minute = 0 }
until = "monday"
```

Refused — `notifying.kept.not-understood`. A table inside a value has exactly
its own keys.

```toml refused
format = 1

[quiet-hours.begin]
hour = 22
minute = 0

[quiet-hours.end]
hour = 22
minute = 0
```

Refused — `notifying.kept.not-understood`. One time given twice is not a
stretch of the clock.

```toml refused
format = 1

[quiet-hours.begin]
hour = 24
minute = 0

[quiet-hours.end]
hour = 7
minute = 0
```

Refused — `notifying.kept.not-understood`. The hours run from 0 to 23.

```toml refused
format = 1
do-not-disturb =
```

Refused — `notifying.kept.not-understood-at`, naming line 2.

```toml refused
format = 2
do-not-disturb = true
```

Refused — `notifying.kept.another-format` — before its keys are judged, so a
file a later alo OS wrote is not reported as a typo.

### Writing it

As `appearance.toml`: whole, read back before it counts, and renamed over the
old file (`notifying.kept.not-expressible`, `notifying.kept.not-written`).

**A file that does not read is not written over.** As `appearance.toml`: the
file is asked at the moment of the write, and a change over one that is there
and does not read is refused (`notifying.kept.not-replaced`) with the file byte
for byte as it was. `alo_notifying::keeping::put_back_as_shipped` is the one
door that replaces such a file, and it writes `format = 1` alone.

## A Settings surface, from sign-in to the next change

For the shell: the calls a Settings surface makes for each of the three
sections kept beside `settings.toml`, in the order a session makes them. None
of them is new — this is the road the sections above describe, walked once,
all three files at a time, by
`crates/alo-choosing/tests/one_persons_folder_from_sign_in_to_the_next_change.rs`,
which holds this section to exactly the calls it makes.

**Once, at sign-in**, the session asks `alo_choosing::the_persons_folder`,
handing it `$XDG_CONFIG_HOME` and `$HOME` as the session has them. It holds the
result and handles both cases before handing any path to a keeper.
On refusal, `alo_choosing::NoFolder::said` supplies the vocabulary sentence
`choosing.session.no-folder`: “this session has no home directory; changes take
effect now, but nothing you change now will be kept once you sign out”. Show
this before a change. Every section starts as the release ships it; changes are
drawn for this sign-in only and nothing is written. No keeper is called and no
temporary or relative fallback folder is invented.
On success, `alo_choosing::PersonsFolder::path_of` joins the folder with the file
name its own crate declares, and that path is all the crate is handed.

| Section | Its path, inside the folder | Drawn at sign-in | A change | Put back as shipped |
|---|---|---|---|---|
| Appearance | `alo_appearance::keeping::THE_FILE` | `alo_appearance::keeping::at_sign_in` | `alo_appearance::keeping::keep` | `alo_appearance::keeping::put_back_as_shipped` |
| Dock | `alo_dock::keeping::THE_FILE` | `alo_dock::keeping::at_sign_in` | `alo_dock::keeping::keep` | `alo_dock::keeping::put_back_as_shipped` |
| Shortcuts | `alo_shortcuts::keeping::THE_FILE` | `alo_shortcuts::keeping::at_sign_in` | `alo_shortcuts::keeping::keep` | `alo_shortcuts::keeping::put_back_as_shipped` |

- **Drawn at sign-in** answers what the section draws and, when its file did
  not read, the refusal beside it. That section is drawn as the release ships
  it and says the refusal's `said` in the section — naming the file, and the
  key when a key was what was wrong. One file that did not read says nothing
  about the other two, which are drawn as the person left them.
- **A change** is the drawn value changed and its `changes()` handed whole to
  the section's change call — never a fragment of the file. When it is
  refused, the section says the refusal's `said`, and when `did_not_read`
  answers, what is wrong with the file as well; the file is byte for byte as it
  was, and the section goes on drawing what it drew before the change, because
  that is what the file still says. A change in one section is written whether
  or not another section's file reads.
- **Put back as shipped** is what a section whose file did not read offers,
  and is the one call that replaces such a file. Afterwards the section is
  drawn as the release ships it and the next change is written as any other.
- **Nothing watches the folder.** A change made in Settings is drawn at once,
  because Settings and the compositor are one process; a file edited by hand is
  read at the next sign-in, by the same call.
