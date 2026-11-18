"""Validate exact producer artifacts before serving inputs to a private VM."""

import hashlib
import os
import pathlib
import re
import sys
import tempfile

NORMAL = ('alo-studio-0.1.0.flatpak', 'source.txt', 'features.txt', 'sdk-versions.txt')
REFERENCE = ('studio-reference.flatpak', 'studio-session', 'source.txt',
             'reference-features.txt', 'sdk-versions.txt')


def read(path):
    if path.is_symlink() or not path.is_file():
        raise ValueError('producer entries must be regular files')
    return path.read_bytes()


def verified(directory, manifest, names):
    if directory.is_symlink() or not directory.is_dir():
        raise ValueError('a regular producer directory is required')
    raw = read(directory / manifest)
    if len(raw) > 16_384:
        raise ValueError('producer manifest exceeds its inventory bound')
    try:
        text = raw.decode('ascii')
    except UnicodeDecodeError as error:
        raise ValueError('producer manifest is not canonical SHA256 text') from error
    records = {}
    for line in text.splitlines(keepends=True):
        match = re.fullmatch(r'([0-9a-f]{64})  ([A-Za-z0-9_.-]+)\n', line)
        if not match:
            raise ValueError('producer manifest is not canonical SHA256 text')
        checksum, name = match.groups()
        if name not in names or name in records:
            raise ValueError('producer manifest has an unknown or repeated entry')
        records[name] = checksum
    if set(records) != set(names):
        raise ValueError('producer manifest does not cover its exact inventory')
    payloads = {name: read(directory / name) for name in names}
    for name, payload in payloads.items():
        if hashlib.sha256(payload).hexdigest() != records[name]:
            raise ValueError('producer payload checksum mismatch')
    return payloads


def prepare(normal_dir, reference_dir, output_dir, expected_source):
    """Publish fresh byte copies only after both complete inventories validate."""
    if not isinstance(expected_source, str) or not re.fullmatch('[0-9a-f]{40}', expected_source):
        raise ValueError('an exact lowercase source revision is required')
    normal_dir, reference_dir, output_dir = map(pathlib.Path, (normal_dir, reference_dir, output_dir))
    if output_dir.exists() or output_dir.is_symlink():
        raise ValueError('the output must be absent')
    output = output_dir.resolve()
    if any(output == original.resolve() or output.is_relative_to(original.resolve())
           for original in (normal_dir, reference_dir)):
        raise ValueError('prepared inputs must be separate from original artifacts')
    try:
        normal = verified(normal_dir, 'checksums.txt', NORMAL)
        reference = verified(reference_dir, 'reference-checksums.txt', REFERENCE)
        source = (expected_source + '\n').encode('ascii')
        if normal['source.txt'] != source or reference['source.txt'] != source:
            raise ValueError('producer artifacts do not match the requested source')
        if normal['features.txt'] != b'normal; verification disabled\n':
            raise ValueError('the normal application contains verification features')
        if reference['reference-features.txt'] != (
                b'private reference only; verification enabled; never the normal application\n'):
            raise ValueError('the private reference has an unexpected feature identity')
        if not normal['sdk-versions.txt'] or normal['sdk-versions.txt'] != reference['sdk-versions.txt']:
            raise ValueError('the producer runtime and SDK records disagree')
        payloads = {'studio.flatpak': normal['alo-studio-0.1.0.flatpak'],
                    'fixture.flatpak': reference['studio-reference.flatpak'],
                    'studio-session': reference['studio-session'],
                    **{name: normal[name] for name in ('source.txt', 'features.txt', 'sdk-versions.txt')}}
        with tempfile.TemporaryDirectory(prefix='.studio-inputs-', dir=output.parent) as temporary:
            staging = pathlib.Path(temporary) / 'ready'
            staging.mkdir()
            for name, payload in payloads.items():
                (staging / name).write_bytes(payload)
            checksums = ''.join(f'{hashlib.sha256(data).hexdigest()}  {name}\n'
                                for name, data in sorted(payloads.items()))
            (staging / 'checksums.txt').write_text(checksums, encoding='ascii', newline='')
            if output_dir.exists() or output_dir.is_symlink():
                raise ValueError('the output appeared while inputs were being checked')
            os.rename(staging, output_dir)
    except OSError as error:
        raise ValueError('private artifacts could not be prepared') from error


if __name__ == '__main__':
    if len(sys.argv) != 5:
        raise SystemExit('usage: prepare_installed_walk.py normal-dir reference-dir output-dir source-sha')
    prepare(*sys.argv[1:])
