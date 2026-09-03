# alo OS

**The sovereign AI workstation. Your models, your hardware, your building.**

alo OS is an operating system whose interface is an agent and whose
first-class workload is a language model you own.

It boots into a native shell you sign into with an alo identity. The
agents reach the whole machine — the real filesystem, the applications
you installed, the window in front of you, the printer nobody can
configure — through one enumerated list of verbs, each proposed before
it runs and recorded after.

**It can run entirely on your own hardware — and it works just as well if
you would rather it did not.** A model on this machine, a model on a
machine you paired with, a provider you added, or no model at all: four
supported ways, none of them a fallback for another, and nothing moving
between them without you saying so. For the organisations who need the
first one, **nothing leaves the building at all** — and the machine says
so, out loud, whichever one you chose.

The distinguishing claim is narrow and testable:

> An action a person would take by hand can be proposed by an agent,
> approved in one click, and afterwards explained — and the model that
> proposed it ran on hardware the customer owns.

**Status: early. Nothing here is released.** This repository currently
holds the decisions and contracts that have to exist before the code
is reviewable. See `ROADMAP.md` for what is actually being built and
in what order.

## Who it is for

Anyone who wants an operating system whose interface is an agent and whose model
they control. **Built in Europe, not only for Europe** — a hospital in Ohio, a
law firm in São Paulo and a bank in Singapore have the same problem as a German
municipality: work that AI would help with, and records that cannot be sent to
somebody else's inference provider.

Europe is where we start, because that is where the migration window and the
procurement rules are. It is not where the product stops. Where alo OS lets an
organisation set a rule — which region inference may happen in, which providers
are permitted — the rule is theirs to name, not ours to ship.

## Why it exists

Three things arrived together, first in Europe and not only there. Windows 10
support ended in October 2025, and a large share of business and public-sector
machines cannot run Windows 11. European public bodies are genuinely
moving off Microsoft, with procurement language and budget behind it.
And AI arrived in places the data cannot follow: hospitals, law firms,
notaries, defence suppliers and municipalities under rules that forbid
sending records to a foreign inference provider, whose only options
today are to do without or to send it anyway and hope.

`alo-workplace` already answers the first two. alo OS answers the
third, and it is the only one of the three whose answer has to reach
below the browser.

## What it is not

- **Not a Linux distribution with a web app on it.** The shell is
  native. The kernel is Linux, unmodified, because hardware support is
  where OS projects die and we do not fight that battle.
- **Not a general-purpose distribution.** No package manager for the
  world, no attempt to be Ubuntu. One system that does one job.
- **Not a device-management product.** Fleet enrollment and policy
  exist for alo OS machines. We do not manage third-party devices.
- **Not a model trainer.** We serve and adapt open weights. We do not
  train from scratch and we do not write inference kernels.
- **Not a phone or a tablet.** Not in v1, possibly never.

## How it is built

Rust, and pinned upstream engines. We own the experience and the
policy; we rent every commodity — the kernel, the graphics stack, the
model runtime, the fine-tuning stack — and we configure them rather
than patching them. The same doctrine one layer up governs
`alo-workplace`.

The rules are in `CLAUDE.md`, and they are short enough to read.

## The parts

| Repository | What it is |
|---|---|
| `alo-os` | This one: the shell, `alo-agentd`, the system services, the image. |
| `alo-workplace` | The workspace that runs on it — mail, files, chat, documents, the product agents. |
| `alo-engine` | The rendering engine, in Rust. **Decided, not started, not scheduled** — no repository yet. |

alo OS is **useful with no account at all**: local files, local models,
downloading and fine-tuning, and agents driving the applications you
installed. An alo workplace tenant adds your mail, calendar and
business records. A machine that was a brick without a subscription
would be a worse product and a dishonest one.

## Design

The screens live in Figma: <https://www.figma.com/design/8q0JVtnLroZYNdDkIQeJni>

`docs/design/figma-brief.md` is the brief they were drawn from — what each screen
has to do, and what the system must never do.

## Licence

GPL-3.0-or-later for the code in this repository. The system image is
an **aggregate**: every rented component keeps its own licence, and
each image publishes an SBOM naming them. See `LICENSE`.

"alo" is a trademark. The code is open; the name is not — fork it
freely, ship it under your own name.

## Contributing

`CONTRIBUTING.md`. Read `docs/decisions/0001-the-capability-model.md`
first if you intend to touch anything the agent can reach; it is the
document the rest of the security posture rests on.

Security reports: `SECURITY.md`. Please do not open a public issue.
