"""Derive a private reference package without changing the normal application."""

import copy
import json
import sys


def manifest(normal):
    """Reuse the normal SDK and permissions, building only the reference example."""
    fixture = copy.deepcopy(normal)
    fixture['id'] = 'org.aloworld.Studio.Verification'
    fixture['command'] = 'studio-reference-window'
    fixture['modules'][0]['build-commands'] = [
        'cargo build --release --locked --offline --features verification --example studio-reference-window',
        'install -Dm755 target/release/examples/studio-reference-window /app/bin/studio-reference-window',
        'install -Dm644 verification/org.aloworld.Studio.Verification.desktop /app/share/applications/org.aloworld.Studio.Verification.desktop',
        'install -Dm644 LICENSE /app/share/licenses/org.aloworld.Studio.Verification/LICENSE',
        'install -Dm644 data/org.aloworld.Studio.svg /app/share/icons/hicolor/scalable/apps/org.aloworld.Studio.Verification.svg',
    ]
    return fixture


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: make_reference_manifest.py normal-manifest.json')
    with open(sys.argv[1], encoding='utf-8') as source:
        print(json.dumps(manifest(json.load(source)), indent=2))
