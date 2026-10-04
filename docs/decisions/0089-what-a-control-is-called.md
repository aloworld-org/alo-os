# ADR 0089 — What a control is called, when two vocabularies name it

**Status:** accepted, 2026-10-04, by the Mac lane under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision: *write the ADR. You are the machine that understands why it matters.*
**Date:** 2026-10-04
**Context:** task 8 of `docs/autonomy/access-and-language-plan.md`, whose second
half is blocked on this; EN 301 549 clauses 11.2.5.3, 11.2.4.6 and 4.1.2;
`crates/alo-access` (`tree.rs`, `words.rs`), `crates/alo-shortcuts`
(`action.rs`, `words.rs`), `crates/alo-shell` (`window_controls.rs`,
`window_control_label.rs`); `CLAUDE.md`'s rule that user-facing strings are
externalised from day one, in 24 languages.

## The question in one line

**A control is drawn with one crate's words and read aloud with another's.
Which of the two is the other's source — and what happens to a control that
only one of them knows about?**

## What was measured, 2026-10-04

The plan asked this as a wording question: clause 11.2.5.3 says a programmatic
name must *contain* the words a person sees, and

```
drawn   shortcuts.action.close-window   "Close the window"   Action::said
read    access.close-this-window        "close this window"  Control::of
```

does not. That is true, and it is the smaller half. **The two vocabularies also
disagree about which controls exist.**

```
window_controls.rs:98   let [minimise, maximise, close] = enabled;
                        drawn: MinimiseWindow, MaximiseWindow, CloseWindow
                               at x-offsets 0, 36, 72

tree.rs:308             Surface::WindowControls => vec![
                            CLOSE_THIS_WINDOW,    "close this window"
                            ARRANGE_THIS_WINDOW,  "move this window"
                        ]
```

So a person using a screen reader is told that the window in front of them has
**two** buttons, one of which is *move this window*. It has **three**, and none
of them is that: `SnapLeft` and `SnapRight` are chords with no button in the
strip (`window_command.rs:63`). **Minimise and maximise are drawn and
unannounced; *move this window* is announced and not drawn.** The reader's
account of that surface is wrong in both directions at once.

Neither a test nor a translator could have caught it. The two lists are built
in different crates from different vocabularies, and nothing compares them —
which is the same shape as *a check that stands in for the thing is not the
thing*, with no check at all standing where the comparison would be.

## The answer already exists in the file, for settings

`tree.rs:110` is `Control::for_setting`, and its own rustdoc says why:

> **The name comes from `Setting::word` rather than from here**, so a setting
> added to `Setting::ALL` arrives in the tree already named and there is no
> second list to keep in step. A tenth setting nobody teaches this file about
> is a tenth control, not a missing one.

That is this decision, made once already, for the nine access settings. What is
left is to make it true of the controls a person acts on, where the second list
is `alo-access`'s own words rather than a list of names in another file.

## The options

**A — the access words become the source.** The drawing crates read
`alo_access::words` for their labels.
- *Costs:* `alo-shortcuts` would depend on `alo-access` to draw a button, which
  inverts the dependency: `alo-access` already depends on `alo-shortcuts`, so
  this is a cycle. It also makes the spoken form authoritative over the visible
  one, which is backwards from what 11.2.5.3 asks — the clause makes the
  **visible** label the thing that must be contained.

**B — the drawn words become the source.** `Control::for_action(action)` takes
its name from `Action::word()`, exactly as `for_setting` takes `Setting::word`.
- *Costs:* the spoken name inherits the phrasing and capitalisation of a
  label — a reader says *Close the window, button* rather than *close this
  window, button*. Three `alo-access` words are retired. A control that is
  drawn and not in the tree becomes a compile-time impossibility only where the
  tree is built from the same list the strip is; where it is not, it stays a
  test.
- *Gains:* no new dependency, no cycle, 11.2.5.3 holds **by construction** in
  all 24 languages rather than by a test that compares two strings, and a
  translator is handed one string per control instead of two that must agree.

**C — a third vocabulary both read.** A new crate holding one word per control.
- *Costs:* a third list to keep in step with two, which is the problem with one
  more participant. Every existing word moves, and every citation of a key in
  every plan and record goes stale on the same day.

**D — keep both and hold them together with a test** asserting that the read
name contains the drawn one.
- *Costs:* it enforces the lesser half and cannot see the greater one. A test
  comparing the two lists' *contents* would have said nothing about minimise
  and maximise being absent from one of them, because a containment check over
  pairs never asks whether the pairs are all there. It also leaves two
  translations per control, which is the thing that drifts.

## The decision

**B, with the counts held as well as the words.**

1. **A control a person acts on takes its name from the action it performs.**
   `Control::for_action(Action)` is added beside `Control::for_setting`, and
   takes `Action::word()`. The three `alo-access` words that named actions —
   `CLOSE_THIS_WINDOW`, `ARRANGE_THIS_WINDOW` and any that follow — are retired
   under ADR 0068's fourth rule rather than left alive with a marker.
2. **`alo-access` keeps its own words for everything nothing draws**: surfaces,
   regions, lists and labels — *who is signing in*, *the recovery screen*, *what
   can be changed*. Those are not controls with a visible label, so there is no
   second string for them to disagree with.
3. **The tree's window-controls surface is built from the same list the strip
   is drawn from**, so the two cannot differ in what exists. Where a surface's
   controls are not yet built from one list, a test asserts the counts match
   and names the surface — the comparison that was missing.
4. **A control may carry a word of its own or an action, never both**, which is
   the shape `Called` took in `alo-applications` the same day for the same
   reason: an exception that is visible in the type cannot be forgotten, and
   nothing may guess which kind a name is.

## What it costs

- **Three retired keys, and a spoken form that changes.** A person who used a
  reader yesterday hears *Close the window* where they heard *close this
  window*. That is a real change to what somebody hears and it is the price of
  the two halves agreeing; the alternative is that they hear a sentence nobody
  can see.
- **One crate learns another's vocabulary.** `alo-access` reading
  `alo_shortcuts::Action::word` is a dependency it already has, but it is a new
  kind of use: the accessibility tree now has an opinion that lives in another
  crate's strings. That is the point of the decision and it is also its risk —
  changing a shortcut's label now changes what a reader says, and the note on
  that word has to say so.
- **This does not fix the other three surfaces.** Only the window controls are
  built from one list today. The rest are decision 3's second sentence — a test
  that names them — and each is work somebody has to do.

## What would change this decision

A control whose visible label is wrong to speak — an icon with a one-word label
whose spoken name must be longer, or a label whose capitalisation is wrong read
aloud in a language that has no case. The remedy then is **not** a second
vocabulary: it is that the word carries both forms, so there is still one
string per control and one translator deciding both.
