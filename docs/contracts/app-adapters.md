# Contract — application adapters

**Status:** contract, and the one outside developers build against. Additive
changes only; a break requires versioning and a deprecation period.

An adapter turns an installed application into an agent. Blender, DaVinci
Resolve, GIMP, Inkscape, LibreOffice — each becomes `@blender`, `@resolve`,
`@gimp`, sitting alongside the workspace's own agents and obeying the same rules
(`docs/decisions/0001-the-capability-model.md`).

We will not write every adapter, and we should not try. This document is the
contract so that other people can.

## The one rule that matters most

**The adapter exposes typed verbs and generates any script internally, from
validated arguments. The model never authors code that executes.**

Most applications are scripted — Blender and Resolve take Python, GIMP takes
Script-Fu, LibreOffice takes UNO calls. It is therefore very tempting to expose
a single verb that takes a script and runs it. That verb would satisfy the
letter of "no arbitrary command" while destroying its purpose: an agent that can
write a script can do anything the person can do, in one approval, leaving a
record that says nothing useful.

So: the model chooses `resize(width, height)`. The adapter builds whatever the
application needs, from those two validated integers. An adapter that accepts
model-written code will not be accepted.

## The four mechanisms

Pick the best one the application supports.

| Mechanism | Use when | Quality |
|---|---|---|
| **The application's automation API** | It has one | Best — semantic, reliable, verifiable |
| **The accessibility tree (AT-SPI)** | No API exists | Good — a real widget tree, controls activated properly |
| **D-Bus** | The application exposes an interface | Good where present |
| **Screenshot and synthetic input** | Nothing else exists | Poor — fragile and **unauditable** |

The accessibility tree is the universal floor: an application with no adapter at
all is still readable and operable through it, so every application gets *some*
capability and adapters add *real* capability.

Screenshot-and-click is a last resort. It must be declared in the adapter
manifest, is marked in every record it produces, and is disabled by policy by
default — because after the fact nobody can say what it actually did, which
breaks the record guarantee the whole model rests on.

## What an adapter declares

```
adapter:
  application:  how the application is identified on the machine
  mechanism:    api | accessibility | dbus | synthetic
  verbs:        the list below
  grants:       what must be granted for any of it to be reachable
```

And each verb, exactly as in `agent-verbs.md`: `name`, `purpose`, `effect`
(`read` or `change`), typed `args` with purposes, `requires`, and how its
approval `sentence` is generated from the validated arguments.

## Rules for adapter authors

1. **Be honest about `effect`.** Anything that modifies a document, a file or
   application state is a `change` and waits for approval. "It only nudges the
   layer" is a change.
2. **Type the arguments narrowly.** `width: u32` beats `value: String`. Narrow
   types are most of the validation.
3. **Never take a script, a command, an expression or a "raw" field.** If a
   capability seems to need one, the verb is too broad — split it.
4. **Generate the sentence from the arguments.** If you cannot describe the
   action from its typed arguments, a person cannot meaningfully approve it.
5. **Fail loudly and specifically.** "The document has no layer named Sky" is a
   good failure. "Error" ends a turn for nothing.
6. **Do not require the application to be visible or focused** unless the verb
   genuinely does. Stealing focus mid-turn is hostile.
7. **Test the refusal paths.** An adapter that has only been tested when it
   works has not been tested.

## Versioning

An adapter declares the application versions it supports. A verb's name and
meaning are stable once published: adding verbs and adding optional arguments is
additive and always allowed; removing a verb, renaming one, changing its
`effect`, or making an optional argument required is a break, and needs a
version and a deprecation period.

Where an application's own API changes underneath, that belongs in
`docs/quirks.md` with the version, the behaviour and the date — the next author
should inherit the knowledge, not the debugging session.

## Distribution and trust

An adapter is code that runs on a person's machine and drives their
applications, so it is treated as such: adapters are signed, they declare their
mechanism and grants up front, and policy can restrict which adapters may load
on a machine or across a fleet. An unsigned adapter can be permitted explicitly
for development and never by default.
