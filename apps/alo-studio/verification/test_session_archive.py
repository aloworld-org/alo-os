"""The private VM receives only the reviewed person-side test and its helpers."""

import pathlib
import tempfile
import unittest
import zipfile

from make_session_archive import build


class SessionArchive(unittest.TestCase):
    def test_only_canonical_test_and_native_transport_sources_are_included(self):
        source = pathlib.Path(__file__).resolve().parent
        with tempfile.TemporaryDirectory(prefix='studio-session-archive-') as folder:
            output = pathlib.Path(folder) / 'studio-session'
            build(source, output)
            self.assertTrue(zipfile.is_zipfile(output))
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()),
                                 {'__main__.py', 'native_controls.py', 'atspi_tree.py', 'audio_selection.py'})
                self.assertEqual(archive.read('__main__.py'),
                                 (source / 'installed_session.py').read_bytes())
                for name in ('native_controls.py', 'atspi_tree.py', 'audio_selection.py'):
                    self.assertEqual(archive.read(name), (source / name).read_bytes())


if __name__ == '__main__':
    unittest.main()
