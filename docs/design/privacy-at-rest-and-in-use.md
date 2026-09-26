# Privacy at rest, and privacy in use

The owner decided on 2026-09-26 that **privacy is not always on screen**. This
says exactly what that changes and what it does not, because the thing being
hidden and the promise underneath it are not the same object and it would be
easy to hide both.

## The three states, and what is drawn in each

| | What is happening | What is on screen |
|---|---|---|
| **At rest** | nothing is leaving, no camera, no microphone | **nothing** |
| **In use** | something is leaving, or the camera or microphone is on | the indicator, lit, with a readable line saying who, where to and why |
| **Cannot be drawn** | the indicator itself failed | a sentence saying so |

**At rest is the change.** The living canvas carries a persistent chip reading
*Private*, labelled in the file as *always visible*. That chip goes. Nothing in
this repository ever required it: the
[egress indicator's own record](../autonomy/updates/the-egress-indicator-on-a-screen.md)
already says that while a question is answered on the machine itself **the light
is dark and there is nothing under it**. A chip that says *Private* all day is a
reassurance rather than a signal, and a reassurance shown constantly is the
thing people stop reading — which is the state they are in when it changes.

**In use is untouched, and is the promise.** alo OS is sold on *nothing leaves
your machine silently*, and that is not a decoration that can be tidied away for
a calmer canvas. The moment anything goes to the network, or a camera or a
microphone is live, it is on screen and it says who, where to and why, in the
person's own language. Full screen does not exempt it. No setting turns it off.

**Cannot be drawn is the one that hiding endangers**, and it is why this
document exists rather than a line in a design file. The record's own words: *a
machine that quietly failed to draw the indicator would look exactly like a
machine on which nothing was leaving, which is the one failure this whole
promise does not survive.* Once *at rest* also draws nothing, **a failed
indicator and a quiet machine look identical**, and the difference can no longer
be seen — only reported. So the sentence the record already requires stops being
a nicety and becomes the only thing standing between a broken indicator and a
silent one.

What follows from that, and is a test rather than an intention:

- The shell knows the difference between *nothing to show* and *could not show
  it*, and never lets the second render as the first.
- A machine that cannot draw the indicator says so **where a person is looking**,
  not only in a log.

## What must change in the design

- The status area's persistent **Privacy · always visible** frame is removed, and
  with it the *Private* label at rest.
- The lit state is designed for each surface it can appear on, **including full
  screen**, where there is no status area to put it in.
- The cannot-be-drawn sentence is designed. It has never been drawn anywhere.

## What this does not decide

Whether a person may *opt in* to seeing a resting indicator. Somebody who wants
the reassurance is not asking for anything unreasonable, and Law 5 says the
choice is theirs — but a default is not a prohibition, and nobody has asked for
it yet.
