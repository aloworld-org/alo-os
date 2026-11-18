"""Validate the bounded subset of record events used by the private Studio walk.

Unknown events refuse certification instead of being silently discarded. This is
an evidence reader, not a replacement for alo-keeping's general record reader.
"""

import math
import re


def require(condition, message):
    if not condition:
        raise ValueError(message)


def u64(value):
    return type(value) is int and 0 <= value < 2 ** 64


def timestamp(value):
    require(isinstance(value, dict) and u64(value.get('secs_since_epoch'))
            and type(value.get('nanos_since_epoch')) is int
            and 0 <= value['nanos_since_epoch'] < 1_000_000_000,
            'The record timestamp is malformed')
    return value['secs_since_epoch'], value['nanos_since_epoch']


def finite_float(token):
    value = float(token)
    require(math.isfinite(value), 'The record contains a nonfinite JSON number')
    return value


def retention(value):
    require(value == 'forever' or (isinstance(value, dict) and len(value) == 1
            and type(value.get('for-days')) is int and 0 < value['for-days'] < 2 ** 32),
            'The record retention rule is malformed')


def named(value):
    return isinstance(value, str) and bool(value.strip())


def operation(what):
    require(isinstance(what, dict) and isinstance(what.get('verb'), str)
            and re.fullmatch(r'[a-z][a-z0-9_.]*', what['verb'])
            and what.get('effect') in ('read', 'change')
            and named(what.get('sentence')) and isinstance(what.get('arguments'), dict),
            'The recorded operation is malformed')
    # Studio exposes only one read. No changing verb may disappear as read noise.
    if what['verb'].startswith('studio.'):
        require(what['effect'] == ('read' if what['verb'] == 'studio.status' else 'change'),
                'The recorded Studio effect is inconsistent')
    for argument in what['arguments'].values():
        require(isinstance(argument, dict) and len(argument) == 1,
                'The recorded argument is malformed')
        tag, value = next(iter(argument.items()))
        require((tag == 'count' and type(value) is int and -(2 ** 63) <= value < 2 ** 63)
                or (tag in ('path', 'application', 'name', 'choice', 'text')
                    and isinstance(value, str)), 'The recorded argument is unreadable')


def event(happened):
    require(isinstance(happened, dict) and len(happened) == 1,
            'The record event is missing or ambiguous')
    tag, body = next(iter(happened.items()))
    require(isinstance(body, dict) and named(body.get('agent')),
            'The recorded event has no agent')
    if tag == 'ran':
        operation(body.get('what'))
        require('from_approval' in body and (body['from_approval'] is None or u64(body['from_approval'])),
                'The recorded approval is malformed')
        require(isinstance(body.get('against'), list) and all(u64(value) for value in body['against']),
                'The recorded grants are malformed')
        require(type(body.get('by_full_access', False)) is bool,
                'The recorded execution policy is malformed')
    elif tag == 'answered-here':
        pass
    elif tag == 'left':
        destination = body.get('destination')
        require(body.get('why') == 'asking' and isinstance(destination, dict)
                and set(destination) == {'provider'}, 'The recorded departure is unreadable')
        provider = destination['provider']
        require(isinstance(provider, dict) and named(provider.get('provider'))
                and provider.get('region') == 'unknown', 'The recorded provider is unreadable')
    elif tag == 'stopped':
        operation(body.get('what'))
        how = body.get('how')
        require(how == 'by-the-person' or (isinstance(how, dict) and len(how) == 1
                and next(iter(how)) in ('before-anybody-was-asked', 'at-the-moment')
                and named(next(iter(how.values())))), 'The recorded refusal is unreadable')
    else:
        raise ValueError('The record contains an event this private check cannot read')
