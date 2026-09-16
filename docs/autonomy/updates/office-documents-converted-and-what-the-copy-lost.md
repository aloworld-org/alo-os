# `.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost

**Date:** 2026-09-16
**Workstream:** v0.5 documents and paper — task 2 of
`docs/autonomy/v0-5-documents-and-paper-plan.md`
**Decision it builds:** [ADR 0039](../../decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md), option A, accepted 2026-09-15
**Responsible contributor:** Claude (kernel-loop worker), for the repository's owner
**Status:** ready for integration

## What changed, for a person

A Word document, an Excel spreadsheet or a PowerPoint presentation someone was
sent can now be converted on this machine into a PDF copy, and the machine says
**exactly what the copy could not carry**: a font it did not have (by name), a
date or formula that updates by itself and is now fixed, macros, a picture or
data linked from somewhere else that was not fetched, comments, tracked changes.
When nothing was lost it says so in its own sentence — and it only says that
when it checked both the original and the copy. The original is never changed
or replaced, a file already called what the copy would be is never written
over, and nothing is sent anywhere to be converted.

## What was built

### `crates/alo-converting` (new)

| File | Responsibility |
|---|---|
| `verbs.rs` | `convert_document(file, into)`: `Effect::Change`, grants over both. `into` is the folder; the copy's name is the document's own ending `.pdf`, never the model's |
| `converting.rs` | The executor. Asks the grants about the copy's own path at the call's moment (a folder granted as `Reach::File` does not cover what goes inside), opens the document, decides its kind from its bytes with `alo-opening`, creates the copy, hands both to the service, removes the copy on any failure. `Done`, `Converted`, `NotConverted` |
| `opening.rs` | `openat2` + `RESOLVE_NO_SYMLINKS` for the document and folder, a hard-linked document refused, the copy created `O_EXCL` relative to the folder handle, removed only while its name still leads to the file this created |
| `passing.rs` | Two descriptors across a Unix socket (`SCM_RIGHTS`, via `rustix`) — the service never learns a path |
| `wire.rs` | The hand-written protocol: one request line, an answer ending `end`. A reply that contradicts itself is no reply |
| `service.rs` | `ConvertingService`, a socket path on this machine; no address form exists |
| `serving.rs` | The service's side: read the original (≤ 128 MiB), re-decide its kind (the client's word is not taken), inventory it, convert in a private scratch folder, inventory the PDF **before** a byte is written to the copy, write it |
| `engine.rs` | **The only file naming the engine.** Fixed argument list per conversion, cleared environment, `HOME` in scratch, 120 s limit then killed |
| `conversion.rs` | The closed set of three conversions |
| `carried.rs` | `Carried::Everything` / `NotEverything`, `NotCarried`, `Field`, `Linked`, `FontName` (made safe to say) |
| `inventory/` | `original.rs` + `word.rs`, `excel.rs`, `powerpoint.rs`, `theme.rs`, `linked.rs`; `copy.rs` (PDF fonts); `difference.rs` |
| `zip.rs`, `xml.rs` | Reading inside a document with `miniz_oxide` and `quick-xml` (ADR 0039 §5), each part capped at 64 MiB and 100:1 past 1 MiB, the document at 256 MiB |
| `machine.rs` | `with_what_converts`: `alo-opening` is told the three formats convert **only when the service answers** |
| `recording.rs` | The record entry: `Entry::ran(…).telling(what the person was told)` |
| `words.rs` | 37 strings, each with a translator's note |
| `bin/alo-convertd.rs` | The service `main`: takes its listening socket from standard input |

### Elsewhere

- `crates/alo-record/src/entry.rs`, `docs/contracts/record-file.md`: an additive
  `told` stamp (absent when empty, `format` stays `1`) — see *Decisions*.
- `crates/alo-saying`: the crate's words collected; LibreOffice added to
  `EVERYTHING_WE_RENT`.
- `crates/alo-by-hand`, `docs/by-hand.md`: the verb answered by hand.
- `docs/contracts/agent-verbs.md`: *The converting verb* and a class row.
- `image/Containerfile`: `ARG THE_CONVERTER=26.2.6` and its sha256, a
  `converter` stage that checks the digest before unpacking, the publisher's RPMs
  installed with `dnf`, `test -x` on the engine, `alo-convertd` built and copied,
  the socket enabled, login 60992 asserted.
- `image/usr/lib/systemd/system/alo-convertd.socket` and `.service`;
  `image/usr/lib/sysusers.d/alo.conf`: `alo-convert` 60992.
- `crates/alo-image/src/converter.rs` (reader) and `converts.rs` (checks), four
  `Wrong` variants, `Image::converter`. `checking.rs`'s *agent is no login*
  fixture moved from 60992 (now taken) to 60993.

## The pinned engine on the gate machine

Installed 2026-09-16 on the WSL Ubuntu 26.04 distribution the gates run in, from
the publisher's release, digest checked before unpacking:

```
LibreOffice 26.2.6.3 8221e31b3ac356a1623c672912a3d2b492f7e3d1
https://download.documentfoundation.org/libreoffice/stable/26.2.6/deb/x86_64/LibreOffice_26.2.6_Linux_x86-64_deb.tar.gz
sha256 fd0e8f8f2408dd2e5b90286e60f3f97cf566ba441cd48cfc5bcc68067303e0bc  (deb, gate machine)
sha256 9833c61bfbec0905c6da54f82ef56123818661c9fec99706d9afece3ad7e9988  (rpm, pinned in the image)
installed at /opt/libreoffice26.2 with dpkg -i
```

26.2.6 rather than 26.8.0: the newest release of the older maintained line, not
a `.0`. The plan names this install as the lane's own step; nothing else on the
shared distribution was changed.

## Measured against the owner's documents

Converting each through the pinned engine, and reading both files:

| File | What the copy did not carry | PDF fonts |
|---|---|---|
| `sample.docx` | Garamond; a date field; a linked picture; comments | `NotoSerif-Regular` |
| `sample.xlsx` | Garamond; a formula using the current moment; comments | `NotoSerif-Regular` |
| `sample.pptx` | Garamond; comments | `NotoSerif-Regular` |

Exactly the README's list. The theme fonts (Aptos, Aptos Display, Aptos Narrow)
are declared in all three and set on no text, and are not reported. No PDF used
compressed object streams. A synthesised Word document set in Liberation Serif
converts with `Carried::Everything`.

## Decisions made here

1. **`into` is a folder, and the copy's name is derived.** The ADR names the
   verb `convert_document(file, into)` and says the copy goes "in the folder the
   person chose". A name the model chose would be a place nobody picked.
2. **Socket activation hands the socket on standard input** (`StandardInput=socket`,
   `Accept=no`). Adopting systemd's descriptor 3 needs `unsafe` in Rust; standard
   input is a descriptor `std` already owns. Same activation the ADR describes,
   by the one road `forbid(unsafe_code)` leaves. The tests start the binary the
   same way.
3. **The record gained a stamp, and it touches a crate this plan "reads and
   never edits".** ADR 0039 §6 (accepted after the plan was written) requires
   the record to keep what the copy could not carry, and `alo-record` had
   nowhere for it. The change is `Entry::telling`, the `from_another_machine`
   shape: additive, absent when empty, never changes what happened. Flagged for
   `alo-record`'s owner.
4. **The copy is inventoried before it is written to the person's folder**, so a
   copy nobody could check never exists where a person can see it even briefly.
5. **A font counts only when set on text**, following each format's own
   inheritance (documented per file, with where it stops following). A font is
   carried when the PDF contains its family, compared past subset tags, styles
   and `PS`/`MT` endings.
6. **The service re-decides the document's kind itself** rather than trusting
   the request.
7. **In the image, `/opt` becomes a real directory** before the engine installs,
   because Fedora bootc links `/opt` into `/var`. Unmeasured until built.

## Verification

All on the WSL Ubuntu 26.04 gate distribution, `CARGO_TARGET_DIR` the loop's,
`RUSTDOCFLAGS=-D warnings`:

- `cargo fmt --all` — clean.
- `cargo clippy -p alo-converting -p alo-record -p alo-image -p alo-saying -p alo-by-hand --all-targets -- -D warnings` — clean.
- `cargo test -p alo-converting` — 57 unit, 9 `converting_a_real_document`, 3 `converting_reaches_nothing`: all pass. The conversions run the real `alo-convertd` binary and the real engine.
- `cargo test -p alo-record` — pass. `cargo test -p alo-image` — 252 unit and the integration tests pass.
- `cargo test -p alo-saying -p alo-by-hand -p alo-collected` — pass.
- `cargo test -p alo-opening --test converting_waits_on_its_decision` — pass.
- `cargo doc --no-deps -p alo-converting -p alo-image -p alo-record` — no warnings.
- Not run by this worker, by instruction: the full workspace suite.

## Not done, and owed

- **The image has not been built or booted.** The `converter` stage, the `dnf`
  install over a replaced `/opt`, the service's sandbox and the engine working
  under `PrivateNetwork=yes` / `RestrictAddressFamilies=AF_UNIX` /
  `ProtectSystem=strict` are declared and held by `alo-image`, not observed.
  First thing to measure on a built image; findings go in `docs/quirks.md`.
- **No certified hardware.** WSL is development evidence only.
- **The shell's window** that shows a copy and the sentences is the shell plan's.
- **The verb is declared and carried out, not offered by a turn**, like printing.
- **Older `.doc`/`.xls`/`.ppt` and OpenDocument** are recognised and not
  converted (ADR 0039 leaves each to a registration and a real file).
- **Inventory limits:** Word table styles; a PowerPoint layout's or shape's own
  list styles and a second master; Excel conditional formats. Where these would
  decide a font the inventory follows the parent style, so it can name a family
  that run does not use.

## Proposed updates for the integration owner

- **CHANGELOG.md:** "Documents people are sent — `.docx`, `.xlsx`, `.pptx` —
  convert on the machine into a PDF copy, and the machine says what the copy
  could not carry: a missing font by name, a date that no longer updates, a
  linked picture it did not fetch, comments. The original is never changed, and
  nothing is sent anywhere to be converted."
- **ROADMAP.md:** v0.5 `.docx`, `.xlsx`, `.pptx` open — `- [x] The code.`; the
  machine half stays open.
- **QUEUE.md / STATE.md:** documents and paper task 2 done; task 4 is next.

## After the first refusal

The workspace gate refused the first handoff twice on
`what_is_advertised::tests::a_refused_workspace_file_is_told_in_the_persons_words_naming_no_path_owner_or_mode`
in `alo-agentd`, which this change had not touched. The test looks for the uid
`989` it hands a file to in the person's answer, and found it inside
`"port":42989` — the port the kernel gave the test's own listener, which the
answer is right to name. The fix takes the machine's own advertised port out of
the line before looking for anything that must not be there, and asserts it
appears nowhere else; every leak it looked for is still looked for. Changing the
uid instead would only move the collision to a different port.
