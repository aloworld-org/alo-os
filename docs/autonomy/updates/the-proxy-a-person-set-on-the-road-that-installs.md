# The proxy a person set, on the road that installs

**Date:** 2026-09-20
**Workstream:** v0.5 — software and the web, task 14
**Task:** 14, *The proxy a person set, read from the machine's file by the roads
that install.* **Done.**
**Contributor:** the Mac lane — Claude Code on an Apple M3, checkout `~/dev/alo-os`
**Machine:** Apple M3 with 8 GB unified memory; built, linted and tested in the
Lima VM on that Mac — Ubuntu 24.04.4 aarch64. Nothing is ticked *on the
machine*.
**Egress:** `git fetch` and `git push` against `github.com/aloworld-org/alo-os`.
Nothing else.

## The bug, with a person on the other end

A person on a company network opens Settings and sets a proxy. Task 4 decided
the setting, task 11 carried it to a turn's question, task 13 gave it a password
they can set — and the two roads that install **took whichever proxy their
caller happened to pass them.** `TheRentedTool::taking` is handed an
`alo_proxy::Carried`, and nothing in the workspace built one from
`/etc/alo-proxy/proxy.json`.

So that machine's applications installed straight out around the proxy — or did
not install at all, which is the same bug read from the other end.

## What is here

`crates/alo-software/src/road.rs`. It reads the machine's own file through
**`alo_networks::proxy_file::kept_on_this_machine`** at
**`alo_networks::proxy_file::THE_MACHINES_PROXY`** — the same one reader and the
same one path `alo-looking-once` uses, and no second spelling of either, because
a second reader of a machine-wide file is a second answer to *what proxy is this
machine on*.

It contains no branch on the setting's shape, no exception list and no loopback
check: `alo_proxy::the_way` decides and this asks. `alo_proxy::signed_in` is the
one door the credential travels through, with the password the machine gave the
unit that carries the errand out (ADR 0059).

`the_tool_for(road, host)` hands back a `TheRentedTool` already carrying the
road. That is what closes the gap — a caller asks **by errand** rather than by
proxy, and so cannot pass the wrong one.

**One small thing was owed to make that callable.** `Source` read the host out
of a place's address, built a `Destination` from it and dropped it, and
`alo_egress::Destination` gives no host back — so nothing could have supplied
`the_tool_for` with one. `Source::host()` now keeps it, read **once** in
`Source::of` and used for both, so a place cannot be shown on the indicator as
one host and reached through a proxy decided about another.

## The three states a machine can be in, each measured

| The machine's file | What happens |
|---|---|
| names a proxy | taken, on all three of this crate's roads |
| is not there at all | **straight out** — an ordinary machine on an ordinary network |
| is there and is not one this machine wrote | **refused**, naming where the file is |

The third is the one that matters. A machine that read an unreadable rule as
*no rule* would send a company's traffic around the company's own proxy with
nobody told. Four shapes are measured — empty, valid JSON that is not this file,
a proxy spelled another way, and bytes that are not text — and every one is a
refusal rather than a quiet `TheProxy::None`.

## The half that earns the task

A test showing the proxy is *read* is the easy half, and it is also the half
that a check doing nothing would pass. So:

- **An errand on a machine whose file names a proxy produces a tool whose
  started program really receives it** — a real child process, its environment
  read back, for `InstallingAnApplication` and `UpdatingAnApplication`. A road
  decided correctly and never reaching the tool would pass every other test here
  and still install straight out.
- **Including the credential.** For a proxy that asks who this machine is, the
  started program receives `http_proxy=http://anna:hunter2@proxy.example.com:8080`
  — and nothing of this process's own environment, asserted by the absence of
  `HOME=`.
- **A proxy that is set and cannot be used refuses.** An automatic
  configuration on a machine with nothing that can evaluate one is
  `NoRoad::NotDecided` on every road, never a straight one. There is no branch
  in `road_out` that could do otherwise — it returns the road or an error and
  has no third answer — and that test is what would fail the day somebody added
  one.
- **The indicator still names where the errand is really going.** On a company
  network every errand goes *through* the proxy, so an indicator naming the
  proxy would say the same thing for every errand on the machine and tell
  nobody anything — and it would be a person's own destination replaced by
  their employer's equipment. The test reads the line and asserts the
  destination is in it and the proxy is not.

## One thing I could not do, said plainly

The acceptance asks for the credential *held by a test that reads the first line
on the wire*. On this road there is no wire this repository can read: the
credential is delivered to a **rented program** through its environment, and the
`CONNECT` line is that program's to send. The wire tests that do exist —
`alo-agentd`'s and `alo-proxy`'s — drive an HTTP client this crate does not
have, and the rented tool is on no machine these tests run on.

What is measurable here is that the password really reaches the program that
would send it, out of a real child process, and that is what is tested. The
literal first line on the wire for this road needs the rented tool present,
which is an *on a machine* measurement and is not ticked.

## What this leaves owed

Named the way the network task named this one:

- **`alo-updating` and `alo-models` are still handed a `TheProxy` by their
  callers.** They are not this plan's crates and were not touched. The same
  shape fits both: read through `kept_on_this_machine`, decide with `the_way`,
  sign in with `signed_in`, refuse rather than fall back.
- `docs/contracts/machine-proxy-file.md` said *no road out reads this file yet*.
  That had been **stale since `alo-looking-once` landed** and was not corrected
  then; it now names the two roads that do read it and the two that still do
  not.

## A note on the unit named

`road.rs` writes `THE_UNIT = "alo-agentd.service"`, because that is the unit
that carries out the software verbs today. ADR 0059 wants exactly this constant
to exist — *the constant in the crate is the list of unit files that must carry
the line* — and `alo-software` is a library rather than a program, so the unit
it names is the one that runs its roads. A second unit that ever installs an
application has to carry the credential line too, and would say so there.

## Gates

Nine in the Lima VM. `alo-software` is one of the crates that does not build on
macOS — not from its own manifest, but transitively through `alo-sessiond`'s
`socket_peercred`, which `rustix` gates to `linux_kernel` — so everything here
was compiled and tested in the VM, which is where gating happens anyway.

## Crates touched

`crates/alo-software` only — the new `src/road.rs`, plus `src/source.rs`,
`src/lib.rs` and `Cargo.toml` — and `docs/contracts/machine-proxy-file.md`.
Nothing in `alo-updating`, `alo-models` or `alo-shell`; no road was re-decided;
nothing reads the machine's description.
