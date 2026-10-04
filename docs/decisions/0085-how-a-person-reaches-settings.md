# How a person reaches Settings

**Status:** proposed — the chord is the owner's to accept, because a sixteenth
action costs every application on the machine a key combination for good.

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

## And a pointer road as well, not instead

A chord is no road for somebody who never learns one. `alo-desktop` already
carries pointer input to window state, so a second road costs no chord and no
new mechanism.
**Both, because either alone leaves somebody out.**
A person who works by keyboard and a person who has never been told a chord
exists are different people, and this project's second law is that no code
runs unless the person chose it. Choosing requires reaching.

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
