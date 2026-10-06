# A name is not a fact about the thing

**Concluded:** four separate instruments, built by three lanes on one day, each
reported something absent that was present or present that was absent. Each was
believed long enough to be acted on or reported, and one was sent to two
machines as a work list.

**What was true.** Every one of them asked a question of a **name** and read the
answer as a fact about the **thing**.

| the instrument | the name it trusted | what it missed |
|---|---|---|
| a wiring detector, lane B | `Type::` | a struct literal — `Popup { … }` names no `::` at all |
| the same, after the first fix | the file a type is defined in | a type built by a production function **in its own module**, which was the exact line somebody was pointing at |
| a unit scan, the third PC | `.service` in `multi-user.target.wants` | `alo-convertd.socket` — socket activation is not a `.service` name |
| a verification pass, the Mac | the `_tests.rs` suffix | `#[cfg(test)]` blocks inside files whose **names** say nothing about tests: `nested_approval.rs:215`, `nested_record.rs:203`, `nested_settings.rs:209`, `settings_places.rs:93` |

A fifth, from the same evening, is the same fault one level down: `-> &Screenshot
{` was read as a struct literal because it ends in a brace. The brace opens a
**function body**. The name of the token was trusted over its position.

And a sixth, which is the fault with no instrument at all: `TheMachine` is
declared **four** times in this workspace, `Recording` three, `Moved` twice,
each by unrelated crates. A count of implementations across the tree answers a
question nobody asked, and it reads as an answer because the name is the same.

*That sentence said **three** `TheMachine`s until it was checked against the
tree, in an entry about not trusting what a name suggests. The number came from
remembering three files rather than counting, which is the same move one level
further in.*

## A seventh, which the repository had already found and warned about

`crates/alo-dock/src/edge.rs` carries this, written before any of the above:

> [ADR 0076] carries that reversal, and its own second paragraph warns that its
> **title is now wrong and deliberately unchanged** — so a reader who learns
> from the filename that the dock is fixed to the bottom **has been misled by a
> record that says so.**

The file is `0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md`
and the dock is **not** fixed to the bottom; the owner reversed that within the
hour on 2026-09-30, and the record keeps its filename because nothing published
is renamed. So the repository contains a decision whose name states the opposite
of its contents, knows it, and says so in the one file most likely to be read
beside it.

**Which makes this entry's finding older than its instruments.** Four of us
built name-trusting checks on one day; somebody had already written down the
general shape and nobody went looking. That is the same failure as the rest, at
the level of the directory: `docs/misreadings/` exists so this is cheap the
second time, and it only works if it is read before a thing is built rather than
after it breaks.

## The mechanism

**A name is a label somebody chose. The thing is what the compiler, the kernel
or systemd does with it.** The two agree most of the time, which is exactly what
makes the gap expensive: an instrument built on the name is right often enough
to be trusted, and wrong on the cases that matter — the socket rather than the
service, the test module in a production file, the literal rather than the
constructor.

Each of these was built by somebody who knew the material. None was careless.
Three of the four were caught by **another lane reading the same question
differently**, not by the author re-reading their own work — and the author's
re-read is what produced the *second* fault in the first instrument, where a
correction moved the count the wrong way and the mechanism had not been touched.

## The cure

**Ask the thing, not its label.** Concretely, in the four shapes that bit:

- **Files:** do not trust a filename or a suffix to tell you what the code in it
  is. Find the `#[cfg(test)]` and compare by **position**. A production-named
  file holds test code in every crate here.
- **Units:** do not grep a wants-directory for one activation mechanism. A thing
  can be started by a socket, a path, a timer or a dbus name. Ask systemd what
  is enabled rather than asking a directory what it is called.
- **Types:** a name is not unique across a workspace. Resolve to the crate
  before counting, or say in the output that you did not.
- **Syntax:** a brace, an arrow and a `::` are tokens, not meanings. `-> &T {`
  and `T { … }` differ by position, not by characters.

And the general form, which is the one worth keeping:

> **When an instrument reports an absence, ask what it would have had to see to
> report a presence — and then go and look at whether the thing takes that
> form.** An absence is only evidence if presence was reachable.

## What it cost, and what it saved

The list that went to two machines was **216** entries. After the first fix it
was **243** — the number moving the wrong way, which is what said the fix had
missed. After the second it was **123**. The Mac then read all twenty-three in
their own crates and confirmed every one, which is the first independent
confirmation the detector has had, and found the fourth instance above while
doing it.

**One evening of reading against a day of wiring types that were fine.** Both
the hardware findings the same night — an installer that does not start on a
clean Windows machine, and a single-disk machine that cannot be installed at all
— came from the same move: reading the source before running the thing.
