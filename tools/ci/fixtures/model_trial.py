#!/usr/bin/python3 -IB
"""Harmless trial stand-in: write only fixed synthetic test prompts, never call a provider."""

import json
import os
from pathlib import Path
import sys


level = os.environ['ALO_EVERY_TRY_LEVEL']
observed = {
    'level': level,
    'tries': os.environ.get('ALO_EVERY_TRY_TRIES'),
    'only': os.environ.get('ALO_EVERY_TRY_ONLY'),
    'model': os.environ.get('ALO_EVERY_TRY_MODEL'),
    'endpoint': os.environ.get('ALO_EVERY_TRY_ENDPOINT'),
    'shown': os.environ.get('ALO_EVERY_TRY_SHOWN'),
    'argv': sys.argv[1:],
}
with Path('trial-calls.jsonl').open('a') as output:
    output.write(json.dumps(observed) + '\n')
shown = observed['shown']
if shown:
    target = Path(shown)
    # Like AtAProvider::writing_down: append, without creating its parent.
    with target.open('a') as output:
        output.write(f'=== question 1 ===\nThe request: synthetic {level} recording request.\n')
        output.write('=== question 2 ===\nThe answer: synthetic completed status.\n')
        output.write('Compare the whole request with what came back.\n')
mode = os.environ['MODEL_FAKE_FAILURE']
sys.exit(1 if mode in {level, 'both'} else 0)
