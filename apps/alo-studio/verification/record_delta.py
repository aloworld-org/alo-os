"""Bounded person-side observations of Studio's execution record.

This reads evidence only. One recorded run is not proof of native approval,
application state or single D-Bus delivery; the controller must observe those
independently. No record contents outside the selected summary are returned.
"""

from dataclasses import dataclass
import hashlib
import json
import os
import pathlib
import stat

from record_shape import event, finite_float, require, retention, timestamp, u64

FILE_LIMIT = 4 * 1024 * 1024
DELTA_LIMIT = 1024 * 1024
LINE_LIMIT = 64 * 1024


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, 'The record contains a duplicate JSON field')
        result[key] = value
    return result


def nonfinite(_):
    raise ValueError('The record contains a nonfinite JSON number')


def entries(data, *, header, previous=(0, 0)):
    require(data and data.endswith(b'\n'), 'The record ends in an incomplete line')
    parsed = []
    for index, line in enumerate(data.splitlines(keepends=True)):
        require(len(line) <= LINE_LIMIT and line.endswith(b'\n'),
                'The record line is incomplete or exceeds its bound')
        try:
            value = json.loads(line.decode('utf-8'), object_pairs_hook=unique_object,
                               parse_constant=nonfinite, parse_float=finite_float)
        except (UnicodeError, RecursionError) as error:
            raise ValueError('The record line cannot be decoded') from error
        require(isinstance(value, dict), 'A record line must be an object')
        if header and index == 0:
            require(type(value.get('format')) is int and value['format'] == 1,
                    'The record has no complete version-one header')
            if 'since' in value:
                previous = timestamp(value['since'])
            if 'under' in value:
                retention(value['under'])
            continue
        moment = timestamp(value.get('at'))
        require(moment >= previous, 'The record contradicts its retained history or append order')
        previous = moment
        event(value.get('happened'))
        if 'at_full_access' in value:
            require(type(value['at_full_access']) is bool, 'The record access flag is malformed')
        parsed.append(value)
    return parsed, previous


def read_record(path):
    """Refuse links/devices and changing reads without waiting on special files."""
    path = pathlib.Path(path)
    with os.fdopen(os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK), 'rb') as source:
        before = os.fstat(source.fileno())
        require(stat.S_ISREG(before.st_mode) and 0 < before.st_size <= FILE_LIMIT,
                'The record must be a bounded regular file')
        data = source.read(FILE_LIMIT + 1)
        after = os.fstat(source.fileno())
        current = path.lstat()
        require(len(data) <= FILE_LIMIT and before.st_size == len(data) == after.st_size
                and before.st_mtime_ns == after.st_mtime_ns
                and before.st_ctime_ns == after.st_ctime_ns
                and stat.S_ISREG(current.st_mode)
                and (current.st_dev, current.st_ino) == (before.st_dev, before.st_ino),
                'The record changed while its evidence was read')
    return data, (before.st_dev, before.st_ino)


@dataclass(frozen=True)
class Checkpoint:
    """Identity of complete, validated history before the observed operation."""
    identity: tuple
    offset: int
    digest: bytes
    last_moment: tuple


def snapshot(path):
    """Remember existing complete history; never create or repair a record."""
    data, identity = read_record(path)
    _, last_moment = entries(data, header=True)
    return Checkpoint(identity, len(data), hashlib.sha256(data).digest(), last_moment)


def verify_microphone_change(path, checkpoint, *, enabled, from_approval, full_access):
    """Require one appended Studio change with the expected action and authority."""
    require(type(enabled) is bool and type(full_access) is bool,
            'The expected action and access policy must be booleans')
    require(from_approval is None if full_access else u64(from_approval),
            'The expected approval does not match the access policy')
    appended = appended_entries(path, checkpoint)
    found = []
    for entry in appended:
        run = entry['happened'].get('ran')
        if run is not None and run['what']['verb'].startswith('studio.') and run['what']['effect'] == 'change':
            found.append((entry, run))
    require(len(found) == 1, 'Expected exactly one recorded Studio change')
    entry, run = found[0]
    verb = 'studio.enable_microphone' if enabled else 'studio.disable_microphone'
    require(run['agent'] == 'alo' and run['what']['verb'] == verb
            and run['what']['arguments'] == {} and run['from_approval'] == from_approval,
            'The recorded change does not match this agent action and approval')
    require(entry.get('at_full_access', False) is full_access
            and run.get('by_full_access', False) is full_access,
            'The recorded change used a different access policy')
    require((full_access or run['against']) and len(run['against']) <= 16,
            'The recorded grant evidence is missing or exceeds its summary bound')
    return dict(agent='alo', verb=verb, recorded_runs=1, from_approval=from_approval,
                against=run['against'], at_full_access=full_access, by_full_access=full_access)


def appended_entries(path, checkpoint):
    """Read the same bounded, unchanged prefix for success and refusal evidence."""
    require(isinstance(checkpoint, Checkpoint), 'A prior record checkpoint is required')
    data, identity = read_record(path)
    require(identity == checkpoint.identity and checkpoint.offset <= len(data)
            and hashlib.sha256(data[:checkpoint.offset]).digest() == checkpoint.digest,
            'The record was replaced, shortened or rewritten')
    delta = data[checkpoint.offset:]
    require(0 < len(delta) <= DELTA_LIMIT, 'The record append is empty or exceeds its bound')
    appended, _ = entries(delta, header=False, previous=checkpoint.last_moment)
    return appended


def verify_microphone_refusal(path, checkpoint, *, enabled):
    """Require one Ask-policy refusal at execution time and no Studio changes.

    Stopped records carry no approval number. The controller retains the native
    pending number separately; this summary does not invent that correlation.
    """
    require(type(enabled) is bool, 'The expected microphone option must be boolean')
    found = []
    appended = appended_entries(path, checkpoint)
    for entry in appended:
        tag, body = next(iter(entry['happened'].items()))
        if tag in ('ran', 'stopped') and body['what']['verb'].startswith('studio.') and body['what']['effect'] == 'change':
            require(tag == 'stopped', 'A Studio change ran during the refused request')
            found.append((entry, body))
    require(len(found) == 1, 'Expected exactly one recorded Studio refusal')
    entry, stopped = found[0]
    verb = 'studio.enable_microphone' if enabled else 'studio.disable_microphone'
    require(stopped['agent'] == 'alo' and stopped['what']['verb'] == verb
            and stopped['what']['arguments'] == {} and entry.get('at_full_access', False) is False
            and isinstance(stopped['how'], dict) and set(stopped['how']) == {'at-the-moment'},
            'The recorded refusal does not match the stale Ask-policy microphone change')
    return dict(agent='alo', verb=verb, recorded_refusals=1, how='at-the-moment', at_full_access=False)
