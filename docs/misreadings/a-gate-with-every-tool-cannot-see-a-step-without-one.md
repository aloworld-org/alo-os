# A gate with every tool cannot see a step without one

**Found 2026-10-30, the walking lane, when an image of main would not build.**

## What was concluded

#1027 was green on every gate, so main could still produce an image.

## What was true

The image's compositor step runs
`cargo build --release --package alo-shell --package alo-desktop` in a container
with no `bpf-linker`. #1027 added `alo-agentd` to `alo-desktop`'s dependencies,
which brought `alo-bounding-kernel` into that build, and the step failed with
*linker `bpf-linker` not found*.

## The mechanism

CI builds the whole workspace on a runner with every tool installed. So it
answers *does this compile with everything present?* The image asks a narrower
question: *does this package compile with only what this step installs?* A new
dependency edge changes the second answer and not the first.

## The cure

A dependency added to a package the image builds on its own is checked against
that step's toolchain, not the workspace's. #1031 made the edge a test:
`crates/alo-desktop/tests/the_desktop_is_built_without_the_agent_service.rs`
walks the desktop's and shell's dependency graph and refuses `alo-agentd` and
`alo-bounding*`.
