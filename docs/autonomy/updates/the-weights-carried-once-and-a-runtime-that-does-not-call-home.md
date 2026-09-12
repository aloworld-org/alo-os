# The weights carried once, and a runtime that does not call home

**Date:** 2026-09-12
**Workstream:** v0.01 delivery plan, task 34 (*Half an image of dead weight,
and a runtime that calls home*)
**Contributor:** Claude (separate checkout, `C:\dev\alo-os-claude`)
**Status:** ready for integration

## What this was

Task 33 built the image for the first time since it gained a runtime, weights
and a unit to serve them, and measured two things a recipe cannot show by being
read. The store carried the weights **twice** — `ollama create` leaves the
checked GGUF in the store under its own digest and writes a second blob of the
same length that the manifest names instead, so `/usr/share/alo/models` was
4.5 GiB for 2.23 GiB of model, on the read-only half of every machine we ship.
And the pinned runtime **called home**: two HTTPS requests to `ollama.com` in
its first eight milliseconds, retried every five minutes for as long as they
failed. `alo-modeld.service` carried `IPAddressDeny=any` against exactly that,
and nothing in this lane had ever started that unit under systemd, so *the
filter stops it* was a sentence nobody had watched being true.

This task fixes the first, decides the second, and watches the third.

## What changed

### The weights are carried once — `image/Containerfile`

The weights stage now removes the source blob **after** the import, and asserts
three things before the store leaves the stage:

```
/runtime/bin/ollama create "${THE_MODELS_ARTEFACT}" -f /Modelfile; \
rm -f "/models/blobs/sha256-${THE_MODELS_SHA256}"; \
/runtime/bin/ollama show "${THE_MODELS_ARTEFACT}" >/dev/null; \
kill "${served}"; \
test -d /models; \
manifest="$(find /models/manifests -type f)"; \
test "$(printf '%s\n' "${manifest}" | wc -l)" = 1; \
for blob in /models/blobs/*; do \
  grep -q "sha256:${blob##*/sha256-}" "${manifest}" \
    || { echo "${blob} is referenced by nothing"; exit 1; }; \
done
```

The runtime can still `show` the model off what is left — which is the line
that would go red if a future runtime referenced the source blob directly, so
`rm -f` cannot quietly break a store; there is exactly one manifest; and every
blob in the store is named by it. **The digest check is untouched**: it runs on
the fetched file before the import reads it, in a step of its own, exactly as
before. Removing the runtime's copy of that file afterwards changes nothing
about what was checked. The recipe's own comment says *carried once* and now
says how that is held.

**Which way, and what the others cost.** Three ways out were on the table.
Importing another way — writing the blob and manifest into the runtime's store
by hand — is a second implementation of a rented engine's storage format, which
is the drift ADR 0011 exists to refuse; the day the runtime changes its layout
the image would be wrong in a way no test here could see. Carrying it
deliberately is 2.23 GiB on every machine we ship, over every network it is
installed across, for a file nothing will ever open. Removing a file the runtime
wrote is the image's business and changes nothing the runtime does — and the
runtime agrees the blob is unused: **at every start it tries to delete it
itself** (`total unused blobs removed: 1`; on a machine, `couldn't remove file
… permission denied` against `/usr`). Now there is nothing for it to try.

### The recipe and the crate agree with the store, held by a test — `crates/alo-image`

- **`weights.rs`** — `TheWeights` reads two more lines off the weights stage,
  after the import line: `drops_the_source()` (an `rm` naming the blob by the
  pinned digest's own argument) and `holds_the_store_to_its_manifest()` (the
  walk over every blob, the lookup in the manifest, and the refusal). Read
  leniently for the same reason as everything else in this crate; a removal
  *before* the import does not count, because there is nothing to remove yet.
- **`wrong.rs`** — `TheWeightsAreCarriedTwice` and `TheStoreIsHeldToNothing`,
  each naming the decision it breaks. Two rather than one because they are two
  lines that go wrong separately: the removal catches today's second copy, and
  only the walk catches the one a runtime update leaves under a name nobody
  wrote down.
- **`checking.rs`** — both checked beside the pin and the digest, and two
  refusal tests that break a copy of the real recipe: the removal turned into
  `true`, and the walk pointed at `/nowhere/*`. Each asserts the *other* finding
  is absent, so the two are shown to be independent.
- **`tests/what_the_image_owes_the_daemons.rs`** —
  `the_store_the_image_carries_is_the_weights_once` reads both lines off the
  real recipe and asserts the digest check is still there, since dropping the
  source is only cheaper if the check before it stayed.

### The runtime's own switch is decided: set — `alo-modeld.service`

`Environment=OLLAMA_NO_CLOUD=1`, as a second lock beside the filter rather than
instead of it. The argument in one sentence: the filter is what makes *nothing
leaves* true and cannot be switched off by a runtime update, and the switch is
what keeps the requests from being made at all, so a machine's journal carries
no line naming the publisher and the runtime spends no cycle retrying a request
that will never leave. It is the engine's own documented setting, so it is
configuration and not a patch (ADR 0011).

Measured, not assumed. Under the image's own systemd (below), the same unit with
the switch unset:

```
level=INFO msg="Ollama cloud disabled: false"
level=WARN msg="model show cloud cache hydration failed" error="Get \"https://ollama.com:443/api/tags?ts=…\": …"
level=WARN msg="model recommendations refresh failed" error="Get \"https://ollama.com/api/experimental/model-recommendations?ts=…\": …"
level=INFO msg="model recommendations cache sleep scheduled" wait=5m46s consecutive_failures=1
```

and with it set:

```
level=INFO msg="Ollama cloud disabled: true"
level=INFO msg="model recommendations cache sleep scheduled" wait=3h47m consecutive_failures=0
```

Neither request is made, no `WARN` is written, and the only line still naming
the publisher is the runtime's own dump of its defaults (`OLLAMA_REMOTES`).
`crates/alo-image` checks it: `TheServerStillAsksItsPublisher` for a unit that
dropped the line or set it to anything but `1`, with the refusal test trying
both, and `the_model_service_does_not_ask_its_publisher` on the real unit.

### The filter watched, rather than read

**How the unit was started.** `podman run --systemd=always … localhost/alo-os:dev
/sbin/init` boots the image's own systemd inside a container — its own units,
its own store, its own logins, its own `systemd-resolved`. Not privileged. The
first attempt failed before the service ran:

```
alo-modeld.service: Failed to keep CAP_SYS_ADMIN: Operation not permitted
alo-modeld.service: Failed at step USER spawning /usr/bin/ollama: Operation not permitted
```

That is not the unit asking for anything: it is systemd's own bookkeeping around
a `User=` switch, which needs the *init* to hold `CAP_SYS_ADMIN` for a moment,
and podman's default container init does not. The plan's constraint was to say
so rather than run it as root, and this is saying so. The container's init was
given `--cap-add=SYS_ADMIN,BPF,NET_ADMIN` — the first for the user switch, the
other two so `IPAddressDeny=` can attach its BPF programme to the unit's control
group. On a machine, PID 1 holds every capability, so this is a strict subset of
what a booted image's init already has. **The unit was not changed**, and its
process came up exactly as the unit says:

```
User=alo-model   Uid: 60991   CapEff: 0000000000000000   CapBnd: 0000000000000000
CapAmb: 0000000000000000   NoNewPrivs: 1
IPAddressAllow=127.0.0.0/8 ::1/128   IPAddressDeny=::/0 0.0.0.0/0
```

Two more things to make the container the shape a machine is, both in
`docs/quirks.md`: podman writes the host's resolver into `/etc/resolv.conf`,
so the container was started with `--dns=127.0.0.53` and `resolved` was given
an upstream, which is how a machine does it — the filtered process asks the
stub on loopback, and `resolved` asks upstream from its own control group. And
glibc's `nss-resolve` reaches `resolved` over a Unix socket, so name resolution
from the unit's login produces no IP packet at all.

**What was watched.** Two `nftables` counters, one on each side of the
boundary. Inside the container, on the output hook, packets from uid 60991 to
port 443 — the SYNs the runtime's two requests generate, and the kernel's
retries. On the host, on the forward hook, packets from the container's address
to port 443 — what actually left. The unit was restarted with a working network
and the counters read after twelve seconds:

| | attempted by `alo-model`, inside | left the container, at the host |
|---|---|---|
| before | 0 | 0 |
| after the unit started | **16 packets, 960 bytes** | **0** |
| after a control `curl https://quay.io/` from an unfiltered process in the same container | 16 | **19 packets, 1836 bytes** — answered `200` |

The requests were made, the network was there, and nothing left. That is
`IPAddressDeny=any` refusing, rather than an absent network. What the runtime
logs in that state is:

```
error="Get \"https://ollama.com:443/api/tags?ts=…\": context deadline exceeded"
```

— not `connection refused` and not `no such host`, because a cgroup egress
filter drops the packet after the socket has sent it, so the connect times out
rather than fails. That reading is in `docs/quirks.md` so nobody takes it for a
slow network. The host counter table was added for the measurement and deleted
after it; nothing on the host was left changed.

**What this is not.** It is the image's own systemd starting the image's own
unit, in a container. It is **not a boot**: no firmware, no disk, no `bootc
install`, and a container's init with three capabilities is not a machine's
init with all of them. Enough to watch a filter refuse a request; not enough to
move anything under *On the machine*.

### The rebuild

```
podman build -f image/Containerfile -t alo-os:dev .
```

from the root of this checkout, podman 5.7.0, Ubuntu 26.04 under WSL2. The
builder's toolchain layers, the runtime fetch and the weights fetch were cached
from task 33's build; the Rust build, the import and the whole machine stage
ran.

| | task 33 | now |
|---|---|---|
| Wall clock | 27:29 | **7:48** |
| The image | 9,000,704,537 bytes (8.38 GiB) | **6,607,474,716 bytes (6.15 GiB)** |
| Difference | | **−2,393,229,821 bytes**, which is the source blob (2,393,231,072) less what the manifest's rounding hides |
| `/usr/share/alo/models` | 4.5 GiB, five blobs, one unreferenced | **2.3 GiB, four blobs, all named by the manifest** |
| Container storage | 26.14 GB | 34.23 GB (task 33's image kept under a second tag for the comparison) |

The build's own assertions passed: the removal, `ollama show` off the store
that is left, one manifest, every blob named, and the five login numbers as
before.

**And the rebuilt image answers.** Started under its own systemd with
`--network=none`, the unit as shipped logged `Ollama cloud disabled: true` and
`total unused blobs removed: 0`, and one request to the loopback address:

```
{"model":"phi3:3.8b-mini-4k-instruct-q4_K_M","response":" Blue","done":true,
 "load_duration":13440745513,"total_duration":14849523261,…}
llama_model_loader: loaded meta data … from /usr/share/alo/models/blobs/sha256-01ec9e67…
```

The one weights blob left is the one the runtime loads, and it answers off it
in 14.8 s cold on four cores with no network at all.

One line I do not understand and will not pretend to: on the rebuilt image the
runtime logs `total blobs: 0` at start where it logged `total blobs: 5` on
task 33's, with the four blobs present, listed, shown and loaded. Checked with
the switch on and off — the line follows the store, not the switch. It is
recorded here as an observation, not a finding.

## Decisions I made, and why

- **Remove the blob in the stage, rather than import differently or carry it.**
  Priced above and in `docs/quirks.md`. The runtime's own prune attempt at every
  start is the strongest argument: the engine itself considers the file unused.
- **`rm -f`, held by `ollama show` and the walk, rather than `rm`.** A future
  runtime that stopped leaving the copy would otherwise fail the build on a
  file that was never there; one that started *referencing* the copy fails on
  `show`, which is the failure that matters.
- **`OLLAMA_NO_CLOUD=1` set, as a second lock.** The argument is one sentence
  above. What it is not: a replacement for the filter, which is the only one of
  the two a runtime update cannot switch off.
- **Three capabilities on the container's init, reported, rather than
  `--privileged` or root.** The unit and its process hold nothing either way;
  the plan asked for what could honestly be measured, said honestly.
- **The control request went to `quay.io`, not `ollama.com`.** Task 33 caused
  one request to the publisher by inspection and said so; proving the network
  was there needed a host, and the one the build already talks to costs nothing
  new.

## What this does **not** claim

- **Not a boot**, and **no *On the machine* box moves.** `ROADMAP.md`'s image
  line keeps its machine half empty. `docs/features.md` was not touched.
- **Not *arrives ready to run***: that still waits on a machine.
- **Not the egress measurement at a boot.** A packet counter beside a booted
  image is still owed, with the boot. What is shown is the unit's filter
  refusing the runtime's requests under the image's own systemd.

## Verification

Windows 11 host, checkout `C:\dev\alo-os-claude`; the build and the
measurements in Ubuntu 26.04 under WSL2.

| Check | Result |
|---|---|
| `podman build -f image/Containerfile -t alo-os:dev .` | exit 0, 7:48, 6,607,474,716 bytes |
| `cargo fmt --all -- --check` | clean |
| `cargo clippy -p alo-image --all-targets -- -D warnings` | clean |
| `cargo test -p alo-image` | 165 unit + 25 integration tests, all passing |
| `cargo doc -p alo-image --no-deps` | clean |

The whole-workspace suite is the supervisor's, as the task instructs.

## Proposed changes to shared documents

**`CHANGELOG.md`** — under Unreleased:

> The image now carries the local model's weights once: the build removes the
> unreferenced copy the runtime's import leaves behind and refuses a store with
> any blob its manifest does not name, and the image is 6.15 GiB where it was
> 8.38. The model service sets the runtime's own no-cloud switch beside its
> network filter, so the runtime makes no request to its publisher at all; the
> filter itself was watched refusing those requests under the image's own
> systemd, with the packets counted on both sides of the boundary. Not yet a
> boot.

**`ROADMAP.md`** — no change. The image line's machine half stays empty.

**`docs/autonomy/QUEUE.md`** — no change proposed. One observation for the
owner rather than a task: `COPY . /alo-os` copies the whole checkout into the
build stage, so any file changing anywhere recompiles the three binaries; a
`.containerignore` naming what the build does not read would keep a docs change
from costing a Rust build.

**`docs/autonomy/STATE.md`** — reference this report; task 34 done in
`docs/autonomy/v0-01-delivery-plan.md`, task 35 already written there.
