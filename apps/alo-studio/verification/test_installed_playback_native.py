"""Exercise the installed-check helper on a real normal app over real AT-SPI.

This local owned-Xvfb native-action seam is independent of the Tab/Space suite.
It is not installed Wayland acceptance or a claim about physical audio/pixels.
"""

import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

from atspi_tree import AtspiTree
from fixtures.review_process import ReviewProcess, media
from native_controls import activate
from review_playback import check


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_REVIEW_SESSION') == '1',
                     'requires a verified normal binary and owned Xvfb')
class ActualNativePlaybackObservation(unittest.TestCase):
    def test_native_actions_and_real_value_properties_pass_the_positional_helper(self):
        self.assertEqual(os.environ.get('STUDIO_REVIEW_PLAYBACK_FEATURE_SCOPE'), 'normal')
        binary = Path(os.environ['STUDIO_REVIEW_PLAYBACK_APP']).resolve(strict=True)
        with tempfile.TemporaryDirectory(prefix='studio-native-review-') as temporary:
            folder = Path(temporary)
            clip = media(folder, 'long', 12)
            workspace = folder / 'app'
            workspace.mkdir()
            app = ReviewProcess(workspace, binary, {'long': clip})
            try:
                with patch('atspi_tree.dbus.SessionBus', return_value=app.session):
                    tree = AtspiTree()
                tree.bus.set_exit_on_disconnect(False)
                try:
                    app.open('long')
                    def prepared():
                        button = app.named('Play recording', 43)
                        scale = app.values()
                        return (button and {24, 25, 30} <= button['states'] and scale
                                and {24, 25, 30} <= scale[0]['states'] and scale[1][1] > 0)
                    app.until(prepared, 'actual native Review did not prepare', 10)
                    presses = []
                    def press(process, matches):
                        self.assertEqual(process, app.process.pid)
                        presses.append('Play recording' if matches('Play recording') else 'Pause playback')
                        activate(tree, process, matches)
                    result = check(tree, app.process.pid, press)
                    self.assertEqual(presses, ['Play recording', 'Pause playback'])
                    self.assertEqual(result['scope'], 'review-playback-position')
                    self.assertGreaterEqual(result['playing'] - result['start'], .5)
                    self.assertLessEqual(abs(result['stable'] - result['paused']), .08)
                    self.assertIsNotNone(app.named('Play recording', 43))
                    if os.environ.get('STUDIO_REVIEW_PLAYBACK_EVIDENCE'):
                        app.capture('installed-helper-local-native-paused')
                finally:
                    tree.bus.close()
            finally:
                app.close()


if __name__ == '__main__':
    unittest.main()
