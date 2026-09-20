# Contract — the portal answers file

**Status:** v0.5, contract. Additive changes only; a break requires versioning
and a deprecation period. See `CLAUDE.md`, "Contracts outlive code".
**Owner:** `crates/alo-portals` (`answers_file`, `answers_head`, `kept_answer`,
`kept_answer_said`, `kept_outcome`, `kept_outcome_said`, `read_back_said`,
`shortening`).
**Decisions:** [ADR 0001](../decisions/0001-the-capability-model.md) §7 (every
execution and every refusal leaves a record),
[ADR 0040](../decisions/0040-what-an-applications-grant-is-over.md) part 2 (no
refusal calls an application an agent). **Held by:**
`crates/alo-portals/tests/what_an_application_was_answered_is_kept.rs`, which
fails when a name the file writes is missing here, and
`crates/alo-portals/tests/what_applications_were_answered_is_kept_as_long_as_the_record.rs`,
which holds *Shortening it*, and
`crates/alo-portals/tests/what_applications_were_answered_reads_back_in_the_persons_language.rs`,
which holds *Reading it to a person*.

This is the file the portal backend (`docs/contracts/portals.md`) writes every
answer it gives an application into — what was handed over and, as carefully,
what was refused — so that a person can read later what their applications
asked for and were told, after the backend that answered has stopped.

It is **not** the agent's record (`docs/contracts/record-file.md`). That file is
*what an agent did*, and every entry in it names an agent or nobody. An
application is not an agent, so its requests are kept here, beside it, in the
same notation.

## Whose it is

**The person's.** One file per login, and
[ADR 0052](../decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md)
says why: a list of every time somebody's calendar asked for their microphone
and what they said is a description of their working day, not a record of what
was executed on the machine. An organisation that manages this machine exports
the agent's record (ADR 0004); it does not export this.

A machine with two people on it has **two of these files**, one in each person's
own state, and neither can read the other's — held by the filesystem rather than
by a field in a line.

## Where it is

`$XDG_STATE_HOME/alo/portal-answers.jsonl` when that variable is set and
absolute; otherwise `$HOME/.local/state/alo/portal-answers.jsonl`, which is the
base directory specification's own default. A relative variable is ignored, as
the specification says. A login with neither variable has **nowhere** for this
file, and the backend refuses rather than guessing at one.

Before ADR 0052 it was `/var/lib/alo/portal-answers.jsonl`, beside the agent's
record. **A machine upgraded from a pre-release image still has that file and
nothing reads it**: it may hold two people's answers, there is no honest way to
split one, and deleting somebody's record is a thing a person does rather than a
thing an upgrade does.

It is held to the rules `alo-remembering` holds the grants and pairings files
to. Before a byte is read or added:

- the path is **not a symbolic link**, and is never followed;
- it is **a regular file**, never a pipe, a device or a folder;
- it belongs to **the login reading it**, and **nobody else can write it**
  (group- or world-writable is refused). A file owned by root in a person's own
  state directory is refused too: there, root's ownership is evidence that
  something else wrote this person's record;
- a file the backend makes is made **`0600`**; the **`alo` folder** is made
  `0700` inside a state directory that already exists, and nothing makes a
  directory under a path that is not there.

A file refused on any of these is neither read nor added to, and the backend
answers no request while its record is refused (see *An answer that is not kept
is not sent*).

## The shape

The first line says what the file is. Every line after it is one answer, in the
order given. Every line is compact JSON with no newline inside it.

```
{"format":1}
{"at":{"secs_since_epoch":1760000000,"nanos_since_epoch":0},"application":"org.gnome.Fractal","portal":"secret","answer":"secret-handed-over","against":[3]}
{"at":{"secs_since_epoch":1760000004,"nanos_since_epoch":0},"application":"org.example.Stranger","portal":"secret","answer":"refused","why":"nothing-granted"}
{"at":{"secs_since_epoch":1760000009,"nanos_since_epoch":0},"portal":"open-with","answer":"unanswered","why":"not-identified"}
```

The file is **appended to and never otherwise rewritten** — the one exception is
a shortening (*Shortening it*, below) — for the agent's record's
reasons: keeping an answer does not read the file, a write the machine
interrupts costs the answer being written and nothing before it, and a reader
needs a JSON parser and nothing else. **Each answer is synced before the
application hears anything.**

## The first line

| Field | Meaning |
|---|---|
| `format` | Which shape the file is in. Required. `1` today. |
| `since` | The moment the file now starts at. Present only once a shortening has removed something. |
| `under` | The retention rule that removed it: `"forever"`, or `{"for-days":n}`. |

```
{"format":1,"since":{"secs_since_epoch":1759395200,"nanos_since_epoch":0},"under":{"for-days":7}}
```

A file whose first line is not a format line is not an answers file and is never
added to. A file whose `format` is higher than the reader knows is refused, not
added to.

`since` and `under` are the agent's record's two fields
(`docs/contracts/record-file.md`), in the same notation, and were added to this
file without raising `format`: a reader that does not know them ignores them.
**`since` cannot age out**, because no shortening walks the first line, so a
file shortened twice still says it was — and a file whose first answers have
aged out never reads as one for a machine where nothing was asked before its
first answer.

## An answer

| Field | Meaning |
|---|---|
| `at` | When it was answered, as seconds and nanoseconds since the Unix epoch. |
| `application` | The application's identifier, as its sandbox named it. **Absent** — not present and empty — when no application was named: a program with no sandbox, or a caller whose process was gone before it could be named. No name is ever invented. |
| `portal` | The portal asked. |
| `answer` | What it was answered with, and the fields beside it depend on it. |

### `portal`

`file-chooser`, `open-with`, `notifications`, `print`, `screenshot`,
`screen-capture`, `camera`, `microphone`, `clipboard`, `trash`, `wallpaper`,
`settings`, `inhibit`, `network-monitor`, `power-profile-monitor`, `secret`.

### `answer`

| `answer` | Meaning | Fields beside it |
|---|---|---|
| `secret-handed-over` | The application was handed its own secret. | `against`: the grant handles it was allowed by. |
| `opened` | The file was handed to the application that opens its kind. | `opener`: that application's identifier; `chosen`: `true` when it was the person's choice rather than a declaration; `against`: the grants over the file and over the opener. |
| `appearance-read` | The application read the appearance settings. | `against` |
| `appearance-sent` | The application was sent appearance settings that changed. | `against` |
| `network-read` | The application read whether this machine reaches anything, and whether the connection is metered. | `against` |
| `refused` | The grants refused it. | `why`: `nothing-granted` (the application holds no grant at all), `never-granted` (nothing it holds has ever covered this), or `lapsed` (a grant covered it and has expired). |
| `not-a-request` | What arrived was never a well-formed request. | `why`: `no-application`, `not-an-identifier`, `needs-a-path`, `not-over-a-path`, `not-a-full-path`, or `could-lead-elsewhere`. |
| `nothing-opens` | Nothing on this machine opens the file. | `why`: `{"no-application":{}}`, or `{"no-application":{"chosen":ID}}` when the person's choice is not installed; `the-file` (a program, empty, damaged, password-protected or unrecognised); or `unreadable`. |
| `unanswered` | It could not be answered with what it asked for, and neither the grants nor *what opens what* were the reason. | `why`: `not-identified`, `not-a-token`, `grants-unread`, `applications-unread`, `keyring-unavailable`, `not-written`, `not-a-file`, `not-decided-here`, `no-such-setting`, `appearance-unread`, `network-unread` (the network manager could not be asked how far this machine reaches), or `{"not-opened":{"opener":ID}}`. |

`against` holds the handles a person revokes grants by, as the grants file
writes them (`docs/contracts/grants-file.md`).

**Every answer is an identity, never a sentence.** The file says the same thing
on a Greek machine as on a German one; whatever shows it to a person words it in
their language.

### Two lines for one request

An answer is kept **before** it is carried out, so two answers can follow each
other for one request:

- `secret-handed-over` followed by `unanswered` / `not-written`, for the same
  application: the secret was allowed, and the application stopped listening
  before it could be written. The response was `2`.
- `opened` followed by `unanswered` / `not-opened`, for the same application:
  the file was allowed and handed to its opener, which did not open it. The
  response was `2`.

## An answer that is not kept is not sent

When an answer cannot be written — the disk is full, the file was refused — the
application is **not** sent it. It is sent the portal's refusal instead (`2`, or
`org.freedesktop.portal.Error.Failed` on the Settings portal), no secret is
written, no file is opened, and no `SettingChanged` is sent. That refusal is the
one thing the backend says that this file does not hold: saying *no* while the
record cannot be written is what keeps every *yes* in it.

## Shortening it

The file is kept **as long as the machine's record, and no longer**: it is
shortened under the same `[record].keeping` rule in the machine description
(`docs/contracts/machine-description.md`) that the agent's record is shortened
under, read as `alo_keeping::Keeping`. When a shortening runs is the session's
decision, not this file's.

- **It takes the rule and a moment, and nothing else.** No answer, application
  or portal can be named for removal.
- **What goes is every answer that reads and is older than the rule keeps** —
  an answer at the rule's own edge stays, and an answer dated after the moment
  the shortening runs at stays, because a clock put back is never a way to
  remove more.
- **A line that does not read is never removed.** It stays byte for byte, in
  its place, because nobody can say how old it is. A torn last line is ended
  with a newline and stays one line. *This differs from the agent's record*,
  which refuses to shorten at all around an unreadable line: here the rule
  holds over every answer that reads, and the unreadable line is kept anyway.
- **A file that would lose nothing is not rewritten**, first line included:
  `"forever"`, and a rule nothing is old enough for yet, touch nothing.
- **`since` moves to the rule's edge, and never back**: the later of the edge
  and any `since` already there. `under` names the rule of the last shortening
  that removed anything.
- **Whole or not at all.** The shortened file is written beside the file as
  `portal-answers.jsonl.shortening`, made `0600` and never through a link,
  synced, and renamed over the file. A machine that stops partway leaves the
  file exactly as it was; the next shortening removes what was left beside it.
  A file whose path no longer holds the file the backend has open, or whose
  first line is refused, is not shortened.
- **Nothing is answered while it is replaced.** The backend keeps each answer
  before sending it, under the same lock a shortening holds from its first
  read to the rename, so no application is answered — and no answer is kept
  into the file renamed away — while the file is replaced.

## Reading it

A line that does not read — one the machine stopped in the middle of, or one
naming an `application` that is not an identifier — is reported by its line
number, counting the format line as line 1, alongside every line that did read,
and never in place of them. A backend that starts over a file ending in the
middle of a line ends that line before its first answer, so the torn line stays
one line and the next answer is whole.

A reader that does not recognise a field inside an answer ignores it.

## Reading it to a person

The file holds identities; a person is shown sentences, in the language they
read, made by `alo-portals` from what the file holds. Nothing here draws the
list, and nothing here decides who may read the file.

- **`KeptAnswer::said`** gives three sentences and a moment: who asked —
  *Asked by* the `application`, or by *an application that could not be named*
  where `application` is absent; what the portal lets an application do; and
  what it was answered with. The last is said **with the words the backend
  answered with** — the same keys `Outcome::said` uses, `alo-capability`'s for
  a refusal of the grants and `alo-applications'` for a file nothing opens —
  and never a second set.
- **Where the file keeps less than the answer held**, the same sentence is used
  with a clause standing in for what is not kept, never a value guessed at:
  *what it asked for* for the path a refusal of a portal over a file named, and
  *that kind of file* for the kind nothing opens. A refusal of a portal over a
  facility loses nothing, and reads back exactly as it was said. Only
  `the-file` and `unreadable` under `nothing-opens` are said with two sentences
  of their own, because which of `alo-opening`'s findings it was is not kept.
- **`ReadBack::said`** says, above the answers, whether the file is whole
  (*nothing has been removed*) or shortened (*does not go all the way back*),
  and hands back `since` and the rule `under` it was shortened by (said by
  `alo-keeping`) beside that sentence. **`since` is a moment, never written into
  the sentence**: how a date is written belongs to the reader's region, not to
  their language.
- **Lines that did not read are said as a count** — *One line …*, *2 lines …*,
  in the reader's plural forms — beside every answer that did read, never in
  place of one. Where every line read, there is no such sentence.

## Versioning

`format` is `1`. Anything that would stop this version reading the file
correctly raises it; anything additive does not. A new `answer`, a new `why` or
a new portal is additive: an older reader reports that line as one it could not
read, with its number, beside everything it could.
