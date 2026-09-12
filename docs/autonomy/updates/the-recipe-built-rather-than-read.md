# A machine that was watched, rather than a recipe that was read

**Date:** 2026-09-11
**Workstream:** v0.01 delivery plan, task 33
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this was

Every image task since 27 ended its report with the same sentence: **nothing in
this lane has ever built this image.** `crates/alo-image` reads `image/`'s
declarations and holds them to the promises in `docs/` — four units, five
logins, two directories, a pinned runtime, pinned weights, a disk declaration
and a document — and every one of those is a *declaration*. Three things in that
recipe cannot be checked by reading it: it fetches 2.23 GiB of weights and holds
them to a digest, it runs the pinned runtime **inside a build stage** to import
them, and it asserts five login numbers against a base that allocates downward.
The last of those already went wrong once — `systemd-sysusers` quietly put alo
OS's agent into the resolver's group, and it was found by building.

So the task was to build it and say honestly what happened.

## The build

    podman build -f image/Containerfile -t alo-os:dev .

which is the command `docs/booting.md` gives, run from the root of this
checkout. `docker` is what the Containerfile's own header names and podman is
what the booting document names; Docker Desktop's engine was not running on this
machine and podman was, and ADR 0011 cares which *base* is rented rather than
which OCI builder assembles it. Podman 5.7.0, Ubuntu 26.04 under WSL2, kernel
`6.18.33.2-microsoft-standard-WSL2`, 4 cores, 5.8 GiB of memory.

Nothing of ours was cached: all four stages ran from their first `RUN`. The
Ubuntu builder base was already in the store; `quay.io/fedora/fedora-bootc:42`
was pulled during this build.

**It succeeded.** Exit 0, `Successfully tagged localhost/alo-os:dev`.

| | |
|---|---|
| Wall clock | **27:29** (810 s user, 454 s system, 76 % CPU) |
| Peak resident set of the builder | 822 MB |
| The image | **9,000,704,537 bytes** — 8.38 GiB |
| Container storage, before and after | 2.12 GB → **26.14 GB** (all four stages kept) |
| The filesystem it is on | 90 GiB → 113 GiB used: **≈23 GiB of disk for one build** |
| Fetched | the runtime's tarball, **2,393,231,072 bytes** of weights, the Fedora base, ~250 MB of Ubuntu packages, two Rust toolchains and `bpf-linker`'s tree |

### The build's own assertions, which is what it was built to find out

**The five numbers were all created as asked, with no fallback.** This is the
one the plan was most worried about, and the log says it plainly:

    Creating group 'alo-agent' with GID 60989.
    Creating group 'alo-greeter' with GID 60990.
    Creating group 'alo-model' with GID 60991.
    Creating user 'alo-agent' (alo OS agent) with UID 60989 and GID 60989.
    Creating group 'alo' with GID 1000.
    Creating user 'alo' (alo OS) with UID 1000 and GID 1000.
    Creating user 'alo-greeter' (alo OS sign-in) with UID 60990 and GID 60990.
    Creating user 'alo-model' (alo OS model service) with UID 60991 and GID 60991.

No `Suggested user ID … already used` line anywhere — so the move to 60989+,
made after the first build of this image put the agent in `systemd-resolve`'s
group, **holds on this base**. The seven `test` lines and the membership check
after it all passed.

**`bootc container lint`: `Checks passed: 13`, `Checks skipped: 1`.** bootc
1.15.1 has fourteen lints and will not say which one it skipped: there is no
verbosity option, `--list` prints the set and the summary prints two counts. A
gate that will not name what it did not check is worth knowing about before
somebody quotes the green line; it is recorded here rather than worked around.

**The model store exists**: `test -d /models` passed in the weights stage, and
the store is on the image at `/usr/share/alo/models`.

### What the built image says when it is asked

Read out of the image afterwards — inspection of a container, **not a boot**:

- `id -u` for `alo`, `alo-agent`, `alo-greeter`, `alo-model` → 1000, 60989,
  60990, 60991; `id -nG alo` → `alo alo-agent`.
- All four units `enabled`; `systemctl enable` made the symlinks, including
  `/etc/systemd/system/user@1000.service.wants/alo-agentd.service`.
- The three binaries of ours and the runtime's are 0755 root:root.
- `/usr/share/alo/models` — and this is where the honest reporting starts.

## What went wrong, and it is two things

### 1. The image carries the weights **twice**, and one copy is referenced by nothing

The recipe's own comment says *2.23 GiB of weights, carried once*. The store on
the built image is **4.5 GiB**:

    -rw-r--r-- 1 root root 2393231072 sha256-01ec9e677f56…
    -rw-r--r-- 1 root root         94 sha256-79bcc381cb85…
    -rw-r--r-- 1 root root        149 sha256-8493c527151c…
    -rw-r--r-- 1 root root 2393231072 sha256-8a83c7fb9049…
    -rw-r--r-- 1 root root         98 sha256-901ce0250d34…

`8a83c7fb…` is **the pinned GGUF** — byte for byte the artefact
`THE_MODELS_SHA256` names and the recipe verifies before anything reads it.
`01ec9e67…` is what `ollama create` wrote out of it, the same length and a
different digest. The manifest names `01ec9e67…` as its model layer and
**names `8a83c7fb…` nowhere**. So every machine built from this recipe carries
2.23 GiB of weights that nothing on it will ever open.

It is not recoverable on the machine either: `/usr` is read-only on a bootc
system, which is the whole of ADR 0011's model, so the runtime could not prune
it even if it wanted to. The cost is an image 2.23 GiB larger than the promise
it was written to keep, a longer install, and a bigger disk than
`docs/booting.md`'s `truncate -s 20G` was sized against (still comfortable — but
sized against a number nobody had measured).

**No decision was changed to make this build pass, and none is changed here.**
It is written into `docs/quirks.md` as an engine behaving unlike its
documentation, and into the plan as **task 34**, which is where somebody decides
between importing differently, dropping the source blob in the weights stage, or
carrying it deliberately.

### 2. The runtime phones home at start — measured, where it used to be reasoned

Task 32 wrote `IPAddressDeny=any` into `alo-modeld.service` on the argument that
*a runtime that fetches a model is an egress an agent caused, and one that phones
home on start is an egress nobody asked for*. That was an argument. Started out
of the image this build produced, as `alo-model`, with **no network at all**, the
pinned runtime says this in its first eight milliseconds:

    WARN model_show_cache.go:142 "model show cloud cache hydration failed"
      error="Get \"https://ollama.com:443/api/tags?ts=…\": dial tcp: lookup ollama.com: …"
    WARN model_recommendations.go:168 "model recommendations refresh failed"
      error="Get \"https://ollama.com/api/experimental/model-recommendations?ts=…\": …"
    INFO model_recommendations.go:177 "model recommendations cache sleep scheduled"
      wait=4m37s consecutive_failures=1

Two outbound HTTPS requests to a named host, before anybody has asked the machine
anything, and a retry scheduled for as long as the machine is up: every ~4m37s
while they fail. Its defaults say the same thing from the other side:
`OLLAMA_REMOTES:[ollama.com]`, `OLLAMA_NO_CLOUD:false`.

**And I caused one of those requests to succeed, which is worth saying out
loud.** The first time the runtime was started by hand out of this image it had
an ordinary network, and its scheduler line came back
`wait=3h39m19s consecutive_failures=0` — a zero after a start is a request that
was answered, so **this machine reached `ollama.com` because of an inspection I
ran**, not because of anything the recipe does. Task 33's constraint is that it
downloads what the recipe downloads and nothing else; that is where it was broken,
by one `GET` from a rented runtime's own start-up path, and the second run was
done with `--network=none` precisely because the first one showed what starting it
costs. Nothing of anybody's left the machine — the requests carry a timestamp and
ask for a list — and it is still an egress nobody asked for, which is the whole
reason the finding matters.

**Law 1 is fine and the unit is why.** `IPAddressAllow=localhost` under
`IPAddressDeny=any` is a kernel-side filter on that service's own control group,
so on a machine these requests do not fail politely — they do not leave. What
changed today is that the sentence in task 32's report — *the egress claim is a
setting read, not a machine watched* — is now half measured: **the thing the
setting exists to stop has been watched happening.** The setting stopping it has
not: what refused these two requests was `--network=none` on a container, which
is a different mechanism. Putting a counter beside the unit is task 34's other
half.

It also means the runtime has a documented switch of its own —
`OLLAMA_NO_CLOUD` — that this unit does not set. That is configuration rather
than a patch (ADR 0011 allows it) and it belongs in the same task, because a
belt beside the kernel's braces is a decision about defence in depth rather
than a line somebody adds while reporting a build.

### And one that was not wrong, but cost twenty minutes

Inspecting the image as an ordinary container, the runtime refused to start with
`Error: could not create directory mkdir /root: file exists`. That is not a
defect in the image: on an ostree-derived base `/root` is a **symlink to
`var/roothome`**, and `/var/roothome` does not exist until the machine boots, so
a Go `MkdirAll` through the dangling link fails with a sentence that reads like a
permissions bug. On the machine `alo-modeld.service` never goes near it — it runs
as `alo-model` with `HOME=/var/lib/alo-model` and `StateDirectory=alo-model`,
which task 32 got right. Started that way inside the image, as that login, with
that home, the runtime came up and answered:

    NAME                                 ID              SIZE      MODIFIED
    phi3:3.8b-mini-4k-instruct-q4_K_M    df90dab51666    2.4 GB    12 minutes ago

That is the artefact name `THE_MODELS_ARTEFACT` pins and `alo-models`' catalogue
entry names, served off the store the image carries, by the login the unit names.
It is **not** the unit running and it is **not** a boot: no systemd, so none of
the sandboxing, none of the IP filter, and nothing started anything. It is one
process run by hand inside a container. The quirk is in `docs/quirks.md` so the
next person inspecting this image does not spend the same twenty minutes.

## What changed in the repository

### `crates/alo-image` now holds the recipe's assertions to the logins it declares

Building the image is what showed that the seven `test` lines at the foot of the
Containerfile are load-bearing — they are the only thing standing between
`systemd-sysusers` taking a different number and a machine whose description
names a login it does not have — and that **nothing held them to the file they
are about**. A sixth login added to `sysusers.d` would be asserted by nobody,
and both files would be individually correct.

- **`crates/alo-image/src/asserted.rs`** (new) — `Asserted::read` reads what the
  recipe asserts *after* `systemd-sysusers` has run: each login's number, each
  group's number, each membership. Read leniently for `crate::runtime`'s reason;
  comments assert nothing, a line that compares against nothing asserts nothing,
  and a comparison against something that is not a number asserts nothing.
- **`checking.rs`** — `the_build_holds_every_login_to_its_number`, one `Wrong`
  per disagreement, for absence *and* for drift: a number asserted as something
  other than what is declared is the same finding.
- **`wrong.rs`** — three variants, each naming what it is about:
  `ALoginTheBuildDoesNotAssert`, `AGroupTheBuildDoesNotAssert`,
  `AMembershipTheBuildDoesNotAssert`.
- **`image.rs`** — `Image::declares()` and `Image::asserted()`. The lookups that
  were there answer about a name somebody already knows; the login nobody
  remembered to assert is exactly the one nobody would think to ask about.

Three refusal tests break one line of a copy of the real image — a sixth login
nobody asserts, a number the two files stopped agreeing on, and the membership
check deleted, which is the one that really happened in September — and one test
on the real image says every number it declares is one its own build asserts.

## Decisions I made, and why

- **Podman rather than Docker.** Docker Desktop's engine was not running;
  `docs/booting.md` names podman for the disk step and for the build. What is
  pinned by ADR 0011 is the base and its digest, not the builder.
- **The findings are written down, not fixed.** Task 33's constraint is that it
  changes no decision to make a build pass, and the build passed. Halving the
  image and setting `OLLAMA_NO_CLOUD` are both changes to the recipe and the
  unit, both have a cheaper and a more expensive form, and the plan's own rule
  is that a finding becomes the next task. They are task 34.
- **The runtime was started by hand, once, and the report says exactly what that
  was worth.** It answers the question the build could not — *is the store the
  image carries one a runtime can read* — and it answers nothing about systemd,
  the unit, or a boot.
- **The report says `Checks skipped: 1` rather than quoting the passing count
  alone.** Thirteen of fourteen is not fourteen.

## What this does **not** claim

- **Not a boot.** No machine, no virtual machine, no disk was written. The
  machine half of `ROADMAP.md`'s image line stays empty, and **no *On the
  machine* box moves.** `docs/features.md` was not touched.
- **Not *arrives ready to run*.** Nothing started `alo-modeld.service`; the one
  thing that serves the model has still never been started by systemd.
- **Not the egress measurement.** Nothing put a packet counter beside a booted
  image. What was measured is that the runtime *tries*, and that a container with
  no network refuses it.

## Verification

Windows 11 host, checkout `C:\dev\alo-os-claude`; the build in Ubuntu 26.04 under
WSL2.

| Check | Result |
|---|---|
| `podman build -f image/Containerfile -t alo-os:dev .` | exit 0, 27:29, 9,000,704,537 bytes |
| `cargo fmt --all` | clean |
| `cargo clippy -p alo-image --all-targets -- -D warnings` | clean |
| `cargo test -p alo-image` | 159 unit + 23 integration tests, all passing |
| `cargo doc -p alo-image --no-deps` | clean |

The whole-workspace suite is the supervisor's, as the task instructs.

## Proposed changes to shared documents

**`CHANGELOG.md`** — under Unreleased:

> The image recipe was built for the first time since it gained a model runtime,
> pinned weights and a service to serve them: 27 minutes, 8.38 GiB, every login
> number asserted and created as asked. Two findings came out of doing it — the
> weights are carried twice, and the pinned runtime makes two requests to its
> publisher at start — both recorded in `docs/quirks.md`. `crates/alo-image` now
> holds the recipe's own build-time assertions to the logins the image declares,
> so a login added to one file and asserted in neither is caught.

**`ROADMAP.md`** — no change. The image line's machine half stays empty.

**`docs/autonomy/QUEUE.md`** — no change proposed.

**`docs/autonomy/STATE.md`** — reference this report; task 33 done, task 34
written into `docs/autonomy/v0-01-delivery-plan.md`.
