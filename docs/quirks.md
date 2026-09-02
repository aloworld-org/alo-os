# Quirks

Where reality and the specification disagree.

An operating system meets three kinds of reality that no document describes
correctly: hardware and firmware, applications being driven through their own
automation, and pinned upstream engines behaving unlike their manuals. When you
lose an afternoon to one of them, write it down here. The next person should
inherit the knowledge, not the debugging session.

## How to write an entry

One entry per quirk, newest first within each section. Every entry says: **what
it is, what version, what actually happens, what we do about it, and the date.**
A quirk with no version and no date is a rumour.

Keep the accommodation and the reason together. Six months from now the code
will look wrong to somebody, and this file is the only thing standing between
them and reintroducing the bug.

The rule this file serves: **strict in what we do, tolerant in what we accept.**
We behave correctly; we cope with hardware and applications that do not.

---

## Hardware and firmware

_(no entries yet)_

<!--
### <Machine or component> — <one-line summary>
**Version:** firmware / kernel / driver version the behaviour was seen on
**Behaviour:** what actually happens, as opposed to what is documented
**Our response:** what we do, and why this rather than something else
**Date:** YYYY-MM-DD, and who saw it
-->

## Application automation

Applications driven through adapters (`docs/contracts/app-adapters.md`) change
their automation surfaces between versions, sometimes silently. This is where
that gets recorded: which version, what changed, and what the adapter now does.

_(no entries yet)_

<!--
### <Application> <version> — <one-line summary>
**Mechanism:** api | accessibility | dbus | synthetic
**Behaviour:** what the API or the accessibility tree actually does
**Our response:** what the adapter does about it
**Date:** YYYY-MM-DD
-->

## Pinned engines

The kernel, Mesa, systemd, the model runtime and the fine-tuning stack are
configured, never patched. When one of them behaves unlike its documentation,
the accommodation lives in our configuration and the reason lives here.

An entry here that says "we patched it" is a bug in the process: a source patch
to an engine requires an ADR first.

_(no entries yet)_

<!--
### <Engine> <version> — <one-line summary>
**Behaviour:** what it does, versus what is documented
**Our response:** the configuration we apply, and why
**Upstream:** issue link if reported
**Date:** YYYY-MM-DD
-->

## Models

Open-weight models in the catalogue have their own personalities: refusing
formats they claim to emit, ignoring stated context limits, or answering in the
wrong language. Where a model in the catalogue misbehaves in a way that affects
the agents, record it here with the exact model and quantisation — "it was fine
for me" is usually a different quantisation.

_(no entries yet)_
