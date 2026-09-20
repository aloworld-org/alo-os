# The camera is not a candidate, so no permission can choose it

**Date:** 2026-09-20
**Workstream:** v0.5 — devices and media, task 4
**Task:** 4, *The camera and the microphone.* Still blocked. This narrows what
it is blocked on and offers a cheap way to decide between two explanations.
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** measured in the Lima VM that gates this repository — Ubuntu 24.04.4
aarch64, PipeWire 1.0.5, WirePlumber 0.4.17, `vivid`. Nothing is ticked *on the
machine*.
**Egress:** none. Nothing was fetched; every tool used was already installed.

## What I did not find

**Another lane found the undeclared-kind fault first**, nine hours before me, on
a different machine and through a different road — the screenshot path in
`alo-capturing` — and landed it as **#61**. I found the same fault
independently, in a session started before that landed, and I am not claiming
it. What follows separates the part that corroborates from the part that is new,
because a repository where two machines quietly claim the same finding is a
repository whose records stop being worth reading.

## What corroborates, and why it is still worth having

The fault spans **two WirePlumber major versions through two different code
paths.**

#61 measured it on 0.5.13: a client that names no `media.class` reaches the
session manager as `Stream/Input/Unknown`, and `prepare-link.lua` refuses it.

On 0.4.17 there is no `prepare-link.lua`. The refusal happens in
`policy-node.lua`'s `canLink()`, on its very first test:

```lua
if properties["media.type"] ~= target_properties["media.type"] then
  return false
end
```

`media.type` is derived from `media.class`, so an undeclared client's is `nil`,
and `nil` equals nothing — every candidate in the graph is rejected. The
instrumented policy, printing what it compares:

```
PROBE findDefinedTarget consumer.media.type=nil want.direction=output
PROBE   linkable name=synthetic-camera dir=output mtype=Video canLink=false
```

Same fault, same symptom, different mechanism, different major version. With
`media.class=Stream/Input/Video` declared, `gst-launch-1.0 pipewiresrc` attached
to a synthetic `Video/Source` and captured **ten buffers, EOS in 0.7 s**.

## What is new: the camera is not a linkable

#61 established that with both faults fixed the camera is still refused, and
reopened the **portal** as the explanation — from a reading of
`client/access-portal.lua`, recorded honestly as a reading.

Here is a measurement that constrains that.

WirePlumber's policy chooses a target only from `linkables_om`:
`findDefinedTarget`, `findDefaultLinkable` and `findBestLinkable` all iterate or
look up in it. Printing every member of it at the moment of a failed camera
attach gives the six loopback audio nodes, the synthetic `Video/Source`, and the
client itself.

**The camera is not there.**

It is not there because WirePlumber never makes a session item for it.
Instrumenting `create-item.lua`'s `addItem` shows it called for every audio node
and for the synthetic `Video/Source` — and **never** for
`v4l2_input.platform-vivid.0`, in a run where that node is present and complete
in the graph, with a full `EnumFormat`.

**That is upstream of permissions.** A portal grants permission on a node; the
policy never considers this node at all. On this stack no permission-store entry
could produce a link, because nothing is choosing among candidates that include
the camera.

## The mechanism, recorded as a reading and not as a measurement

The one anomaly in that node's whole trace is a parameter enumeration that fails
where it succeeds on every audio node:

```
enum_params_for_cache_done: <WpNode:49> enum params failed:
    enum params id:2 (Spa:Enum:ParamId:Props) failed
```

| `pw-cli enum-params <node> Props` | Result |
|---|---|
| the camera (`v4l2_input.platform-vivid.0`) | **nothing at all** |
| an audio source (control) | a full `Props` object — volume, mute |

It is the same failure WirePlumber 0.5.17 reported *fatally* on 2026-09-19, and
which 0.4.17 logs at debug and continues past.

**That this failure is why the node never reaches `create-item.lua` is not
measured**, and one observation argues against the simplest version of it: an
object manager with the identical interest, run from `wpexec` in a separate
process against the settled graph, **does** see the node. So whatever excludes
it lives in the daemon's own handling rather than in a property of the global. I
am recording the correlation and stopping there rather than writing down the
tidy story, because the tidy story is exactly what the next section is about.

## The cheap experiment that decides it

Before anybody arranges a machine with a desktop session and a real camera:

**Print `linkables_om`'s members on 0.5.13 at the moment of a failed camera
attach.**

- If the camera is **absent** there too, the portal cannot be the explanation on
  that stack either, and the question becomes why a V4L2 node never becomes a
  linkable.
- If it is **present**, the portal reading stands and what I measured is a
  0.4-only quirk.

The instrumentation is a copy of `create-item.lua` and `policy-node.lua` into
`/etc/wireplumber/scripts/`, which WirePlumber prefers over the installed ones.
Nothing installed is patched — that is configuring a rented engine (ADR 0011) —
and deleting the copies restores it exactly.

## A control that was measuring the wrong thing

`docs/quirks.md` recorded *`vivid` differs from a real camera — **eliminated***,
concluded from a synthetic video source being refused in exactly the same way.
Sound reasoning from what was known: if a source with no camera in it is refused
identically, the camera cannot be the variable.

**That control was itself failing for the undeclared-kind fault.** Both
experiments were rejected before `vivid` was reached, so they agreed for a
reason unrelated to what was being tested. Remove that fault and they stop
agreeing: the synthetic source links, the camera does not.

It was not a wrong measurement. It was a **correct measurement of the wrong
thing**, which is harder to notice, and it was believed for a day. A control
experiment is only as good as the thing it holds constant, and this one held
constant a fault nobody knew was there.

## What changes in the code: nothing

`alo-cameras` reads the graph rather than opening streams; `alo-in-use` already
keys an in-use video stream on `Stream/Input/Video`; #61 declared the kind where
a capture is opened. Everything this measurement depends on was already written
that way.

## Crates touched

None. `docs/quirks.md` and the plan.

## One finding for whoever owns the gate machine

`v4l2loopback-dkms` has been in dpkg state `iF` in the Lima VM since 2026-09-17:
its 0.12.7 source calls `v4l2_fh_del(fh)` with one argument where kernel
7.0.0-31's header declares two. Any `apt-get install` there re-triggers it and
reports `dpkg returned an error code (1)` **after successfully installing what
was asked for**. Nothing depends on it — the camera fixture is `vivid` — but it
will read like a failed install to whoever meets it next.
