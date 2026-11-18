"""Live native accessibility transport on an isolated session and display."""

import os
import pathlib
import subprocess
import tempfile
import time
import unittest

from atspi_tree import AtspiTree
from native_controls import activate

FIXTURE = r'''
import gi, pathlib, sys
gi.require_version('Gtk', '3.0')
from gi.repository import Gtk
window = Gtk.Window(title='Studio walk native control fixture')
button = Gtk.Button(label='Choose reference')
def chosen(_):
    pathlib.Path(sys.argv[1]).write_text('activated once')
    Gtk.main_quit()
button.connect('clicked', chosen)
window.add(button)
window.show_all()
Gtk.main()
'''


@unittest.skipUnless(os.environ.get('STUDIO_PRIVATE_UI_SESSION') == '1',
                     'requires an owned isolated DBus session and X display')
class LiveNativeTree(unittest.TestCase):
    def test_the_actual_native_button_is_read_and_activated_by_process(self):
        tree = AtspiTree()
        with tempfile.TemporaryDirectory(prefix='studio-atspi-test-') as folder:
            result = pathlib.Path(folder) / 'chosen'
            environment = dict(os.environ, GTK_MODULES='atk-bridge', NO_AT_BRIDGE='0')
            process = subprocess.Popen(['python3', '-I', '-c', FIXTURE, str(result)],
                                       env=environment, stdout=subprocess.DEVNULL)
            try:
                deadline = time.monotonic() + 15
                while True:
                    try:
                        activate(tree, process.pid, lambda name: name == 'Choose reference')
                        break
                    except ValueError:
                        if process.poll() is not None or time.monotonic() >= deadline:
                            raise
                        time.sleep(.1)
                process.wait(timeout=5)
                self.assertEqual(process.returncode, 0)
                self.assertEqual(result.read_text(), 'activated once')
            finally:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=5)


if __name__ == '__main__':
    unittest.main()
