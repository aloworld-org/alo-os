"""Read the person's pending sentence and number without answering its card.

The connected socket is caller-owned. The installed controller connects only
to the disposable person's fixed daemon socket. Empty Waiting is not proof
that a task completed; the process observer establishes completion separately.
"""

import json
import math
import time

from record_delta import unique_object, nonfinite
from record_shape import require, u64, finite_float

LIMIT = 64 * 1024
REQUEST = b'{"format":1,"asks":{"waiting":{}}}\n'


def read_pending(connection, *, enabled, timeout=3):
    """Send only Waiting, then return the exact local microphone card or None."""
    require(type(enabled) is bool, 'The expected microphone choice must be boolean')
    require(type(timeout) in (int, float) and math.isfinite(timeout) and 0 < timeout <= 3,
            'The pending read timeout must be finite and at most three seconds')
    deadline = time.monotonic() + timeout
    connection.settimeout(timeout)
    connection.sendall(REQUEST)
    data = bytearray()
    while not data.endswith(b'\n'):
        remaining = deadline - time.monotonic()
        require(remaining > 0, 'The pending read exceeded its time bound')
        connection.settimeout(remaining)
        part = connection.recv(LIMIT + 1 - len(data))
        require(part, 'The pending reply ended before its complete line')
        data.extend(part)
        require(len(data) <= LIMIT, 'The pending reply exceeds its size bound')
    require(data.count(b'\n') == 1, 'The pending reply contains more than one message')
    try:
        body = json.loads(data.decode('utf-8'), object_pairs_hook=unique_object,
                          parse_constant=nonfinite, parse_float=finite_float)
        require(isinstance(body, dict) and type(body.get('format')) is int
                and body['format'] == 1 and set(body) == {'format', 'tells'},
                'The pending reply has the wrong envelope')
        require(isinstance(body['tells'], dict) and set(body['tells']) == {'waiting'},
                'The daemon did not answer Waiting')
        waiting = body['tells']['waiting']
        require(isinstance(waiting, dict) and set(waiting) == {'changes'}
                and isinstance(waiting['changes'], list), 'The pending changes are malformed')
        changes = waiting['changes']
        if not changes:
            return None
        require(len(changes) == 1 and isinstance(changes[0], dict),
                'The pending microphone card is ambiguous')
        change = changes[0]
        require(set(change) <= {'number', 'sentence', 'lapses_in', 'touches', 'changes', 'leaves', 'texts'},
                'The pending card contains undeclared or remote fields')
        sentence = ('enable' if enabled else 'disable') + ' the microphone for this alo Studio recording'
        require(u64(change['number']) and u64(change['lapses_in']) and change['lapses_in'] > 0,
                'The pending microphone card has no live typed identity')
        require(change.get('texts', []) == []
                and change['touches'] == [{'application': 'org.aloworld.Studio'}],
                'The pending card is not the local Studio microphone change')
        words = change['sentence']
        require(isinstance(words, dict) and set(words) == {'text', 'came_from'} and words.get('text') == sentence
                and words.get('came_from') in ('the-source', 'translation'),
                'The pending card does not say the expected microphone change')
        for name in ('changes', 'leaves'):
            wording = change.get(name)
            require(wording is None or (isinstance(wording, dict)
                    and set(wording) == {'text', 'came_from'} and isinstance(wording['text'], str)
                    and wording['came_from'] in ('the-source', 'translation', 'no-sentence')),
                    'The pending card contains malformed optional wording')
        return dict(number=change['number'], sentence=sentence)
    except (KeyError, TypeError, UnicodeError, RecursionError) as error:
        raise ValueError('The pending microphone reply is incomplete or malformed') from error
