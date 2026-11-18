"""Report decoded Studio media evidence without certifying the installed flow.

Run against the retained receiver directory, with an exact expected source and
explicit marker/motion rectangles for each recording. This reads evidence only;
it does not infer regions, repair media or grant any agent capability.
"""

import argparse
import hashlib
import json
import pathlib
import re
import sys

# Isolated Python omits the script directory. Import only these checked-in
# siblings, never modules from the caller's working directory or environment.
if __name__ == '__main__':
    sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))

import decode_media
import recording_health
import reference_health

CORE = ('provenance', 'reference', 'screen_decode', 'screen_recording',
        'window_decode', 'window_recording')


def unread(reason):
    """Keep an unattempted check distinct from an observed refusal."""
    return dict(status='not_reached', reason=reason)


def evidence(path, report, *, limit):
    """Read a bounded regular artifact and retain its consumed-byte identity."""
    if path.is_symlink() or not path.is_file() or path.stat().st_size > limit:
        raise ValueError(f'{path.name} is missing or is not a bounded regular file')
    data = path.read_bytes()
    if len(data) > limit:
        raise ValueError(f'{path.name} exceeds its evidence limit')
    report['inputs'][path.name] = dict(sha256=hashlib.sha256(data).hexdigest())
    return data


def identity(session, source):
    """Validate the observed deployment without claiming its authenticity."""
    if not isinstance(source, str) or not re.fullmatch('[0-9a-f]{40}', source):
        raise ValueError('an exact lowercase expected source revision is required')
    if (not isinstance(session, dict) or type(session.get('version')) is not int
            or session['version'] != 1 or not isinstance(session.get('events'), list)
            or any(not isinstance(event, dict) for event in session['events'])):
        raise ValueError('the session report has no valid versioned event inventory')
    events = [event for event in session['events'] if event.get('kind') == 'installed-provenance']
    if len(events) != 1:
        raise ValueError('exactly one installed provenance observation is required')
    event = events[0]
    if event.get('source') != source or event.get('verification') is not False:
        raise ValueError('installed provenance does not identify the expected normal source')
    for key in ('studio_commit', 'reference_commit', 'runtime_commit'):
        if not isinstance(event.get(key), str) or not re.fullmatch('[0-9a-f]{64}', event[key]):
            raise ValueError(f'the observed {key} is incomplete')
    if (not isinstance(event.get('runtime'), str)
            or not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_-]+/[A-Za-z0-9_.-]+', event['runtime'])):
        raise ValueError('the observed deployed runtime identity is invalid')
    return {key: event[key] for key in ('source', 'verification', 'studio_commit',
                                       'reference_commit', 'runtime', 'runtime_commit')}


def audit(evidence_dir, *, expected_source, regions):
    """Return a JSON-safe media verdict; full installed acceptance stays pending."""
    folder = pathlib.Path(evidence_dir)
    try:
        # Reject nonfinite or non-JSON API values before copying them into the
        # report. Invalid regions remain a decode refusal, never guessed boxes.
        regions = json.loads(json.dumps(regions, allow_nan=False))
    except (TypeError, ValueError):
        regions = None
    report = dict(version=1, scope='decoded-recording-media', status='failed',
                  installed_acceptance='not_reached', inputs={}, regions=regions,
                  checks={name: unread('requires valid installed provenance') for name in CORE})
    checks = report['checks']
    checks['agent_permissions'] = unread('requires a separate real installed-agent permission walk')
    checks['hardware_microphone'] = unread('a virtual audio fixture cannot certify a hardware microphone')
    checks['native_person_flow'] = unread('media and deployment observations do not certify native consent, cleanup or source isolation')
    try:
        session = json.loads(evidence(folder / 'session-report.json', report, limit=1_048_576))
        checks['provenance'] = dict(status='passed', identity=identity(session, expected_source))
    except (OSError, ValueError, TypeError) as error:
        checks['provenance'] = dict(status='failed', reason=str(error))
        return report
    try:
        trace = json.loads(evidence(folder / 'reference.json', report, limit=1_048_576))
        checks['reference'] = dict(status='passed', measurements=reference_health.inspect(trace))
    except (OSError, ValueError, TypeError) as error:
        checks['reference'] = dict(status='failed', reason=str(error))
    for source in ('screen', 'window'):
        decoding, recording = source + '_decode', source + '_recording'
        checks[recording] = unread('requires a complete decode and the independent reference for microphone synchronization')
        try:
            if not isinstance(regions, dict):
                raise ValueError('explicit JSON-safe recording regions are required')
            boxes = regions[source]
            if not isinstance(boxes, dict) or set(boxes) != {'marker_region', 'motion_region'}:
                raise ValueError(f'explicit marker and motion regions are required for {source}')
            path = folder / (source + '.webm')
            evidence(path, report, limit=67_108_864)
            video, audio = decode_media.decode(path, **boxes)
            checks[decoding] = dict(status='passed')
        except (OSError, ValueError, KeyError, TypeError) as error:
            checks[decoding] = dict(status='failed', reason=str(error))
            continue
        if source == 'window' and checks['reference']['status'] != 'passed':
            continue
        try:
            measurements = recording_health.inspect(video, audio, microphone=source == 'window')
            checks[recording] = dict(status='passed', measurements=measurements)
        except (ValueError, TypeError) as error:
            checks[recording] = dict(status='failed', reason=str(error))
    if all(checks[name]['status'] == 'passed' for name in CORE):
        report['status'] = 'passed'
    return report


def main():
    """Print one structured verdict; exit zero only for the declared media scope."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence')
    parser.add_argument('--source', required=True)
    parser.add_argument('--regions', required=True, type=pathlib.Path)
    args = parser.parse_args()
    try:
        regions = json.loads(args.regions.read_bytes())
    except (OSError, ValueError):
        regions = None
    report = audit(args.evidence, expected_source=args.source, regions=regions)
    print(json.dumps(report, sort_keys=True, allow_nan=False))
    return 0 if report['status'] == 'passed' else 1


if __name__ == '__main__':
    raise SystemExit(main())
