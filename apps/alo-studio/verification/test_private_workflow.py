"""Private instrument tests rerun in CI with actual decoder/native transports."""

import pathlib
import unittest


class PrivateWorkflow(unittest.TestCase):
    def test_ci_requires_media_tools_and_executes_live_private_instrument_tests(self):
        root = pathlib.Path(__file__).resolve().parents[3]
        workflow = (root / '.github/workflows/studio.yml').read_text()
        self.assertIn('  verification-instruments:', workflow)
        section = workflow.split('  verification-instruments:', 1)[1]
        self.assertIn('command -v ffmpeg ffprobe', section)
        self.assertIn('STUDIO_PRIVATE_UI_SESSION=1', section)
        self.assertIn('GDK_BACKEND=x11', section)
        self.assertIn('dbus-run-session -- xvfb-run -a', section)
        self.assertIn('python3 -I -m unittest discover', section)
        self.assertIn('-s apps/alo-studio/verification', section)


if __name__ == '__main__':
    unittest.main()
