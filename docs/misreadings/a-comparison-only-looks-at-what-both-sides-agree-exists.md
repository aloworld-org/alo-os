# A comparison only looks at what both sides already agree exists

**Concluded** — as a wording problem: *the programmatic name does not contain
the visible label, so the names need fixing.* True, and much too small.

**True**, measured on main at `12e72eb7`:

```
crates/alo-shell/src/window_controls.rs:100-102   DRAWN
    Action::MinimiseWindow, Action::MaximiseWindow, Action::CloseWindow
crates/alo-access/src/tree.rs:309-310             ANNOUNCED
    words::CLOSE_THIS_WINDOW, words::ARRANGE_THIS_WINDOW
anything comparing the two lists                  none
```

**A screen-reader user is told that window has two buttons, one of them *arrange
this window*. It has three, and none of them is that.** Minimise and maximise
are drawn and never announced; arranging is announced and never drawn —
snapping is a chord with no button in the strip. **Wrong in both directions at
once**, and only a person using a screen reader ever meets it.

**The mechanism.** Two lists must agree and nothing compares them. Worse, **the
obvious fix would have hidden it**: a containment test over the name pairs — the
natural answer to the wording question — passes while saying nothing about the
two missing buttons, because **a check over pairs never asks whether the pairs
are all there**. A green test certifying a list short by two is worse than no
test.

**The cure.** When two lists must agree, test the **sets**, not the pairs: every
drawn control has an announced name and every announced name has a drawn
control, counted both ways. Then remove the second list where you can — **this
repository had already written that answer down**, in `Control::for_setting`,
which takes its name from `Setting::word` *so there is no second list to keep in
step*. The next surface was then built the old way. **The knowledge was present
and unread**, which is worth more than a new rule.

**Found by** the applications lane, answering a different question. Verified
here rather than restated. Settled in
[ADR 0089](../decisions/0089-what-a-control-is-called.md).
