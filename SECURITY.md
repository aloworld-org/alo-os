# Security

alo OS is sold on sovereignty and auditability. That makes security reports the
most valuable thing an outsider can give us, and it makes a slow or defensive
response worse than the bug.

## Reporting

**Please do not open a public issue for a security problem.**

Report privately through GitHub's private vulnerability reporting on this
repository: **Security → Report a vulnerability**.

Tell us what you found, how to reproduce it, and what you think it lets someone
do. A rough report sent early is more useful than a polished one sent late.

**What to expect:** an acknowledgement within 3 working days, an assessment
within 10, and a fix or a dated plan after that. We will tell you honestly if we
think it is not a vulnerability, and why. We will credit you by name unless you
ask us not to.

Please give us 90 days before public disclosure, or less if the fix ships
sooner. If we go quiet, or you disagree with our assessment, disclose — a vendor
who does not answer has forfeited the courtesy.

## What we are most interested in

The whole system matters, but these are where a bug is worst:

- **`alo-agentd` and the privileged broker.** Anything that lets an agent reach
  a path, application or device that was never granted; any escalation out of
  the person's own authority; any way to make one approval cause more than one
  action.
- **Anything that gets code of the model's own composition to execute.** This is
  the load-bearing rule of the whole design
  (`docs/decisions/0001-the-capability-model.md` §1). If you find a way around
  it — through an adapter, a path, an argument that reaches an interpreter — that
  is the highest-severity class of bug in this product.
- **Approval integrity.** Anything where the sentence a person approves does not
  match what actually runs, or where an approval persists beyond the single
  action it named.
- **Grants.** Escaping a granted path, widening a grant by using it, surviving a
  revocation, or outliving an expiry.
- **Egress.** Any network egress an agent causes that does not fire the
  indicator, and any telemetry at all — we ship none, so any is a bug.
- **Context.** Anything that reads the screen, the selection or the clipboard
  without an invocation.
- **The image and updates.** Unsigned or downgraded images accepted as bootable;
  anything that breaks rollback.

## Prompt injection

We assume the model is already saying whatever an attacker wants. So
"the agent can be talked into proposing something bad" is expected and is not by
itself a vulnerability — the approval is the control.

**It becomes a vulnerability when injected text causes an effect a person did
not approve**: something executed without a tap, a sentence that misdescribes
what ran, reach beyond a grant, or a change dressed as a read. Those we very
much want to hear about.

## Scope

In scope: this repository, the images we publish, and the adapters we ship.

Out of scope: the pinned upstream components we do not write — the Linux kernel,
Mesa, systemd, the model runtime and the fine-tuning stack. Report those to their
maintainers; tell us too if it affects our images, so we can pin a fixed version.
Third-party adapters we do not distribute are their authors' to fix, though we
will help route the report and can restrict a signed adapter that is dangerous.

## Our side of it

Security fixes ship to the stable channel as soon as they are ready, not on the
release train. Advisories are published with the fix, describing what was
possible — not "various security improvements". `alo-agentd` and the broker get
a third-party audit before v1, and we publish it whatever it says.
