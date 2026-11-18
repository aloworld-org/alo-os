#!/usr/bin/env bash
set -euo pipefail
export ALO_STUDIO_PUBLISHER_PLUGIN_PATH=/usr/lib/alo-capture-provider/gstreamer-1.0
export CARGO_TARGET_DIR=/source/apps/alo-studio/target
bash apps/alo-studio/packaging/verify-pipewire.sh
export ALO_STUDIO_CAPTURE_PROVIDER="$CARGO_TARGET_DIR/debug/alo-capture-provider"
export ALO_STUDIO_CAPTURE_REMOTE="$CARGO_TARGET_DIR/debug/alo-capture-remote"
bash apps/alo-studio/packaging/verify-pipewire.sh
