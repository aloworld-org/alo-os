"""Package the private person-side installed test with a closed source list."""

import pathlib
import shutil
import sys
import tempfile
import zipapp


def build(source, output):
    """Create a Python zip application without collecting unrelated local files."""
    with tempfile.TemporaryDirectory(prefix='studio-session-source-') as folder:
        prepared = pathlib.Path(folder)
        shutil.copyfile(source / 'installed_session.py', prepared / '__main__.py')
        for name in ('native_controls.py', 'atspi_tree.py', 'audio_selection.py'):
            shutil.copyfile(source / name, prepared / name)
        zipapp.create_archive(prepared, target=output, interpreter='/usr/bin/env python3',
                              compressed=True)


if __name__ == '__main__':
    if len(sys.argv) != 2:
        raise SystemExit('usage: make_session_archive.py output-path')
    build(pathlib.Path(__file__).resolve().parent, pathlib.Path(sys.argv[1]))
