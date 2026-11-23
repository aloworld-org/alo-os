"""Real GTK4 controls for the private instrument's machine-state compatibility checks."""

import gi
import json
from pathlib import Path
import sys

gi.require_version('Gtk', '4.0')
from gi.repository import Gtk, GLib

configuration = json.loads(Path(sys.argv[1]).read_text())
folder = Path(configuration['folder'])
GLib.set_prgname(configuration['application'])
application = Gtk.Application(application_id=configuration['application'])


def opened(app):
    window = Gtk.ApplicationWindow(application=app, title='Private GTK4 native control fixture')
    window.set_default_size(420, 180)
    box = Gtk.Box(orientation=Gtk.Orientation.VERTICAL)
    window.set_child(box)
    for option in configuration['buttons']:
        button = Gtk.Button(label=option['name'])
        button.set_sensitive(option.get('enabled', True))
        box.append(button)

        def pressed(_button, identifier=option['id']):
            with (folder / 'actions.jsonl').open('a') as output:
                output.write(json.dumps(identifier) + '\n')

        button.connect('clicked', pressed)
        if not option.get('visible', True):
            button.set_visible(False)
    window.present()
    prepared = folder / 'ready.tmp'
    prepared.write_text('ready\n')
    prepared.replace(folder / 'ready')


application.connect('activate', opened)
application.run([])
