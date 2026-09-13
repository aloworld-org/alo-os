#!/bin/sh
# Resume the Mac lane on the models-measured plan (docs/autonomy/a-loop-on-a-mac.md).
# Idempotent: a loop already running holds the lock and this exits.
#
# The gates run inside a Linux VM named by ALO_KERNEL_LOOP_LINUX, never on the
# Mac itself. Set it to whatever runs a shell in your VM: `limactl shell alo`,
# or `orb -m alo`. The loop appends `bash -lc "<script>"` to it.
set -eu
cd "$(dirname "$0")/../.."
git pull --ff-only -q
export ALO_LOOP_PLAN=docs/autonomy/v0-5-the-models-measured-plan.md
export ALO_KERNEL_LOOP_LINUX="${ALO_KERNEL_LOOP_LINUX:-limactl shell alo}"
export ALO_KERNEL_LOOP_WORKER="${ALO_KERNEL_LOOP_WORKER:-$(command -v claude)}"
mkdir -p .kernel-loop
exec ./tools/kernel-loop/target/release/alo-kernel-loop run >> .kernel-loop/resume.log 2>&1
