"""The private VM receives only the reviewed person-side test and its helpers."""

import pathlib
import tempfile
import unittest
import zipfile
from unittest.mock import patch

from make_session_archive import build
import installed_session
import approved_session

APPROVAL = ('approved_session.py', 'approved_runtime.py', 'microphone_turn.py',
            'pending_microphone.py', 'record_delta.py', 'record_shape.py',
            'delivery_observation.py', 'asking_turn_observation.py', 'asking_turn_shape.py')


class SessionArchive(unittest.TestCase):
    def test_only_canonical_test_and_native_transport_sources_are_included(self):
        source = pathlib.Path(installed_session.__file__).resolve().parent
        with tempfile.TemporaryDirectory(prefix='studio-session-archive-') as folder:
            output = pathlib.Path(folder) / 'studio-session'
            build(source, output)
            self.assertTrue(zipfile.is_zipfile(output))
            with zipfile.ZipFile(output) as archive:
                self.assertEqual(set(archive.namelist()),
                                 {'__main__.py', 'native_controls.py', 'atspi_tree.py', 'audio_selection.py', 'reference_window.py',
                                  'agent_session.py', 'agent_settings.py', 'agent_process.py', *APPROVAL})
                self.assertEqual(archive.read('__main__.py'),
                                 (source / 'installed_session.py').read_bytes())
                for name in ('native_controls.py', 'atspi_tree.py', 'audio_selection.py', 'reference_window.py',
                             'agent_session.py', 'agent_settings.py', 'agent_process.py', *APPROVAL):
                    self.assertEqual(archive.read(name), (source / name).read_bytes())

    def test_entry_selects_only_the_requested_fixed_flow(self):
        for args, approval in (([], False), (['--approvals'], True)):
            with self.subTest(args=args), patch.object(installed_session, 'Session') as recording, patch.object(approved_session, 'ApprovalSession') as approved:
                installed_session.main(args)
                selected, other = (approved, recording) if approval else (recording, approved)
                selected.assert_called_once_with()
                selected.return_value.run.assert_called_once_with()
                other.assert_not_called()

    def test_unknown_or_extra_arguments_do_not_launch_a_test(self):
        for args in (['--unknown'], ['--approvals', 'anything'], ['--approvals', '--approvals']):
            with self.subTest(args=args), patch.object(installed_session, 'Session') as recording, patch.object(approved_session, 'ApprovalSession') as approved:
                with self.assertRaises(ValueError):
                    installed_session.main(args)
                recording.assert_not_called()
                approved.assert_not_called()


if __name__ == '__main__':
    unittest.main()
