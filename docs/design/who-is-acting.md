# Who is acting: telling alo apart from the person

alo is **deep teal `#0F6B72`**. The person is **navy `#102A43`**.
[ADR 0067](../decisions/0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md)
adopts both and records the risk they carry: they are two dark, desaturated
blues, and *you* and *the machine* have never before been told apart by colours
that close. Terracotta and navy could not be confused; these can.

**Both anchors are approved and neither moves.** This document is how the
difference is made legible anyway, and how it is measured on a real screen
rather than argued about over a swatch.

## The rules that do the work

The colour is never the signal. It is the confirmation beside one.

- **No alo state is ever a small teal dot alone.** At any compact size the teal
  arrives with the **four-corner alo mark**, or with a plainly written **alo**
  label. A dot on its own is not permitted anywhere, at any size, in any theme.
- **Work in progress is a named cursor and a boundary.** When alo is acting, its
  cursor carries its name and the affected object carries a boundary around it,
  so what is being touched is visible without reading a colour at all.
- **Human selection differs in shape, not in hue.** A navy version of the alo
  indicator is not a human indicator. Selection uses its own shape and its own
  control treatment; someone who cannot see either colour must still be able to
  say which of the two they are looking at.

The rule underneath all three is [ADR 0010](../decisions/0010-terracotta-is-reserved-and-never-alone.md)'s
and is unchanged: a signal carried by hue alone is not a signal. Around one man
in twelve cannot rely on hue, and EN 301 549 has required otherwise for twenty
years.

## The test

**On real displays, not in a rendering and not on a swatch.** A swatch is two
rectangles side by side at a size nothing in the interface uses; the question is
whether a person crossing a room can tell who is acting.

Both roles are shown at **each of the three sizes they really appear at**:

| | Where it appears |
|---|---|
| **Dock size** | the badge on an application icon |
| **Window-control size** | the controls on a window's title area |
| **Selected-object size** | the boundary and handles around an object on the canvas |

and in **each of these conditions**:

- **light theme** and **dark theme**;
- **grayscale**, which is the honest version of the colour-blindness question —
  if the two roles survive with all hue removed, they survive for everybody.

## What counts as passing

**A person can say who is acting without naming the colour.** That is the whole
criterion, and it is deliberately not *the two colours are distinguishable*: the
interface may pass this test while the two blues look identical, because the
mark, the shape and the word are what carry it.

A test where the only thing separating the two roles is which blue they are has
**failed**, even if the observer got the answer right.

## If it does not pass

In this order, and no other:

1. **Adjust the lightness** of the two roles' UI treatments, or of the surfaces
   behind them, until they separate. The brand anchors — deep teal `#0F6B72` and
   navy `#102A43` — stay exactly as they are.
2. **Strengthen the mark, the boundary or the label**, which is the half the
   criterion above actually rests on.

Two things are **not** available. The alo mark is never removed to tidy a
crowded control. And alo's colour is never changed on the evidence of a palette
swatch — only a test at these sizes, in these conditions, can say anything about
it.

## The result

Filled in when the test has been run, by the person who ran it, with the machine
and the display named. **Blank is the honest state until then**, and a row here
is the only thing that makes the claim *you can tell who is acting* a
measurement rather than an intention.

| Size | Theme | Observer could say who is acting | Without naming the colour | Machine, display, date |
|---|---|---|---|---|
| dock | light | | | |
| dock | dark | | | |
| dock | grayscale | | | |
| window control | light | | | |
| window control | dark | | | |
| window control | grayscale | | | |
| selected object | light | | | |
| selected object | dark | | | |
| selected object | grayscale | | | |
