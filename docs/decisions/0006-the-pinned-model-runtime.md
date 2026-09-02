# ADR 0006 — Ollama is the pinned model runtime, behind our own trait

**Status:** accepted
**Date:** 2026-09-02
**Context:** `crates/alo-models`, `docs/features.md` (the AI stack), `ROADMAP.md`
v0.01; `alo-workplace`'s `AiConfig`, which has spoken to an OpenAI-compatible
endpoint since 2025

## The decision in one line

**Ollama** is the model runtime alo OS ships — pinned, configured, never
patched — and nothing in this repository talks to it directly: every caller
goes through our own `ModelRuntime` trait, so the runtime is a dependency we
can replace rather than a shape our product is stuck in.

## What was actually wrong

`docs/features.md` and `ROADMAP.md` both said "the pinned runtime" without ever
naming one. That is the kind of gap that gets filled by whoever writes the first
line of code, and then defended for years because it is already there.

## Why Ollama

**It is the whole of the v0.01 model stack in one component.** The roadmap item
is "catalogue, pull, serve, unload, remove". Ollama does downloading, storage,
serving, unloading and eviction. Choosing anything else means building most of
that ourselves, which is the opposite of renting a commodity.

**It speaks the API the workspace already speaks.** `AiConfig.base_url` in
`alo-workplace` documents an OpenAI-compatible endpoint at `localhost:11434` —
Ollama's own default. So "the agents point at the local model" is a
configuration change against code that shipped a year ago rather than a new
inference path, which is exactly what makes the v0.01 exit gate reachable.

**It is llama.cpp underneath**, which is where the actual inference work and
the hardware support live. We are renting the thing we said we would rent, with
model management on top rather than instead.

**Its licence is MIT**, so nothing about shipping it in a GPL-3.0 system image
is complicated.

## Why behind a trait, and not just called directly

Because the runtime is the part of this stack most likely to be replaced, and
the part we least want to argue about later.

`ModelRuntime` describes what alo OS needs — which models are installed, fetch
one, remove one, what is loaded now, load and unload — in *our* vocabulary. The
Ollama adapter is one implementation of it. That buys three things:

1. **vLLM stays possible.** `docs/features.md` puts "serving more than one
   person from one workstation" at v1. When that arrives, it is a second
   implementation, not a rewrite of everything that calls a model.
2. **The product's shape is ours.** Model management is a surface people see —
   the disk accounting, the licence gate, what "installed" means. If that
   vocabulary is Ollama's, then every future decision about it is really a
   question about what Ollama does.
3. **It is testable without a runtime installed.** A trait can be implemented by
   a stub, so the code that decides *what* to do is tested without a five
   gigabyte download, and the adapter is tested against a stub HTTP server that
   returns the responses Ollama returns. Neither test needs a GPU.

The doctrine is unchanged from every other rented engine: pinned to a version in
the image, configured, never patched. A patch to Ollama requires an ADR, as with
Linux, Mesa and systemd.

## Consequences

- **Ollama's API is not our API.** Nothing outside the adapter names an Ollama
  endpoint, model-name convention or response field. A reviewer should be able
  to find every mention of Ollama in one file.
- **The version is pinned in the image**, and moving it is a deliberate change
  with the model runtime versioned alongside the drivers it needs — which is
  `docs/features.md`'s promise that an upgrade cannot break a working stack.
- **The catalogue is ours, not Ollama's.** `crates/alo-models`'s catalogue
  decides what alo OS offers and states each licence; Ollama's library is not a
  curated list and has no licence gate. What we offer is a decision, not a
  mirror of somebody else's index.
- Model *weights* are still never redistributed by us: the machine fetches from
  upstream, and the catalogue records from where.

## Alternatives rejected

**vLLM.** Better throughput and far better at serving many concurrent users —
and worse at what this SKU is for. It has no model management to speak of, is
harder to operate, and buys performance v0.01 does not need at the cost of the
downloading, storage and eviction that v0.01 is entirely about. Revisit at v1
with the multi-user requirement, as a second implementation of the trait.

**llama.cpp directly.** We would rebuild model management ourselves — download,
storage layout, eviction, an HTTP surface — which is precisely the work Ollama
already does over llama.cpp. Renting the layer below the one we need is not
thrift, it is a longer route to the same place.

**Our own inference runtime.** An explicit non-goal (`docs/features.md`): we do
not write inference kernels and do not compete with llama.cpp or vLLM.
