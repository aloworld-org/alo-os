# Three primary model choices, separate from privacy settings

Date: 2026-09-08. Workstream: owner direction and integration documentation.
Status: ready for integration, documentation only; no runtime behavior changed.

## Direction clarified by the owner

The main model-selection flow has exactly three source choices:

1. Local models: the person's models and local runtimes.
2. Your own API provider: compatible third-party APIs with the person's credentials.
3. Alo: alo's model service, with honest location/provider and commercial terms.

`docs/features.md` now states these separately from its existing four deployment
configurations. The previous heading described technical configurations and could
be misread as the main setup menu; it now distinguishes the two concepts.
The no-agent configuration remains available as an opt-out. Existing paired-
machine support is retained without inventing a fourth main source category;
detailed UI placement has not been decided by this clarification.

"Prefer local processing" and "keep questions on this PC" are not replacements
for these three choices. Any advanced privacy controls belong separately in
settings; neither their implementation nor changed enforcement is approved here.
Model ownership still does not establish where processing occurs. There is no
silent switch from a local selection to an API, or from one provider to another.

## Scope and decisions preserved

No model/provider monopoly, mandatory alo subscription for third-party paths, or
ownership-based privacy qualification is introduced. Protocol compatibility,
typed capabilities, accepted organisation policy and release tiers still apply.
Alo hosting's existing later-release tier is not promoted or marked delivered.
No unavailable endpoint or integration is to be presented as functional.

ADR 0021 remains PROPOSED. The owner approved this three-choice presentation
direction, not Option A/B/C, a change to `ThisMachineOnly`, a new kernel programme,
or a weakened egress guarantee. Truthful labels do not prove confinement.

## Verification and handoff

Inspected the current feature section, release tiers and ADR 0021 status.
`git diff --check` passed. These two Markdown files are the only changes; no
runtime or hardware tests are claimed and no test was invented to certify prose.
Used the clean integration helper checkout, leaving the active desktop checkout
and Claude's checkout untouched. The Claude prompt is delivered in chat only.

The integration worker should reference this report in STATE and apply the
three-choice distinction when scheduling model-selection UI. Preserve existing
unfinished delivery items and all physical acceptance requirements. No functional
changelog or completed release checkbox is warranted by this clarification.
