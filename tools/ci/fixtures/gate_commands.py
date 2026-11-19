#!/usr/bin/python3 -IB
"""Harmless commands for executing only the gate's APT setup in a fixture.

Every privileged APT/hosts operation is intercepted here. No real sudo,
package manager, resolver or HTTP client is ever executed. The legacy grep/sed
rewrite runs against validated temporary fixture paths only, so the regression
can observe the original HTTP downgrade rather than merely missing source text.
"""

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys


root = Path(os.environ['GATE_FAKE_ROOT']).resolve()
apt = root / 'apt'
program = Path(sys.argv[0]).name
args = sys.argv[1:]
# Also catch an accidentally unprivileged spelling. The fixture must never
# fall through to a real package manager, source writer or helper CLI.
if program in ('tee', 'apt-get', 'sed', 'python3'):
    args = [program, *args]
    program = 'sudo'


def trace(kind, **fields):
    """Append one bounded observation to this test's own log."""
    with (root / 'trace.jsonl').open('a') as output:
        output.write(json.dumps(dict(kind=kind, **fields)) + '\n')


def sources():
    """Source bytes from the fixture only."""
    return {str(path.relative_to(apt)): path.read_text()
            for path in sorted(apt.rglob('*')) if path.is_file()}


def require(condition, message):
    """Refuse unexpected commands without falling through to a host tool."""
    if not condition:
        raise SystemExit('fixture refuses: ' + message)


def local_path(value):
    """Translate standard absolute paths or validate a temporary grep result."""
    if value == '/etc/apt/sources.list':
        path = apt / 'sources.list'
    elif value.rstrip('/') == '/etc/apt/sources.list.d':
        path = apt / 'sources.list.d'
    else:
        path = Path(value).resolve()
    require(path.is_relative_to(apt), 'path outside temporary APT tree')
    return str(path)


if program == 'sudo':
    trace('sudo', argv=args)
    require(bool(args), 'empty sudo')
    if args[0] == 'tee':
        data = sys.stdin.read()
        if args[1:] == ['/etc/apt/apt.conf.d/80-gate-network']:
            trace('network-options', text=data)
        elif args[1:] == ['-a', '/etc/hosts']:
            with (root / 'hosts').open('a') as output:
                output.write(data)
            trace('pinned-host', text=data)
        else:
            require(False, 'unexpected tee target')
        print(data, end='')
    elif args == ['python3', '-I', 'tools/ci/ubuntu_archive.py']:
        path = Path(os.environ['GATE_SOURCE_ROOT']) / 'tools/ci/ubuntu_archive.py'
        spec = importlib.util.spec_from_file_location('tested_ubuntu_archive', path)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        changed = helper.configure(apt)
        trace('archive-helper', changed=changed)
        print(f'Ubuntu archive source files updated: {changed}')
    elif args[0] == 'grep':
        require(args[1:3] == ['-lrE', 'apt-mirrors\\.txt|azure\\.archive'], 'unexpected legacy grep')
        require(len(args) == 5, 'unexpected grep arguments')
        result = subprocess.run(['/usr/bin/grep', *args[1:3],
                                 *map(local_path, args[3:])], check=False)
        raise SystemExit(result.returncode)
    elif args[0] == 'sed':
        require(args[1:3] == ['-i', '-E'], 'unexpected legacy sed')
        require(len(args) == 8 and args[3] == '-e' and args[5] == '-e', 'unexpected sed arguments')
        target = local_path(args[-1])
        result = subprocess.run(['/usr/bin/sed', *args[1:-1], target], check=False)
        raise SystemExit(result.returncode)
    elif args[0] == 'timeout':
        require(len(args) >= 4 and args[2] == 'apt-get', 'unexpected bounded command')
        require(args[1] in ('60', '90', '120'), 'APT deadline changed')
        operation = args[3:]
        mode = os.environ['GATE_FAKE_MODE']
        if operation == ['update']:
            count_path = root / 'updates'
            count = int(count_path.read_text()) + 1 if count_path.exists() else 1
            count_path.write_text(str(count))
            trace('update', seconds=args[1], number=count, sources=sources())
            if mode == 'retry-fails' and count == 2:
                raise SystemExit(124)
        elif operation and operation[0] == 'install':
            require(operation[1:4] == ['--download-only', '--no-install-recommends', '--yes'], 'download flags')
            count_path = root / 'downloads'
            count = int(count_path.read_text()) + 1 if count_path.exists() else 1
            count_path.write_text(str(count))
            trace('download', seconds=args[1], number=count, packages=operation[4:])
            if mode != 'healthy' and count == 1:
                raise SystemExit(124)
            (root / 'downloaded').write_text('checked download completed')
        else:
            require(False, 'unexpected APT operation')
    elif args[0] == 'apt-get':
        require(args[1:5] == ['install', '--no-download', '--no-install-recommends', '--yes'], 'final install must use checked download')
        require((root / 'downloaded').exists(), 'installation before a successful fetch')
        trace('install', packages=args[5:])
    else:
        require(False, 'unexpected sudo command')
elif program == 'getent':
    require(args and args[0] == 'ahostsv4', 'unexpected resolver call')
    require(args[1:] in (['archive.ubuntu.com'], ['security.ubuntu.com']), 'unexpected resolver host')
    trace('resolver', argv=args)
    raise SystemExit(1)
elif program == 'curl':
    trace('public-dns', argv=args)
    require('--max-time' in args and args[args.index('--max-time') + 1] == '5', 'unbounded DNS lookup')
    require(args[-1].startswith(('https://1.1.1.1/', 'https://8.8.8.8/')), 'unexpected network endpoint')
    print('{"data":"198.51.100.24"}')
elif program in ('weston', 'dbus-daemon', 'broadwayd', 'gtk-builder-tool'):
    trace('program', name=program, argv=args)
    print(program + ' fixture')
else:
    require(False, 'unexpected fixture executable')
