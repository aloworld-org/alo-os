# alo OS

An operating system whose interface is an agent and whose first-class workload
is a language model you own.

The shell is native. Agents reach the machine — the filesystem, the applications
installed on it, the window in front of you, the printer — through one
enumerated list of verbs. Every verb is proposed before it runs and recorded
after, and anything a verb can do, a person can do by hand.

Where inference happens is a choice, and it is always the owner's: a model on
this machine, a model on a machine paired with it, a provider they added, or no
model at all. None of the four is a fallback for another, nothing moves between
them on its own, and the machine says which one is in use. Where the answer is
the first, nothing leaves the building.

The claim is narrow and testable:

> An action a person would take by hand can be proposed by an agent, approved in
> one click, and afterwards explained — and the model that proposed it ran on
> hardware the customer owns.

## What it is not

- **Not a Linux distribution with a web application on it.** The shell is
  native. The kernel is Linux, unmodified.
- **Not a general-purpose distribution.** No package manager for the world.
- **Not a device-management product.** Fleet enrollment exists for alo OS
  machines; third-party devices are out of scope.
- **Not a model trainer.** Open weights are served and adapted, not trained from
  scratch, and no inference kernels are written here.
- **Not a phone or a tablet.**

## How it is built

Rust, and pinned upstream engines. The experience and the policy are ours; every
commodity underneath — the kernel, the graphics stack, the model runtime, the
fine-tuning stack — is rented, configured rather than patched, and pinned to an
exact version.

The decisions that shape the system are in `docs/decisions/`, one file per
decision, each with the alternatives it rejected. `docs/features.md` is the only
list of what gets built, and `ROADMAP.md` the only order it is built in.

## Design

The screens are in Figma: <https://www.figma.com/design/8q0JVtnLroZYNdDkIQeJni>.
`docs/design/figma-brief.md` is the brief they were drawn from.

## Licence

GPL-3.0-or-later for the code in this repository. The system image is an
**aggregate**: every rented component keeps its own licence, and each image
publishes an SBOM naming them. See `LICENSE`.

"alo" is a trademark. The code is open; the name is not — fork it freely, ship
it under your own name.

## Contributing

See `CONTRIBUTING.md`. Read `docs/decisions/0001-the-capability-model.md` first
if you intend to touch anything an agent can reach; the rest of the security
posture rests on it.

Security reports: `SECURITY.md`. Please do not open a public issue.
