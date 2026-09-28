# A conversion's loss report is resolved against the host's fonts, and is not stable even there

**Version:** the pinned engine, `/opt/libreoffice26.2/program/soffice`, on the
development PC's WSL2 guest; `crates/alo-converting` at main, 2026-09-27.

**Whose:** ours.

**Behaviour:** `an_older_word_document_is_converted_and_what_it_lost_is_named`
expects the loss set `{FontSubstituted("Garamond"), FieldFixed(Date), Comments}`.
On this machine it sometimes reports `FontSubstituted("Liberation Serif")` as
well, and sometimes does not — **on the same machine, with the pinned engine
present, and no skip taken**. Four runs produced both outcomes.

The fixtures' own README says the documents set their text in Garamond, *"which
the image does not ship"* — so the expected loss was written against the font set
of the pinned **image**. The test runs the engine **installed on the host**, and a
host install resolves fonts through that host's fontconfig. Here: Garamond absent,
Times New Roman absent, Liberation Serif present in four faces, no msttcorefonts,
and `fc-match "Times New Roman"` returning Liberation Serif.

**No other lane can see this.** `this_machine_cannot_run_the_engine()` asks exactly
one path and does not search `PATH`; the pinned engine has no aarch64 build, so the
Mac lane skips all eight conversion tests and reports them passed. That is intended
and recorded in task 10 of the documents plan — but it means a converting result
from a lane without the pin is an **absence, not a corroboration**. It was read as
corroboration once, by two lanes, until one `ls` was run.

**Our response:** recorded as task 11 of
`../autonomy/v0-5-documents-and-paper-plan.md` rather than fixed, because where the
engine's fonts come from is a decision. **Do not fix it by installing fonts**: the
report is not monotone in font availability — the machine with *fewer*
Microsoft-compatible fonts reported *less* loss — so a font install moves the red
rather than removing it.

`lost()` is a user-facing answer. A report that varies with the host, and varies
between runs on one unchanged host, is worse to ship than one that is merely wrong,
because reading it twice tells a person nothing about which reading is true.

**Date:** 2026-09-27.
