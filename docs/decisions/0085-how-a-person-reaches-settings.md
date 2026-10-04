# ADR 0085 — How a person reaches Settings

**Status:** **accepted, 2026-10-04**, by the development PC under
`docs/autonomy/a-new-machine-becomes-a-lane.md`'s rule for a blocker that is a
decision: *write the ADR. You are the machine that understands why it matters;
waiting for one with less context to decide it is worse rather than safer.*
Its two exceptions are what needs the owner personally and what needs a lawyer,
and neither question here is either.

**This record held both questions open for most of a day, and that was the
error it now corrects.** The same section of the charter says *stopping while
work is available is the one outcome that is always wrong*. A sixteenth action
does cost every application a key combination for good — that is why the cost
is priced below rather than why somebody else should price it.

## The question

Nothing opens Settings. `SettingsWindow` is built, `settings_raster` lays it
out, it routes keys, and it refuses correctly in every tested case.
**Every construction of it sits in a test file or a `#[cfg(test)]` block.**
Task 18 of `docs/autonomy/the-shell-plan.md` is to give a person a road to it.
This records which road, and why the cheaper-looking one is worse.

## What was measured

In this repository at main `dd3fe93d`:

```
SettingsWindow::closed()        every construction in a *_tests.rs or tests/
opened_by_hand in production    all RecordWindow's method of the same name
"Settings" in alo-desktop       0 outside a comment
alo-compositor.rs               197 lines, names no window at all
alo-desktop main.rs             RunningWindow and FillingWindow, under the
                                comment *Shut, because nothing on a machine
                                opens it yet*
Action variants                 15, a closed list
default chords declared         33 lines naming an Action
actions handled in alo-shell    window_command.rs, canvas_command.rs
TheAgent and Launcher           fall through to None in
                                alo-dividing/keyboard.rs; no handler anywhere
alo-desktop's pointer road      the_pointer_is reaches window state already
```

## The road not taken, and why it is worse rather than cheaper

**Reaching Settings from the launcher spends no chord and does not work.**
`Action::Launcher` is declared and falls through to `None`; it reaches no
handler anywhere, so **the launcher has no surface either.**
*Open Settings from the launcher* would be one unreachable surface offering
another — the fault moved rather than closed, with a plan now able to say the
road exists.

## The road proposed, from this crate's own doctrine

**A sixteenth `Action`, bound by default to the conventional chord.**

`crates/alo-shortcuts/src/action.rs` prices a new action in its own header:
*every action here costs a chord that no application on the machine can ever
see again, and a system that took thirty of them would be a system whose
applications behave strangely for reasons nobody can find.* The cost is real,
and this file is where it is paid rather than waved through.

**The convention is what pays it, and that is the crate's own rule.**
`defaults.rs` says: *Conventional wherever being conventional costs nothing.
The machines this replaces are Windows machines, and fifteen years of muscle
memory put maximise on `Super+Up`… Inventing better ones would be paying for a
preference with everybody else's first afternoon.* Settings has that
convention and it is `Super+I`.
**Leaving it unclaimed would be the surprising choice.**

**And the cost is smaller than the header's worst case, measurably.**
`Super` is this system's modifier: the existing defaults put the agent, the
launcher, maximise, the halves of the screen and the canvas on it. An
application receiving `Super+I` today is one already fighting the system for
`Super`. Held to the header's own test — what an application can no longer
see — the answer is one combination on a modifier the system already owns.

**A person can move it or clear it, and the format already expects that.**
`changes.rs`: *what is stored is the difference — the actions a person moved,
and the actions a person cleared — and everything else comes from the code
that is running*, written against exactly this case: *a release that added an
action would reach them with it unbound.* With a default it reaches them
bound, and clearing it is a thing the format already holds.

**Decided: taken.** `Super+I` becomes the sixteenth action's default. The cost
is one combination on a modifier this system already owns, against a convention
fifteen years of muscle memory already carries — and `alo-shortcuts` is this
lane's crate, so the action is this lane's to add. **Leaving it unclaimed would
have been the surprising choice**, and leaving the decision unmade was the
expensive one.

## And a pointer road as well, not instead

A chord is no road for somebody who never learns one. `alo-desktop` already
carries pointer input to window state, so a second road costs no chord and no
new mechanism.
**Both, because either alone leaves somebody out.**
A person who works by keyboard and a person who has never been told a chord
exists are different people, and this project's second law is that no code
runs unless the person chose it. Choosing requires reaching.

### Where it goes, from the design rather than from this record

This record left the place open. The design answers it, and the answer is that
**Settings is an application on the Dock** rather than a control standing
beside the applications. Five nodes in the design are named Settings. In the
overflow lists among them it sits under a heading reading *More open apps*,
after a divider, beside Mail, Calendar, Notes and Terminal, and above *Show
all windows*. That list appears on the `Dock 09 · Overflow` sheet and again on
the Left, Right and Top edge specifications.

**One of the five is a definition rather than a placement.**
Reading it as a placement is the mistake this section exists so that nobody
repeats. Node `348:23618`, `Dock edges / Button / Settings`, is one of fifteen
44x44 buttons in an evenly spaced row inside a frame named `Dock edges ·
Components`: Docs, Browser, Blender, Files, Mail, Calendar, Notes, Terminal,
Settings, Music, Photos, Tasks, Search, History, More.
**A component sheet is not a layout.**
On its own it says a Settings button exists, and not where one goes.

Its size is worth keeping. **44 by 44**, which is the owner's ruling of
2026-09-30 that a click target stays at least 44 while a glyph may shrink.
`places.rs` makes every place `ICON` across, which is 48, so the weakness
recorded in that file does not stand in the way here.

### Which means the mechanism really is one that already exists

Because the design makes Settings an application, the Dock needs no new kind
of item, and the earlier sentence in this record was right for a reason it did
not give. `clicking.rs` already answers `OpenAWindowNearTheView` for an
application with nothing open, and `AppId::named` accepts any non-blank name,
so `"Settings"` is legal today.
**The pointer road already exists, pointed at a window nothing opens.**

### What is open is a sentence rather than a type

`crates/alo-applications/src/application.rs` holds a rule, and a test holds
the rule to the code:
`the_words_around_an_identifier_are_translated_and_it_is_not`.
The reason it gives is that an application's name *is packaged in and is not
ours to translate*, and `AppId::name()` says the same of itself — *an
application's name came off this machine rather than out of a vocabulary.*

**Settings is ours.** The reason that rule gives does not reach it: nobody
else packaged it, and a European product shipping its own control labelled in
English has a bug rather than a convention. So the question this record hands
over is narrow, and it belongs to the owner:

- **Is an application the system itself provides an exception to that rule?**
  Yes means that test's name stops being true of every application, and the
  exception has to be visible in the type rather than kept as a habit. No
  means the Dock shows one English word to every person who reads another
  language.

**Decided: yes.** An application **this project packages** takes its name from
the vocabulary; an application **somebody else packages** keeps the name they
were given. The rule's own stated reason is the discriminator rather than its
wording: *packaged in and not ours to translate* is a fact about third-party
software, and it says nothing about ours.

Three things leave no other answer available:

- `CLAUDE.md` is absolute — *user-facing strings are externalized from day one.
  Hardcoded English is a bug, and "English plus the big five" is the same bug
  wearing a business case.* A Dock entry reading `Settings` in every language is
  that bug, shipped by us, in the first row a person sees.
- The Dock's own accessibility note in the design says each icon carries **a
  tooltip and a spoken name**. A screen reader saying an English word into a
  German session is the same failure heard rather than seen.
- The rule being bent was never about us. `application.rs` says the name is
  *the machine's and the packager's, and neither is translated* — and for
  Settings this project is the packager, so that sentence decides nothing here.

**What it costs, priced rather than waved through.**
`the_words_around_an_identifier_are_translated_and_it_is_not` stops being true
of *every* application, so **the exception has to be visible in the type rather
than kept as a habit**: an application carries either a packager's string or one
of our words, and nothing may guess which. How the type says so belongs to the
change that makes it, in `alo-applications` — the applications lane's crate by
charter line 152. **This record decides the rule; that lane carries it.**

## What this does not decide

- **Whether `RunningWindow` and `FillingWindow` share the road.** They carry
  the same fault, and the desktop says so about them in its own struct
  comments. Naming them is not owning them, and deciding for them belongs to
  whoever measures them.
- **What Settings offers once open.** That belongs to
  `docs/autonomy/where-a-persons-settings-are-kept-plan.md`, whose task 7
  already declares the sentences. This records only how a person arrives.

## Consequences

- `where-a-persons-settings-are-kept-plan.md` task 7 becomes finishable. Its
  status says the sentence holds and *no person can read it*; a road is what
  makes the second half false.
- `hands-on-the-desktop-plan.md` task 5 becomes finishable. Its failing clause
  is *a person can turn each off*, and a Settings surface is the writing end it
  names.
- A sixteenth action is a **public surface**: additive, documented in the
  change that adds it, and `Action::ALL` gains a member that whatever asserts
  over that list will hold.
