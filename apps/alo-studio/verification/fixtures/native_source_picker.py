"""Native GTK choice fixture shaped like xdg-desktop-portal-gtk AccessDialog.

Only test-owned widgets and files are involved. This does not reply to a real
portal request or grant a real application capture permission.
"""

import json
from pathlib import Path
import sys

import dbus
import dbus.service
from dbus.mainloop.glib import DBusGMainLoop
import gi

gi.require_version('Gtk', '3.0')
from gi.repository import Gtk


DBusGMainLoop(set_as_default=True)
configuration = json.loads(Path(sys.argv[1]).read_text())
folder = Path(configuration['folder'])
connection = dbus.SessionBus()
owner = dbus.service.BusName(configuration['bus_name'], bus=connection, do_not_queue=True)
dialog = Gtk.MessageDialog(message_type=Gtk.MessageType.QUESTION,
                           buttons=Gtk.ButtonsType.NONE,
                           text='Private native source choice fixture')
dialog.set_title('Private native source choice fixture')
dialog.add_button('Cancel', Gtk.ResponseType.CANCEL)
dialog.add_button('Share', Gtk.ResponseType.OK)
area = dialog.get_message_area()
label = Gtk.Label(label='Source')
area.add(label)
rows = []
group = None
for option in configuration['options']:
    button = Gtk.RadioButton.new_with_label_from_widget(group, option['label'])
    group = group or button
    area.add(button)
    button.set_sensitive(option.get('enabled', True))
    rows.append((option, button))
# The first radio is initially selected, as capture_choice chooses sources.first.
# Signal observation begins afterwards, so a later choice has a distinct effect.
for option, button in rows:
    def toggled(widget, identifier=option['id']):
        if widget.get_active():
            with (folder / 'choices.jsonl').open('a') as output:
                output.write(json.dumps(identifier) + '\n')
    button.connect('toggled', toggled)


def responded(_dialog, response):
    """Record only a real native button response and the selected real radio."""
    selected = [option['id'] for option, button in rows if button.get_active()]
    prepared = folder / 'response.tmp'
    prepared.write_text(json.dumps({
        'code': 0 if response == Gtk.ResponseType.OK else 1,
        'selected': selected[0] if response == Gtk.ResponseType.OK else None,
    }))
    prepared.replace(folder / 'response.json')
    Gtk.main_quit()


dialog.connect('response', responded)
dialog.show_all()
for option, button in rows:
    if not option.get('visible', True):
        button.hide()
Gtk.main()
