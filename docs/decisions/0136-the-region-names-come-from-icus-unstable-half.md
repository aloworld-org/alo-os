# ADR 0136 — The region names come from icu's unstable half

**Status:** accepted, 2026-10-28, by the owner, who asked for this record.

**Context:** [ADR 0011](0011-the-base-is-rented-and-the-image-is-a-container.md) (what is rented
and never edited); `docs/features.md`, *a region for dates and times, chosen
apart from the language*; `docs/contracts/person-settings.md`, `formats.toml`;
`crates/alo-formats`.

## The decision in one line

`alo-formats` turns on `icu`'s **`unstable`** feature for one API —
`icu::experimental::displaynames::multi::RegionDisplayNames` — and uses it in
**one file**, `crates/alo-formats/src/region_names.rs`, held by a test that
fails if its answers change shape.

## What it is for

Settings lists every region a person may write dates and times like — the 249
codes ISO 3166-1 assigns, and Kosovo — **named in the language the person
reads**. A Maltese reader reads Maltese names; an Irish reader reads Irish ones.

The names are CLDR's. CLDR has every one of those regions named in all 24
official EU languages, kept current by people whose work that is. The only
road in Rust to those names, without writing them out ourselves, is the
display-names API, and `icu` 2 ships it under `experimental`, which the crate
only compiles with `unstable` on.

## The options, and why this one

1. **CLDR's names through `icu`'s unstable API** — this decision.
2. **A table of our own**, 250 names × 24 languages. Six thousand strings
   nobody here can check, wrong first in Maltese and Irish — the languages
   `CLAUDE.md` says we may not leave behind — and drifting from CLDR from the
   day it is typed.
3. **English names for everybody.** A machine that speaks Swedish and lists
   *Austria* where a Swede looks for *Österrike*. That is the bug the i18n rule
   names.
4. **Codes, not names** (`AT`, `BE`). Nobody chooses a country by its ISO code.

Only the first gives every person their own language's names without a table
we would have to maintain.

## Where it is confined

- **One file uses it**: `crates/alo-formats/src/region_names.rs`. No other file
  in the workspace names `icu::experimental`, and that is checkable with one
  search. Everything else — dates, times, numbers, the first day of the week,
  collation — uses `icu`'s stable API.
- **One crate turns the feature on**: `crates/alo-formats/Cargo.toml`, with a
  comment pointing here.
- **It is pinned by `Cargo.lock`**: `icu` 2.3.1 and `icu_experimental` 0.6.0 as
  of this record. An upgrade is a change somebody makes on purpose, not one
  that arrives.

## What to check on every upgrade of `icu`

The unstable API may change between minor versions without notice. It fails in
one of two ways, and only one of them is loud:

- **It stops compiling.** `region_names.rs` breaks and nothing else does.
  Repair it there.
- **It compiles and answers something else** — no names, or names in another
  language. This is the quiet one, because `region_names` falls back to the
  English name and then to the code, so a person would still see a full,
  sorted list, in English, whatever they read.

`crates/alo-formats/tests/the_region_names_keep_their_shape.rs` holds the
second. It runs on every build, and it fails if:

- Germany, Belgium and Kosovo are not called exactly what CLDR calls them in
  English, German and French;
- any of the 24 languages calls Germany by its English name or its code —
  which is what the fallback would give;
- Kosovo is missing from the list, or listed by its code, in any of the 24.

So on an upgrade: build, run `cargo test -p alo-formats`, and if a name in that
test changed because **CLDR** changed it, update the test and say so in the
commit. If it changed because the **API** did, fix `region_names.rs`.

**When `icu` moves display names to its stable API**, drop the `unstable`
feature and mark this record superseded.

## Kosovo stays on the list

`XK` is not an ISO-assigned code. The European Union, CLDR and most of the world
use it, and people keep calendars there. **The owner confirmed on 2026-10-28
that it stays.** `crates/alo-formats/src/regions.rs` says why in its header, and
the test above fails if it is ever dropped or left unnamed.
