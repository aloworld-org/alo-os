# Settings refusals carry no credential

- Date: 2026-09-08
- Workstream: model selection and configuration (`alo-choosing`)
- Contributor: Claude Code
- Task: Fix settings-error redaction, published on its own

## The leak

`NotSet::NotUnderstood` carried a `toml::de::Error` whole, and a TOML error
quotes the line it failed on:

```text
TOML parse error at line 6, column 1
  |
6 | key = "sk-live-0123456789"
  | ^^^
unknown field `key`
```

The sentence a person reads never contained that — `choosing.settings.not-understood`
is filled with the path and nothing else — but the value did. A `Debug` of the
refusal, a log line that formatted one, or a support bundle that captured one
would have carried somebody's API key.

`CLAUDE.md`: *credentials never appear in logs, errors, or commits.*

**It was found by a test written for the settings file's own protection.** The
file has no `key` field, so a pasted credential is refused — and the refusal
then repeated what it had just refused to store. That residual was reported in
`three-model-choices-in-the-backend.md`; this is it closed.

## What changed

`alo_choosing::NotToml` — what a parser said, with everything the file said
taken out of it. `NotSet::NotUnderstood` carries one of these instead of the
raw error, built at the single place a parser's error becomes one of ours, so
there is no road by which an unredacted one reaches a caller.

**The rule: a quoted run is kept only if it is one of this file format's own
words.** `format`, `answers`, `provider`, `endpoint`, `needs-a-key` and the rest
— a list, next to the shapes it describes, rather than a heuristic, because a
heuristic is what fails on the one input nobody tried. Everything else becomes
`…`.

Both delimiters are treated alike. TOML quotes a *value* with `"` and a *name*
with a backtick, so keeping backticked runs would have been nearly right — and
nearly right here means a key pasted as a bare name, which TOML accepts,
arriving in a log as ``unknown field `sk-live-…` ``.

**Useful diagnostics survive.** The line and column are arithmetic about the
file rather than anything in it, and they are what sends somebody to the right
place. So does the kind of mistake and the list of names this format really has:

```text
unknown field `…`, expected one of `format`, `answers`, `brought`, `provider`, `reading` (line 7, column 2)
```

`Debug` is written by hand rather than derived — not because the derived one
would differ today, but because the next field somebody adds here would be
printed too, and what this type prints has to be decided rather than defaulted.

## Evidence

`crates/alo-choosing/src/unreadable.rs` — four tests, using the synthetic secret
`sk-live-DO-NOT-LOG-9f3c1a`:

| Test | What it proves |
|---|---|
| `the_words_this_format_uses_for_itself_are_kept` | `unknown field `…`, expected one of `format`, `answers`` — the diagnostic survives |
| `nothing_the_file_said_survives_either_kind_of_quote` | the secret redacted as a value, as a bare name, in a duplicate-key message, and behind an unbalanced quote |
| `where_it_gave_up_is_counted_from_one` | line and column, including a newline belonging to the line it ends |
| `neither_display_nor_debug_can_carry_a_credential` | **the real road** — the secret pasted into a real settings file, read through `written::read`, and absent from `Display`, from `Debug`, and from the `Debug` of the whole refusal |

**Two existing assertions were updated rather than deleted**, and both are
deliberate losses:

- `a_key_nobody_declared_is_refused` asserted the refusal quoted the typo back
  (`readng`). A key nobody declared is exactly where a pasted credential lands,
  and this cannot tell a typo from an API key. It now asserts the line, the kind
  of mistake, the declared names — and that the typo does **not** appear.
- `a_settings_file_has_nowhere_to_put_a_credential` asserted the message named
  the `key` field. It now asserts the quoted `` `key` `` does not survive.
  (Not `contains("key")`: `needs-a-key` is one of this format's own words and
  survives on purpose.)

Gates: `cargo fmt --check`, `clippy --workspace --all-targets -D warnings`,
`cargo test --workspace`, `RUSTDOCFLAGS="-D warnings" cargo doc`, the
supervisor's own tests, and the BPF target's fmt and clippy on the pinned
nightly. Kernel tests took the shared lock.

## What this does not cover

`alo-agentd`'s `NotDescribed::NotUnderstood` carries a raw `toml::de::Error` for
the **machine description**, which is the organisation's file. It has the same
shape and not the same exposure — that file has no credential field and no
derived keyring name — so it is named here as the next place to apply the same
treatment rather than left unmentioned.

## Proposed integration updates

**CHANGELOG.md** — nothing user-visible; the sentence a person reads is
unchanged.

**ROADMAP.md** — no tick.

**docs/autonomy/QUEUE.md** — no new item.

**docs/autonomy/STATE.md** — a settings refusal can no longer carry a pasted
credential into a log, a `Debug` or a support bundle; the line, the kind of
mistake and this format's own field names survive, and nothing the file said
does.
